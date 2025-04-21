use serde::{Deserialize, Serialize};

const DEFAULT_BACKUP_PATH: &'static str = "~/cdb-adm-backup";
const DEFAULT_SETTINGS_PATH: &'static str = "~/.config/cdb-adm.toml";
const DEFAULT_CDB_SETTINGS_PATH: &'static str = "~/.config/cdb.toml";
const DEFAULT_ADM_SETTINGS_PATH: &'static str = "~/.config/adm.toml";

pub fn default_backup_path() -> iocore::Path {
    use iocore::{env, Path};
    env::var("CDB_ADM_BACKUP_PATH")
        .map(|path| Path::raw(path))
        .unwrap_or_else(|_| Path::raw(DEFAULT_BACKUP_PATH))
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(rename = "backup-path")]
    #[serde(rename(deserialize = "backup_path"))]
    backup_path: Option<String>,

    pub cdb: Option<CDBSettings>,
    pub adm: Option<ADMSettings>,
    path: iocore::Path,
}
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            path: iocore::Path::raw(DEFAULT_SETTINGS_PATH),
            cdb: Some(CDBSettings::default()),
            adm: Some(ADMSettings::default()),
            backup_path: Some(default_backup_path().to_string()),
        }
    }
}
impl SettingsEnvPath for Settings {
    fn env_var_name() -> &'static str {
        "CDB_ADM_SETTINGS"
    }

    fn default_path() -> &'static str {
        DEFAULT_SETTINGS_PATH
    }
}

impl Settings {
    pub fn backup_path(&self) -> iocore::Path {
        self.backup_path
            .clone()
            .map(|path| iocore::Path::raw(path))
            .unwrap_or_else(|| iocore::Path::cwd())
            .try_canonicalize()
    }

    pub fn validate(&self) -> crate::Result<()> {
        self.validate_backup_path()?;
        Ok(())
    }

    pub fn validate_backup_path(&self) -> crate::Result<()> {
        let path = self.backup_path();
        if path.is_file() {
            Err(crate::Error::ConfigurationError(format!(
                "'backup_path' cannot be a file: {:#?}",
                path.to_string()
            )))
        } else {
            Ok(())
        }
    }
    pub fn cdb(&self) -> CDBSettings {
        self.cdb.clone().unwrap_or_default()
    }
    pub fn adm(&self) -> ADMSettings {
        self.adm.clone().unwrap_or_default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDBSettingsDelete {
    pub domains: Vec<String>,
    pub keys: Vec<Vec<String>>,
}
impl Default for CDBSettingsDelete {
    fn default() -> CDBSettingsDelete {
        CDBSettingsDelete {
            domains: Vec::new(),
            keys: Vec::new(),
        }
    }
}
impl CDBSettingsDelete {
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
    delete: CDBSettingsDelete,
}
impl SettingsEnvPath for CDBSettings {
    fn env_var_name() -> &'static str {
        "CDM_SETTINGS"
    }

    fn default_path() -> &'static str {
        DEFAULT_CDB_SETTINGS_PATH
    }
}

impl CDBSettings {
    pub fn defaults_exec_args(&self) -> Vec<Vec<String>> {
        self.delete.defaults_exec_args()
    }
}
impl Default for CDBSettings {
    fn default() -> CDBSettings {
        CDBSettings {
            delete: CDBSettingsDelete::default(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADMSettings {
    display_warnings: bool,
    include_non_needed: bool,
    include_system_uids: bool,
}
impl Default for ADMSettings {
    fn default() -> ADMSettings {
        ADMSettings {
            display_warnings: true,
            include_non_needed: true,
            include_system_uids: true,
        }
    }
}
impl SettingsEnvPath for ADMSettings {
    fn env_var_name() -> &'static str {
        "ADM_SETTINGS"
    }

    fn default_path() -> &'static str {
        DEFAULT_ADM_SETTINGS_PATH
    }
}

pub trait SettingsEnvPath: Default + serde::de::DeserializeOwned {
    fn env_var_name() -> &'static str;
    fn default_path() -> &'static str;
    fn from_env() -> crate::Result<Self> {
        let path = match iocore::env::var(Self::env_var_name()) {
            Ok(path) => iocore::Path::readable_file(path.to_string())?,
            Err(_) => iocore::Path::new(Self::default_path()),
        };
        Ok(Self::from_path(&path)?)
    }
    fn from_path(path: &iocore::Path) -> crate::Result<Self> {
        if !path.is_file() {
            return Err(crate::Error::SettingsError(format!(
                "config {} does not exist",
                path.to_string()
            )));
        }
        let settings = toml::from_str::<Self>(&path.read()?).map_err(|error| {
            crate::Error::SettingsError(format!(
                "reading toml from config path {:#?}: {}",
                path.to_string(),
                error
            ))
        })?;
        Ok(settings)
    }
    fn detect() -> crate::Result<Self> {
        match Self::from_env() {
            Ok(settings) => Ok(settings),
            Err(_) => Ok(Self::from_path(&iocore::Path::from(Self::default_path()))?),
        }
    }
    fn cli(quiet: bool) -> Self {
        match Self::detect() {
            Ok(settings) => settings,
            Err(error) => {
                if !quiet {
                    eprintln!("[warning] loading settings: {}", error);
                }
                Self::default()
            },
        }
    }
}
