mod adm;
mod launchctl;
mod parser;

pub use adm::{
    agents_and_daemons_path_map, list_agents_and_daemons, list_agents_and_daemons_paths,
    salient_system_uids, system_uids, Uid,
};
pub use launchctl::{
    agent_or_daemon, agent_or_daemon_prefix, bootout_agent_or_daemon, launchctl, launchctl_ok,
    list_active_agents_and_daemons, list_all_agents_and_daemons, list_disabled_agents_and_daemons,
    turn_off_agent_or_daemon,
};
pub use parser::{extract_service_info_opt, extract_service_name, parse_services};

pub const NON_NEEDED_SERVICES: [&'static str; 123] = include!("agents-and-daemons.noon");
pub const BOOTOUT_SERVICES: [&'static str; 56] = include!("bootout.noon");

pub fn turn_off_smart(
    uid: &Uid,
    quiet: bool,
    services: Vec<String>,
    include_non_needed: bool,
    include_system_uids: bool,
) {
    let mut services_set = Vec::<String>::new();
    if include_non_needed {
        services_set.extend(crate::to_vec_string!(NON_NEEDED_SERVICES));
    }
    services_set.extend(services);
    let services_set = crate::no_doubles(crate::to_slice_str!(services_set));
    let services_to_turn_off = list_active_agents_and_daemons(uid, include_system_uids)
        .unwrap()
        .iter()
        .filter(|(_, service, _, _, _)| {
            for name in &services_set {
                if service.as_str() == name.as_str() {
                    return true;
                } else if service.as_str().contains(name.as_str()) {
                    return true;
                } else if name.as_str().contains(service.as_str()) {
                    eprintln!("--------------------------------------------------------------------------------");
                    eprintln!("[info] given service name {:#?} contains actual service name {:#?}", name.as_str(), service.as_str());
                    eprintln!("--------------------------------------------------------------------------------");
                    return true;
                }
            }
            false
        })
        .map(|(domain, service, pid, _, _)| (domain.to_string(), service.to_string(), *pid))
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
    use iocore::Path;
    for (domain, service, pid) in services_to_turn_off {
        let log_base_path = Path::cwd().join("logs").join(service.as_str()).join(domain.replace("/", "-"));
        log_base_path.join("launchd.0.log").write(&Path::raw("/private/var/log/com.apple.xpc.launchd/launchd.log").read_bytes().unwrap()).unwrap();
        match bootout_disable_and_kill_smart(uid, &domain, &service) {
            Ok(_) => {
                if !quiet {
                    println!("{}/{} ({}) turned off", &domain, &service, pid);
                }
                let launchd_log = Path::raw("/private/var/log/com.apple.xpc.launchd/launchd.log").read().unwrap();
                log_base_path.join("launchd.1.log").write(launchd_log.as_bytes()).unwrap();

                // if launchd_log.contains("failed lookup: name = com.apple.modelmanager") {
                //     eprintln!("\n------------------------------------------------------------------------------------------------");
                //     eprintln!("launchd error detected, check {:#?}", log_base_path.to_string());
                //     eprintln!("\n------------------------------------------------------------------------------------------------");
                //     std::process::exit(1);
                // }
                // <Notice>: Last log repeated 1 times
                // (system) <Warning>: failed lookup: name = com.apple.modelmanager, requestor = coreaudiod[418], error = 3: No such process
                // (system) <Warning>: failed lookup: name = com.apple.BTAudioHALPlugin.xpc, requestor = coreaudiod[418], error = 3: No such process
                // println!("[enter]");
                // std::io::stdin().read_line(&mut String::new()).unwrap();

                // match test_osx_app_opens() {
                //     Ok(_) => {},
                //     Err(error) => {

                //         eprintln!("\n----------------------------------------------------------");
                //         eprintln!("app does not open: {}", error);
                //         eprintln!("----------------------------------------------------------\n");
                //         std::process::exit(3);
                //     },
                // }
            },
            Err(error) =>
                if !quiet {
                    eprintln!("{}", error)
                },
        }
    }
}

fn launchctl_subcommand(args: &[&str], as_root: bool) -> crate::Result<i64> {
    match launchctl_ok(args, as_root) {
        Ok((exit_code, _, err)) => match exit_code {
            0 | 64 | 113 => Ok(exit_code),
            3 | 125 => Err(crate::Error::LaunchdServiceNotRunning(format!(
                "`launchctl {}' failed with {}: {}",
                args.join(" "),
                exit_code,
                err
            ))),
            _ => Err(crate::Error::LaunchdError(format!(
                "`launchctl {}' failed with {}: {}",
                args.join(" "),
                exit_code,
                err
            ))),
        },
        Err(err) => Err(err),
    }
}

#[rustfmt::skip]
fn bootout_disable_and_kill_smart(
    uid: &Uid,
    domain: &str,
    service: &str,
) -> crate::Result<()> {
    let as_root = !domain.ends_with(&uid.to_string());
    let services_target = format!("{}/{}", domain, service);
    launchctl_subcommand(crate::to_slice_str!(vec!["bootout".to_string(), services_target.to_string()]), as_root)?;
    launchctl_subcommand(crate::to_slice_str!(vec!["disable".to_string(), services_target.to_string()]), as_root)?;
    launchctl_subcommand(crate::to_slice_str!(vec!["kill".to_string(),
                                                   "9".to_string(),       services_target.to_string()]), as_root)?;
    Ok(())
}

#[rustfmt::skip]
fn enable_and_kickstart_smart(
    uid: &Uid,
    domain: &str,
    service: &str,
    path: &iocore::Path,
) -> crate::Result<()> {
    let as_root = !domain.ends_with(&uid.to_string());
    let services_target = format!("{}/{}", domain, service);
    launchctl_subcommand(crate::to_slice_str!(vec!["enable".to_string(), services_target.to_string()]), as_root)?;
    launchctl_subcommand(crate::to_slice_str!(vec!["bootstrap".to_string(), services_target.to_string(), path.to_string()]), as_root)?;
    Ok(())
}

pub fn boot_up_smart(uid: &Uid, quiet: bool, services: Vec<String>, include_non_needed: bool) {
    let mut services_set = Vec::<String>::new();
    if include_non_needed {
        services_set.extend(crate::to_vec_string!(NON_NEEDED_SERVICES));
    }
    services_set.extend(services);
    let services_to_boot_up = list_all_agents_and_daemons(uid)
        .unwrap()
        .iter()
        .filter(|(_, service, _, _, _, info)| {
            if info.is_none() {
                if !quiet {
                    eprintln!("[warning] path not found for {:#?}", &service);
                }
                return false;
            }
            for name in &services_set {
                if service.as_str() == name.as_str() {
                    return true;
                } else if service.as_str().contains(name.as_str()) {
                    return true;
                } else if name.as_str().contains(service.as_str()) {
                    eprintln!("--------------------------------------------------------------------------------");
                    eprintln!("[info] given service name {:#?} contains actual service name {:#?}", name.as_str(), service.as_str());
                    eprintln!("--------------------------------------------------------------------------------");
                    return true;
                }
            }
            if !quiet {
                eprintln!("[warning] no matches for service {:#?}", &service);
            }
            false
        })
        .map(|(domain, service, pid, _, _, info)| {
            (domain.to_string(), service.to_string(), *pid, info.clone().unwrap())
        })
        .collect::<Vec<(String, String, i64, (iocore::Path, plist::Dictionary))>>();

    if !services_to_boot_up.is_empty() {
        if !quiet {
            println!("booting-up services");
        }
    } else {
        if !quiet {
            println!("ok");
        }
    }
    for (domain, service, pid, (path, _)) in services_to_boot_up {
        match enable_and_kickstart_smart(uid, &domain, &service, &path) {
            Ok(_) =>
                if !quiet {
                    println!("{}/{} ({}) booted-up", &domain, &service, pid);
                },
            Err(error) =>
                if !quiet {
                    eprintln!("{}", error)
                },
        }
    }
}

pub fn test_osx_app_opens() -> crate::Result<bool> {
    let app_name = "Firefox.app";
    let cli_name = "firefox";
    use std::process::{Command, Stdio};
    let mut cmd = Command::new("/usr/bin/open");
    let cmd = cmd.current_dir(".");
    let cmd = cmd.args(crate::to_slice_str!(vec![format!("/Applications/{}", app_name)]));
    let cmd = cmd.stdin(Stdio::null());
    let cmd = cmd.stdout(Stdio::piped());
    let cmd = cmd.stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;

    let exit_code: i64 = output.status.code().unwrap_or_default().into();
    if exit_code == 0 {
        let timeout = std::time::Duration::from_secs(10);
        wait_for_subprocess_with_status(format!("g p -qr {}", cli_name).as_str(), true, &timeout)
            .map_err(|msg| {
            crate::Error::IOError(format!("waiting for {} to be up: {}", &app_name, msg))
        })?;
        std::thread::sleep(std::time::Duration::from_millis(33));

        wait_for_subprocess_with_status(
            format!("g p -qr {} -k", cli_name).as_str(),
            true,
            &timeout,
        )
        .map_err(|msg| {
            crate::Error::IOError(format!("waiting for {} to die: {}", &app_name, msg))
        })?;
        std::thread::sleep(std::time::Duration::from_millis(33));
        wait_for_subprocess_with_status(format!("g p -qr {}", cli_name).as_str(), false, &timeout)
            .map_err(|msg| {
                crate::Error::IOError(format!("waiting for {} NOT be up: {}", &app_name, msg))
            })?;
        Ok(true)
    } else {
        let out = String::from_utf8(output.stdout).unwrap();
        let err = String::from_utf8(output.stderr).unwrap();
        Err(crate::Error::IOError(format!("{} does not open({}):\n<stdout>{}</stdout>\n<stderr>{}</stderr>", &app_name, exit_code, out, err)))
    }
}

pub fn wait_for_subprocess_with_status(
    command: &str,
    ok: bool,
    timeout: &std::time::Duration,
) -> Result<(), String> {
    let now = std::time::Instant::now();
    while !subprocess_match(command, ok) {
        std::thread::sleep(std::time::Duration::from_millis(33));
        if now.elapsed() > *timeout {
            return Err(format!(
                "timed out waiting for command {:#?} within {}s",
                command,
                timeout.as_secs()
            ));
        }
    }
    Ok(())
}

pub fn subprocess_match(command: &str, ok: bool) -> bool {
    iocore::shell_command(command, ".")
        .map(|status| if ok { status == 0 } else { status != 0 })
        .unwrap_or_else(|_| !ok)
}
