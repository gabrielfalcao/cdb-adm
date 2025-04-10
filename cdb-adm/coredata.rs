use crate::{Error, Result};
mod consts;
use std::collections::BTreeMap;
use std::process::{Command, Stdio};

pub use consts::DEFAULT_DOMAINS;
use serde::{Deserialize, Serialize};

pub fn export_domain(domain: impl std::fmt::Display) -> Result<plist::Value> {
    let domain = domain.to_string();
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let path = iocore::Path::raw("/tmp").join(format!("{}-{}", &domain, &ts));

    let (exit_code, _, err) = defaults(&["export", &domain.to_string(), &path.to_string()])?;
    if exit_code != 0 {
        return Err(Error::IOError(format!(
            "defaults export {} failed[{}]: {}",
            &domain, exit_code, err
        )));
    }
    let bytes = match path.read_bytes() {
        Ok(bytes) => {
            path.delete_unchecked();
            bytes
        },
        Err(e) => {
            path.delete_unchecked();
            return Err(e.into());
        },
    };
    let plist = plist::from_bytes::<plist::Value>(&bytes)?;
    Ok(plist)
}

pub fn defaults_delete_domain(domain: impl std::fmt::Display) -> Result<(String, plist::Value)> {
    let domain = domain.to_string();
    let plist = export_domain(&domain)?;
    let shell_result = defaults(&["delete", &domain.to_string()])?;
    match shell_result {
        (0, _, _) => Ok((domain, plist)),
        (exit_code, _, err) => Err(Error::IOError(format!(
            "defaults delete {} failed[{}]: {}",
            &domain, exit_code, err
        ))),
    }
}
pub fn delete_domains(domains: &[&str]) -> Result<DeleteDefaultsMacOSResult> {
    let mut errors = BTreeMap::<String, Error>::new();
    let mut domain_map = export_domains(domains, true)?;

    for domain in domains {
        match defaults_delete_domain(&domain) {
            Ok((domain, plist)) => {
                domain_map.insert(domain.to_string(), plist);
            },
            Err(e) => {
                errors.insert(domain.to_string(), e);
            },
        }
        save_domain_map(&domain, domain_map.clone());
    }
    Ok(DeleteDefaultsMacOSResult { domain_map, errors })
}
pub fn save_domain_map(domain: impl std::fmt::Display, domains: BTreeMap<String, plist::Value>) {
    let domain = domain.to_string();
    let data = serde_json::to_string_pretty(&domains).unwrap_or_default();
    if data.is_empty() {
        return;
    }
    let ts = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let path = iocore::Path::raw("/tmp")
        .join(format!("{}-{}", &domain, &ts))
        .try_canonicalize();

    path.write_unchecked(data.as_bytes());
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeleteDefaultsMacOSResult {
    pub domain_map: BTreeMap<String, plist::Value>,
    pub errors: BTreeMap<String, Error>,
}
pub fn list_domains() -> Result<Vec<String>> {
    match defaults(&["domains"])? {
        (0, csv, _) =>
            Ok(csv.split(",").map(|domain| domain.trim().to_string()).collect::<Vec<String>>()),
        (exit_code, _, error_message) => Err(Error::IOError(format!(
            "command `default domains' exited[{}]: {}",
            exit_code, error_message
        ))),
    }
}
pub fn export_domains(domains: &[&str], global: bool) -> Result<BTreeMap<String, plist::Value>> {
    let mut data = BTreeMap::<String, plist::Value>::new();
    if global {
        data.insert("NSGlobalDomain".to_string(), export_domain("NSGlobalDomain")?);
    }
    for domain in domains {
        data.insert(domain.to_string(), export_domain(&domain)?);
    }
    Ok(data)
}
pub fn export_all_domains() -> Result<BTreeMap<String, plist::Value>> {
    Ok(export_domains(
        &list_domains()?
            .iter()
            .filter(|domain| !domain.is_empty())
            .map(|domain| domain.as_str())
            .collect::<Vec<&str>>(),
        true,
    )?)
}
pub fn defaults(args: &[&str]) -> Result<(i64, String, String)> {
    let (exit_code, stdout, stderr) = defaults_ok(args)?;
    if exit_code == 0 {
        Ok((exit_code, stdout, stderr))
    } else {
        let command = format!("defaults {}", args.join(" "));
        Err(Error::IOError(format!(
            "command `{}' failed with exit code {}",
            command, exit_code
        )))
    }
}
pub fn defaults_ok(args: &[&str]) -> Result<(i64, String, String)> {
    let mut cmd = Command::new("defaults");
    let cmd = cmd.current_dir("/System");
    let cmd = cmd.args(args);
    let cmd = cmd.stdin(Stdio::null());
    let cmd = cmd.stdout(Stdio::piped());
    let cmd = cmd.stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;
    let exit_code: i64 = output.status.code().unwrap_or_default().into();
    Ok((
        exit_code,
        String::from_utf8(output.stdout).unwrap_or_default(),
        String::from_utf8(output.stderr).unwrap_or_default(),
    ))
}

pub fn coredata_fix(quiet: bool) -> Result<()> {
    for args in vec![
        vec!["delete", "-globalDomain", "NSLinguisticDataAssetsRequested"],
        vec!["delete", "-globalDomain", "NSPreferredWebServices"],
        vec!["delete", "-globalDomain", "AppleInterfaceStyle"],
        vec![
            "delete",
            "-globalDomain",
            "com.apple.gms.availability.useCasesWhoseAssetsNotReady",
        ],
        vec!["delete", "-globalDomain", "com.apple.gms.availability.disallowedUseCases"],
        vec!["write", "-globalDomain", "AppleLanguages", "-array", "\"en-US\""],
        vec![
            "write",
            "-globalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
            "-array",
            "us",
        ],
        vec![
            "write",
            "-globalDomain",
            "NSLinguisticDataAssetsRequestedByChecker",
            "-dict",
            "KB_SpellingLanguage",
            "-dict",
            "KB_SpellingLanguageIsAutomatic",
            "false",
        ],
        vec!["write", "-globalDomain", "AppleShowScrollBars", "-string", "Always"],
    ] {
        defaults_ok(&args)?;
        if !quiet {
            eprintln!("defaults {} -", args.join(" "));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use plist::{Dictionary, Value};

    use super::{export_all_domains, list_domains};
    use crate::Result;
    #[test]
    fn test_list_domains() -> Result<()> {
        let domains = list_domains()?;
        assert_eq!(domains.is_empty(), false);
        assert_eq!(domains.contains(&"com.apple.FontBook".to_string()), true);
        Ok(())
    }
    #[test]
    fn test_export_all_domains() -> Result<()> {
        let domains: BTreeMap<String, plist::Value> = export_all_domains()?;
        assert_eq!(domains.is_empty(), false);
        assert_eq!(domains.contains_key(&"com.apple.Safari".to_string()), true);
        let safari = match domains.get("com.apple.Safari").unwrap() {
            Value::Dictionary(safari) => safari.clone(),
            _ => Dictionary::new(),
        };
        let extensions_enabled = match safari.get("ExtensionsEnabled").unwrap() {
            Value::Boolean(extensions_enabled) => extensions_enabled.clone(),
            _ => false,
        };
        assert_eq!(extensions_enabled, true);
        Ok(())
    }
}
