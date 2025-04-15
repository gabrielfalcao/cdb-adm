use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDBSettingsDelete {
    pub domains: Vec<String>,
    pub keys: Vec<Vec<String>>,
}
impl CDBSettingsDelete {
    pub fn new() -> CDBSettingsDelete {
        CDBSettingsDelete {
            domains: Vec::new(),
            keys: Vec::new(),
        }
    }

    pub fn defaults_exec_args(&self) -> Vec<Vec<String>> {
        let mut args = Vec::<Vec<String>>::new();
        args.extend(
            self.domains
                .clone()
                .iter()
                .map(|domain| vec!["delete".to_string(), domain.to_string()])
                .collect::<Vec<Vec<String>>>(),
        );
        args.extend(self.keys.clone().iter().map(|keys| {
            let mut args = vec!["delete".to_string()];
            args.extend(keys.iter().map(|key| key.to_string()).collect::<Vec<String>>());
            args
        }));
        args
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDBSettings {
    #[serde(rename(serialize = "backup-path", deserialize = "backup-path"))]
    #[serde(rename(deserialize = "backup_path"))]
    backup_path: Option<iocore::Path>,
    delete: CDBSettingsDelete,
}
impl CDBSettings {
    pub fn new() -> CDBSettings {
        CDBSettings {
            backup_path: None,
            delete: CDBSettingsDelete::new(),
        }
    }

    pub fn backup_path(&self) -> iocore::Path {
        self.backup_path.clone().unwrap_or_else(|| iocore::Path::cwd()).try_canonicalize()
    }

    pub fn from_path(path: &iocore::Path) -> crate::Result<CDBSettings> {
        if !path.exists() {
            Err(crate::Error::IOError(format!("{} does not exist", path)))
        } else if !path.is_file() {
            Err(crate::Error::IOError(format!(
                "{}({}) exists but is not a file",
                path,
                path.kind()
            )))
        } else {
            Ok(toml::from_str::<CDBSettings>(&path.read()?)?)
        }
    }

    pub fn from_env() -> crate::Result<CDBSettings> {
        use iocore::Path;
        let path = match std::env::var("CDB_SETTINGS") {
            Ok(path) => Path::readable_file(path.to_string()).unwrap(),
            Err(_) => Path::new("~/.config/cdb.toml"),
        };
        Ok(CDBSettings::from_path(&path)?)
    }

    pub fn cli(quiet: bool) -> CDBSettings {
        match CDBSettings::from_env() {
            Ok(settings) => settings,
            Err(error) => {
                if !quiet {
                    eprintln!("[warning] loading settings: {}", error);
                }
                CDBSettings::new()
            },
        }
    }

    pub fn defaults_exec_args(&self) -> Vec<Vec<String>> {
        self.delete.defaults_exec_args()
    }
}
impl Default for CDBSettings {
    fn default() -> CDBSettings {
        CDBSettings::from_env().unwrap()
    }
}
