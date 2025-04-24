mod errors;
pub use errors::{Error, Result};

pub mod cli;
pub use cli::{adb,cdb};
mod settings;
pub use settings::{ADMSettings, CDBSettings, Settings, SettingsEnvPath};
mod md;
pub use md::turn_off_mdutil;
mod spctl;
pub use spctl::spctl_global_disable;
mod coredata;
pub use coredata::{
    coredata_fix, defaults_delete, defaults_delete_domain, defaults_write, delete_domains,
    export_all_domains, export_domain, export_domains, export_library_preferences,
    export_plists_from_path, list_domains, DeleteDefaultsMacOSResult,
};
mod key_chain_data;
pub use key_chain_data::KeychainData;

pub mod ad_manager;
use std::collections::BTreeSet;

pub use ad_manager::{
    agent_or_daemon, agent_or_daemon_prefix, agents_and_daemons_path_map, boot_up_smart,
    extract_service_info_opt, extract_service_name, launchctl, launchctl_ok,
    list_active_agents_and_daemons, list_agents_and_daemons, list_agents_and_daemons_paths,
    list_all_agents_and_daemons, parse_services, salient_system_uids, system_uids, turn_off_smart,
    Uid
};

pub fn no_doubles(list: &[&str]) -> Vec<String> {
    let mut set = BTreeSet::<String>::new();
    set.extend(list.iter().map(|o| o.to_string()));
    let mut no_doubles = Vec::from_iter(set.iter());
    no_doubles.sort();
    no_doubles.iter().map(|o| o.to_string()).collect::<Vec<String>>()
}

#[macro_export]
macro_rules! to_vec_string {
    ($slice:expr) => {
        $slice.iter().map(|j| j.to_string()).collect::<Vec<String>>()
    };
}
#[macro_export]
macro_rules! to_slice_str {
    ($vec_string:expr) => {
        &$vec_string.iter().map(|j| j.as_str()).collect::<Vec<&str>>()
    };
}

// #[macro_export]
// macro_rules! settings_from_env {
//     ($struct_name:ident, $env_var_name:expr, $default_path:expr) => {
//         pub fn from_env() -> crate::Result<$struct_name> {
//             let path = match iocore::env::var($env_var_name) {
//                 Ok(path) => iocore::Path::readable_file(path.to_string())?,
//                 Err(_) => iocore::Path::new($default_path),
//             };
//             Ok(<$struct_name>::from_path(&path)?)
//         }
//     };
// }
// #[macro_export]
// macro_rules! settings_from_path {
//     ($struct_name:ident, $default_path:expr) => {
//         pub fn from_path(path: &iocore::Path) -> crate::Result<$struct_name> {
//             if !path.is_file() {
//                 return Err(crate::Error::SettingsError(format!(
//                     "config {} does not exist",
//                     path.to_string()
//                 )));
//             }
//             let settings = toml::from_str::<$struct_name>(&path.read()?).map_err(|error| {
//                 crate::Error::SettingsError(format!(
//                     "reading toml from config path {:#?}: {}",
//                     path.to_string(),
//                     error
//                 ))
//             })?;
//             Ok(settings)
//         }
//     };
// }

// #[macro_export]
// macro_rules! settings_detect {
//     ($struct_name:ident, $default_path:expr) => {
//         pub fn detect() -> crate::Result<$struct_name> {
//             match $struct_name::from_env() {
//                 Ok(settings) => Ok(settings),
//                 Err(_) => Ok($struct_name::from_path(&iocore::Path::from($default_path))?),
//             }
//         }
//     };
// }

// #[macro_export]
// macro_rules! settings_cli {
//     ($struct_name:ident, $default_path:expr) => {
//         pub fn cli(quiet: bool) -> $struct_name {
//             match $struct_name::detect() {
//                 Ok(settings) => settings,
//                 Err(error) => {
//                     if !quiet {
//                         eprintln!("[warning] loading settings: {}", error);
//                     }
//                     <$struct_name>::default()
//                 },
//             }
//         }
//     };
// }
