mod errors;
pub use errors::{Error, Result};

mod coredata;
pub use coredata::{
    coredata_fix, delete_domains, export_all_domains, export_domain, export_domains, list_domains,
    save_domain_map, DeleteDefaultsMacOSResult, DEFAULT_DOMAINS,
};
mod key_chain_data;
pub use key_chain_data::KeychainData;

pub mod ad_manager;
use std::collections::BTreeSet;

pub use ad_manager::{
    boot_up, boot_up_system_agent_or_daemon, boot_up_user_agent_or_daemon, list_agents_and_daemons,
    list_agents_and_daemons_paths, turn_off, turn_off_system_agent_or_daemon,
    turn_off_user_agent_or_daemon, Uid,
};

pub fn no_doubles(list: &[&str]) -> Vec<String> {
    let mut set = BTreeSet::<String>::new();
    set.extend(list.iter().map(|o| o.to_string()));
    let mut no_doubles = Vec::from_iter(set.iter());
    no_doubles.sort();
    no_doubles.iter().map(|o| o.to_string()).collect::<Vec<String>>()
}
