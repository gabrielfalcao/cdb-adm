use crate::{no_doubles, Error};

mod adm;
mod launchctl;
mod parser;
use std::collections::BTreeSet;

pub use adm::{
    agents_and_daemons_path_map, list_agents_and_daemons, list_agents_and_daemons_paths,
    salient_system_uids, system_uids, Uid,
};
pub use launchctl::{
    agent_or_daemon, agent_or_daemon_prefix, bootout_agent_or_daemon, launchctl, launchctl_ok,
    list_active_agents_and_daemons, list_active_agents_and_daemons_by_domain,
    turn_off_agent_or_daemon,
};
pub use parser::{extract_service_info_opt, extract_service_name, parse_services};

pub const NON_NEEDED_SERVICES: [&'static str; 307] = include!("agents-and-daemons.noon");
pub const BOOTOUT_SERVICES: [&'static str; 56] = include!("bootout.noon");

pub fn turn_off(
    uid: Option<Uid>,
    gui: bool,
    quiet: bool,
    silent_warnings: bool,
    user_services: Vec<String>,
    system_services: Vec<String>,
    include_non_needed: bool,
    include_system_uids: bool,
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
        turn_off_system_agent_or_daemon(ad, quiet, gui, silent_warnings, &mut success, &mut errors);
    }
    if !quiet {
        println!("turning off user({}) services", uid.unwrap_or_default());
    }
    for ad in &user_services_set {
        turn_off_user_agent_or_daemon(
            ad,
            uid,
            gui,
            quiet,
            silent_warnings,
            &mut success,
            &mut errors,
        );
    }
    if include_non_needed && include_system_uids {
        for uid in system_uids() {
            for ad in &user_services_set {
                turn_off_user_agent_or_daemon(
                    ad,
                    uid,
                    gui,
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
    gui: bool,
    quiet: bool,
    silent_warnings: bool,
) -> (Vec<String>, Vec<(String, Error)>) {
    let mut errors = Vec::<(String, Error)>::new();
    let mut success = Vec::<String>::new();
    if !quiet {
        println!("turning off system services");
    }

    for ad in BOOTOUT_SERVICES {
        match bootout_agent_or_daemon(&ad, None, gui) {
            Ok(_) => {},
            Err(Error::LaunchdServiceNotRunning(e)) =>
                if !silent_warnings {
                    eprintln!("bootout {}[warning] {}", &ad, e);
                },
            Err(e) => {
                errors.push((agent_or_daemon(&ad, uid.clone(), gui).to_string(), e.clone()));
            },
        };
    }
    if !quiet {
        println!("turning off user({}) services", uid.unwrap_or_default());
    }
    for ad in BOOTOUT_SERVICES {
        match bootout_agent_or_daemon(&ad, uid.clone(), gui) {
            Ok(_) => {
                success.push(agent_or_daemon(&ad, uid.clone(), gui).to_string());
            },
            Err(Error::LaunchdServiceNotRunning(e)) => {
                success.push(agent_or_daemon(&ad, uid.clone(), gui).to_string());

                if !silent_warnings {
                    eprintln!("bootout {}[warning] {}", &ad, e);
                }
            },
            Err(e) => {
                errors.push((agent_or_daemon(&ad, uid.clone(), gui).to_string(), e.clone()));
            },
        };
    }
    (success, errors)
}

pub fn turn_off_system_agent_or_daemon(
    n: impl std::fmt::Display,
    gui: bool,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match turn_off_agent_or_daemon(&n, None, gui, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} turned off", agent_or_daemon(&n, None, gui));
            }
            success.push(agent_or_daemon(&n, None, gui));
        },
        Err(Error::LaunchdServiceNotRunning(_)) => {},
        Err(e) => {
            // if !quiet {
            //     eprintln!(
            //         "{} might be already turned off: {:#?}",
            //         agent_or_daemon(&n, None, gui),
            //         e.to_string()
            //     );
            // }
            errors.push((agent_or_daemon(&n, None, gui), e));
        },
    }
}
pub fn turn_off_user_agent_or_daemon(
    n: impl std::fmt::Display,
    uid: Option<Uid>,
    gui: bool,
    quiet: bool,
    silent_warnings: bool,
    success: &mut Vec<String>,
    errors: &mut Vec<(String, Error)>,
) {
    let n = n.to_string();
    match turn_off_agent_or_daemon(&n, uid, gui, silent_warnings) {
        Ok(_) => {
            if !quiet {
                println!("{} seems to be turned off now - ", agent_or_daemon(&n, uid, gui));
            }
            success.push(agent_or_daemon(&n, uid, gui));
        },
        Err(Error::LaunchdServiceNotRunning(_)) => {},
        Err(e) => {
            if !quiet {
                println!(
                    "{} might be already turned off: {:#?} -",
                    agent_or_daemon(&n, uid, gui),
                    e.to_string()
                );
            }
            errors.push((agent_or_daemon(&n, uid, gui), e));
        },
    }
}

pub fn turn_off_smart(
    uid: &Uid,
    quiet: bool,
    services: Vec<String>,
    include_non_needed: bool,
    include_system_uids: bool,
) {
    let mut services_set = BTreeSet::<String>::new();

    services_set.extend(services);
    if include_non_needed {
        services_set.extend(no_doubles(&NON_NEEDED_SERVICES));
    }

    let services_to_turn_off = list_active_agents_and_daemons(uid, include_system_uids)
        .unwrap()
        .iter()
        .filter(|(_, service, _, _)| {
            for name in &services_set {
                if service.as_str().contains(name.as_str()) {
                    return true;
                } else {
                    if service.as_str().contains(name.as_str()) {
                        eprintln!("[info] {:#?} contains {:#?}", service.as_str(), name.as_str());
                        return true;
                    } else if name.as_str().contains(service.as_str()) {
                        eprintln!("[info] {:#?} contains {:#?}", name.as_str(), service.as_str());
                        return true;
                    }
                }
            }
            false
        })
        .map(|(domain, service, pid, _)| (domain.to_string(), service.to_string(), *pid))
        .collect::<Vec<(String, String, i64)>>();
    if !services_to_turn_off.is_empty() {
        if !quiet {
            println!("turning off services");
        }
    } else {
        if !quiet {
            println!("ok");
        }
    }
    for (domain, service, pid) in services_to_turn_off {
        match bootout_and_disable_smart(uid, &domain, &service) {
            Ok(_) =>
                if !quiet {
                    println!("{}/{} ({}) turned off", &domain, &service, pid);
                },
            Err(error) =>
                if !quiet {
                    eprintln!("{}", error)
                },
        }
    }
}

fn launchctl_bootout_or_disable_smart(
    subcommand: &str,
    service_target: &str,
    as_root: bool,
) -> std::result::Result<(), String> {
    let args = vec![subcommand.to_string(), service_target.to_string()];
    match launchctl_ok(crate::to_slice_str!(args), as_root) {
        Ok((0, _, _)) => Ok(()),
        Ok((exit_code, _, err)) =>
            Err(format!("`launchctl {}' failed with {}: {}", args.join(" "), exit_code, err)),
        Err(err) => Err(err.to_string()),
    }?;
    Ok(())
}
fn bootout_and_disable_smart(
    uid: &Uid,
    domain: &str,
    service: &str,
) -> std::result::Result<(), String> {
    let as_root = !domain.ends_with(&uid.to_string());
    let services_target = format!("{}/{}", domain, service);
    launchctl_bootout_or_disable_smart("bootout", services_target.as_str(), as_root)?;
    launchctl_bootout_or_disable_smart("disable", services_target.as_str(), as_root)?;
    Ok(())
}
