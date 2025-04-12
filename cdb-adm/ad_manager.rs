use crate::{no_doubles, Error};

mod adm;
mod launchctl;
use std::collections::BTreeSet;

pub use adm::{list_agents_and_daemons, list_agents_and_daemons_paths, system_uids, Uid};
pub use launchctl::{
    agent_or_daemon, boot_up_agent_or_daemon, bootout_agent_or_daemon, turn_off_agent_or_daemon,
};

pub const NON_NEEDED_SERVICES: [&'static str; 240] = include!("agents-and-daemons.noon");
pub const BOOTOUT_SERVICES: [&'static str; 58] = include!("bootout.noon");

pub fn turn_off(
    uid: Option<Uid>,
    quiet: bool,
    silent_warnings: bool,
    user_services: Vec<String>,
    system_services: Vec<String>,
    include_non_needed: bool,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut errors = Vec::<(String, Error)>::new();
    let mut success = Vec::<String>::new();
    let mut system_services_set = BTreeSet::<String>::new();
    let mut user_services_set = BTreeSet::<String>::new();

    if include_non_needed {
        system_services_set.extend(no_doubles(&NON_NEEDED_SERVICES));
    }
    system_services_set.extend(system_services);

    if include_non_needed {
        user_services_set.extend(no_doubles(&NON_NEEDED_SERVICES));
    }
    user_services_set.extend(user_services);

    if !quiet {
        println!("turning off system services");
    }

    for ad in &system_services_set {
        turn_off_system_agent_or_daemon(ad, quiet, silent_warnings, &mut success, &mut errors);
    }
    if !quiet {
        println!("turning off user({}) services", uid.unwrap_or_default());
    }
    for ad in &user_services_set {
        turn_off_user_agent_or_daemon(ad, uid, quiet, silent_warnings, &mut success, &mut errors);
    }
    if include_non_needed {
        for uid in system_uids() {
            for ad in &user_services_set {
                turn_off_user_agent_or_daemon(
                    ad,
                    uid,
                    quiet,
                    silent_warnings,
                    &mut success,
                    &mut errors,
                );
            }
        }
    }

    (success, errors)
}

pub fn boot_out(
    uid: Option<Uid>,
    quiet: bool,
    silent_warnings: bool,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut errors = Vec::<(String, Error)>::new();
    let mut success = Vec::<String>::new();
    if !quiet {
        println!("turning off system services");
    }

    for ad in BOOTOUT_SERVICES {
        match bootout_agent_or_daemon(&ad, None) {
            Ok(_) => {},
            Err(Error::LaunchdServiceNotRunning(e)) =>
                if !silent_warnings {
                    eprintln!("bootout {}[warning] {}", &ad, e);
                },
            Err(e) => {
                errors.push((agent_or_daemon(&ad, uid.clone()).to_string(), e.clone()));
            },
        };
    }
    if !quiet {
        println!("turning off user({}) services", uid.unwrap_or_default());
    }
    for ad in BOOTOUT_SERVICES {
        match bootout_agent_or_daemon(&ad, uid.clone()) {
            Ok(_) => {
                success.push(agent_or_daemon(&ad, uid.clone()).to_string());
            },
            Err(Error::LaunchdServiceNotRunning(e)) => {
                success.push(agent_or_daemon(&ad, uid.clone()).to_string());

                if !silent_warnings {
                    eprintln!("bootout {}[warning] {}", &ad, e);
                }
            },
            Err(e) => {
                errors.push((agent_or_daemon(&ad, uid.clone()).to_string(), e.clone()));
            },
        };
    }
    (success, errors)
}

pub fn turn_off_system_agent_or_daemon(
    n: impl std::fmt::Display,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match turn_off_agent_or_daemon(&n, None, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} turned off", agent_or_daemon(&n, None));
            }
            success.push(agent_or_daemon(&n, None));
        },
        Err(e) => {
            // if !quiet {
            //     eprintln!(
            //         "{} might be already turned off: {:#?}",
            //         agent_or_daemon(&n, None),
            //         e.to_string()
            //     );
            // }
            errors.push((agent_or_daemon(&n, None), e));
        },
    }
}
pub fn turn_off_user_agent_or_daemon(
    n: impl std::fmt::Display,
    uid: Option<Uid>,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match turn_off_agent_or_daemon(&n, uid, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} seems to be turned off now.", agent_or_daemon(&n, uid));
            }
            success.push(agent_or_daemon(&n, uid));
        },
        Err(e) => {
            if !quiet {
                println!(
                    "{} might be already turned off: {:#?}",
                    agent_or_daemon(&n, uid),
                    e.to_string()
                );
            }
            errors.push((agent_or_daemon(&n, uid), e));
        },
    }
}

pub fn boot_up(
    uid: Option<Uid>,
    quiet: bool,
    silent_warnings: bool,
    user_services: Vec<String>,
    system_services: Vec<String>,
    include_non_needed: bool,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut errors = Vec::<(String, Error)>::new();
    let mut success = Vec::<String>::new();
    let mut system_services_set = BTreeSet::<String>::new();
    let mut user_services_set = BTreeSet::<String>::new();

    if include_non_needed {
        system_services_set.extend(no_doubles(&NON_NEEDED_SERVICES));
    }
    system_services_set.extend(system_services);

    if include_non_needed {
        user_services_set.extend(no_doubles(&NON_NEEDED_SERVICES));
    }
    user_services_set.extend(user_services);

    if !quiet {
        println!("booting up system services");
    }

    for ad in &system_services_set {
        boot_up_system_agent_or_daemon(&ad, quiet, silent_warnings, &mut success, &mut errors);
    }
    if !quiet {
        println!("booting up user({}) services", uid.unwrap_or_default());
    }
    for ad in &user_services_set {
        boot_up_user_agent_or_daemon(&ad, uid, quiet, silent_warnings, &mut success, &mut errors);
    }
    if include_non_needed {
        for uid in system_uids() {
            for ad in &user_services_set {
                boot_up_user_agent_or_daemon(
                    &ad,
                    uid,
                    quiet,
                    silent_warnings,
                    &mut success,
                    &mut errors,
                );
            }
        }
    }

    (success, errors)
}

pub fn boot_up_system_agent_or_daemon(
    n: impl std::fmt::Display,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match boot_up_agent_or_daemon(&n, None, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} turned up.", agent_or_daemon(&n, None));
            }
            success.push(agent_or_daemon(&n, None));
        },
        Err(e) => {
            if !quiet {
                println!(
                    "{} might be already turned up: {:#?}",
                    agent_or_daemon(&n, None),
                    e.to_string()
                );
            }
            errors.push((agent_or_daemon(&n, None), e));
        },
    }
}
pub fn boot_up_user_agent_or_daemon(
    n: impl std::fmt::Display,
    uid: Option<Uid>,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match boot_up_agent_or_daemon(&n, uid, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} turned up.", agent_or_daemon(&n, uid));
            }
            success.push(agent_or_daemon(&n, uid));
        },
        Err(e) => {
            if !quiet {
                println!(
                    "{} might be already turned up: {:#?}",
                    agent_or_daemon(&n, uid),
                    e.to_string()
                );
            }
            errors.push((agent_or_daemon(&n, uid), e));
        },
    }
}
