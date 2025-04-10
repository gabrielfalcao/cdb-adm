use security_framework::os::macos::keychain::{SecKeychain, SecPreferencesDomain};

use crate::Result;

const ACCOUNT: &'static str = "cdb-adm";

#[derive(Clone, PartialEq, Eq)]
pub struct KeychainData {
    service: String,
    data: Vec<u8>,
}

impl std::fmt::Debug for KeychainData {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "KeychainData[service: {:#?}, account: {:#?}, data: {:#?}]",
            &self.service,
            ACCOUNT,
            self.data(),
        )
    }
}

impl KeychainData {
    pub fn new(service: impl std::fmt::Display, data: &[u8]) -> KeychainData {
        let service = service.to_string();
        let data = data.to_vec();
        KeychainData { service, data }
    }

    pub fn save(&mut self) -> Result<()> {
        let keychain = SecKeychain::default_for_domain(SecPreferencesDomain::User)?;
        keychain.set_generic_password(self.service.as_str(), ACCOUNT, &self.data)?;
        Ok(())
    }

    pub fn get(service: impl std::fmt::Display) -> Result<KeychainData> {
        let service = service.to_string();

        let keychain = SecKeychain::default()?;
        let (item, _) = keychain.find_generic_password(service.as_str(), ACCOUNT)?;
        let data = item.to_vec();
        Ok(KeychainData::new(service, &data))
    }

    pub fn delete(&mut self) -> Result<()> {
        let keychain = SecKeychain::default()?;
        let (_, item) = keychain.find_generic_password(self.service.as_str(), ACCOUNT)?;
        item.delete();
        Ok(())
    }

    pub fn data(&self) -> String {
        String::from_utf8(self.data.clone()).unwrap_or_else(|_| hex::encode(&self.data))
    }
}
#[cfg(test)]
mod test {
    use crate::{Error, KeychainData, Result};

    #[test]
    fn test_get_set_delete() -> Result<()> {
        let mut keychain_data = KeychainData::new("cdb-adm-test", b"cdb adm data");
        keychain_data.save()?;
        let result = KeychainData::get("cdb-adm-test");
        assert_eq!(&result, &Ok(keychain_data));

        result?.delete()?;

        assert_eq!(
            KeychainData::get("cdb-adm-test"),
            Err(Error::KeychainError(format!(
                "The specified item could not be found in the keychain."
            )))
        );
        Ok(())
    }
}
