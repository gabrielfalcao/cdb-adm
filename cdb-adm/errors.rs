use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Error {
    IOError(String),
    JsonError(String),
    LaunchdError(String),
    LaunchdServiceNotRunning(String),
    ParseIntError(String),
    KeychainError(String),
    PlistError(String),
    TomlError(String),
    CoreDataError(String),
    ParseError(String),
    SystemError(String),
    ConfigurationError(String),
    SettingsError(String),
}
impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}: {}",
            self.variant(),
            match self {
                Self::IOError(e) => e.to_string(),
                Self::JsonError(e) => e.to_string(),
                Self::LaunchdError(e) => e.to_string(),
                Self::LaunchdServiceNotRunning(e) => e.to_string(),
                Self::ParseIntError(e) => e.to_string(),
                Self::KeychainError(e) => e.to_string(),
                Self::PlistError(e) => e.to_string(),
                Self::TomlError(e) => e.to_string(),
                Self::CoreDataError(e) => e.to_string(),
                Self::ParseError(e) => e.to_string(),
                Self::SystemError(e) => e.to_string(),
                Self::ConfigurationError(e) => e.to_string(),
                Self::SettingsError(e) => e.to_string(),
            }
        )
    }
}

impl Error {
    pub fn variant(&self) -> String {
        match self {
            Error::IOError(_) => "IOError",
            Error::JsonError(_) => "JsonError",
            Error::LaunchdError(_) => "LaunchdError",
            Error::LaunchdServiceNotRunning(_) => "LaunchdServiceNotRunning",
            Error::ParseIntError(_) => "ParseIntError",
            Error::KeychainError(_) => "KeychainError",
            Error::PlistError(_) => "PlistError",
            Error::TomlError(_) => "TomlError",
            Error::CoreDataError(_) => "CoreDataError",
            Error::ParseError(_) => "ParseError",
            Error::SystemError(_) => "SystemError",
            Error::ConfigurationError(_) => "ConfigurationError",
            Error::SettingsError(_) => "SettingsError",
        }
        .to_string()
    }
}

impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IOError(e.to_string())
    }
}
impl From<std::num::ParseIntError> for Error {
    fn from(e: std::num::ParseIntError) -> Self {
        Error::ParseIntError(e.to_string())
    }
}
impl From<iocore::Error> for Error {
    fn from(e: iocore::Error) -> Self {
        Error::IOError(e.to_string())
    }
}
impl From<plist::Error> for Error {
    fn from(e: plist::Error) -> Self {
        Error::PlistError(e.to_string())
    }
}
impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::JsonError(e.to_string())
    }
}
impl From<security_framework::base::Error> for Error {
    fn from(e: security_framework::base::Error) -> Self {
        Error::KeychainError(e.to_string())
    }
}
impl From<toml::de::Error> for Error {
    fn from(e: toml::de::Error) -> Error {
        Error::TomlError(e.to_string())
    }
}
impl From<std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>> for Error {
    fn from(e: std::io::IntoInnerError<std::io::BufWriter<Vec<u8>>>) -> Self {
        Error::IOError(format!("{}", e))
    }
}
pub type Result<T> = std::result::Result<T, Error>;
//find . -type f -name 'errors.rs*' -exec refactors -wp {} '(Error::\w+)e.to_string[(][)]' '$1(e.to_string())' \;
