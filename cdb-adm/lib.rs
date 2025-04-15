mod errors;
pub use errors::{Error, Result};

mod settings;
pub use settings::CDBSettings;
mod md;
pub use md::turn_off_mdutil;
mod coredata;
pub use coredata::{
    coredata_fix, delete_domains, export_all_domains, export_domain, export_domains,
    export_library_preferences, export_plists_from_path, list_domains, DeleteDefaultsMacOSResult,
};
mod key_chain_data;
pub use key_chain_data::KeychainData;

pub mod ad_manager;
use std::collections::BTreeSet;

pub use ad_manager::{
    agent_or_daemon, agent_or_daemon_prefix, agents_and_daemons_path_map, boot_out, launchctl,
    launchctl_ok, list_agents_and_daemons, list_agents_and_daemons_paths, turn_off,
    turn_off_system_agent_or_daemon, turn_off_user_agent_or_daemon, Uid,
};

pub fn no_doubles(list: &[&str]) -> Vec<String> {
    let mut set = BTreeSet::<String>::new();
    set.extend(list.iter().map(|o| o.to_string()));
    let mut no_doubles = Vec::from_iter(set.iter());
    no_doubles.sort();
    no_doubles.iter().map(|o| o.to_string()).collect::<Vec<String>>()
}

pub fn escalate() -> Result<()> {
    let user = iocore::User::id()?;
    if user.name() == "root" && user.uid() == 0 {
        Ok(())
    } else {
        Err(Error::IOError(format!("{}[uid {}] is not root", &user.name, &user.uid)))
    }
}
