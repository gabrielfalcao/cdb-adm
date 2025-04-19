use std::process::{Command, Stdio};

use crate::{parse_services, to_slice_str, Error, Result, Uid};

pub fn turn_off_agent_or_daemon(
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    gui: bool,
    silent_warnings: bool,
) -> Result<()> {
    // println!("next input turns off '{}'. [ENTER] to continue", agent_or_daemon(&ad, uid.clone()));
    // let mut line = String::new();
    // std::io::stdin().read_line(&mut line).unwrap();
    match bootout_agent_or_daemon(&ad, uid.clone(), gui) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("bootout {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    match disable_agent_or_daemon(&ad, uid, gui) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("disable {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    Ok(())
}

pub fn agent_or_daemon_prefix(uid: Option<Uid>, gui: bool) -> String {
    match uid {
        Some(uid) => format!("{}/{}", if gui { "gui" } else { "user" }, uid),
        None => format!("system"),
    }
}
pub fn agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>, gui: bool) -> String {
    format!("{}/{}", agent_or_daemon_prefix(uid, gui), ad)
}

pub fn launchctl_act(
    subcommand: impl std::fmt::Display,
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    gui: bool,
) -> Result<i64> {
    let args = vec![subcommand.to_string(), agent_or_daemon(&ad, uid, gui)];
    let (exit_code, out, err) = launchctl(
        &args
            .iter()
            .filter(|domain| !domain.is_empty())
            .map(|domain| domain.as_str())
            .collect::<Vec<&str>>(),
        uid.is_none(),
    )?;
    if exit_code == 0 {
        return Ok(0);
    }
    eprintln!(
        "<'launchctl {}'>{}: <stdout>{}</stdout><stderr>{}</stderr>",
        args.join(" "),
        exit_code,
        out.trim(),
        err.trim()
    );

    match exit_code {
        3 | 125 => Err(Error::LaunchdServiceNotRunning(agent_or_daemon(&ad, uid, gui).to_string())),
        exit_code => Err(Error::LaunchdError(format!(
            "`launchctl {}' failed with exit code {:#?}: {}",
            args.join(" "),
            exit_code,
            err
        ))),
    }
}
pub fn launchctl(args: &[&str], as_root: bool) -> Result<(i64, String, String)> {
    match launchctl_ok(args, as_root)? {
        (0, out, err) => Ok((0, out, err)),
        (3 | 125, _, err) => Err(Error::LaunchdServiceNotRunning(format!(
            "`launchctl {}' failed: {}",
            args.join(" "),
            err
        ))),
        (exit_code, _, err) => Err(Error::LaunchdError(format!(
            "`launchctl {}' failed with exit code {:#?}: {}",
            args.join(" "),
            exit_code,
            err
        ))),
    }
}

pub fn bootout_agent_or_daemon(
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    gui: bool,
) -> Result<i64> {
    Ok(launchctl_act("bootout", ad, uid, gui)?)
}
pub fn disable_agent_or_daemon(
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    gui: bool,
) -> Result<i64> {
    Ok(launchctl_act("disable", ad, uid, gui)?)
}
pub fn launchctl_ok(args: &[&str], as_root: bool) -> Result<(i64, String, String)> {
    let username = if as_root {
        "root".to_string()
    } else {
        let user = iocore::User::id()?;
        if user.uid == 0 {
            return Err(Error::IOError(format!("cannot run launchctl as non-root as root")));
        } else {
            user.name.to_string()
        }
    };
    let args = vec![
        "su".to_string(),
        "-l".to_string(),
        username,
        "-c".to_string(),
        format!("launchctl {}", args.join(" ")),
    ];
    let mut cmd = Command::new("sudo");
    let cmd = cmd.current_dir("/System");
    let cmd = cmd.args(to_slice_str!(args));
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
pub fn launchctl_print(domain: &str) -> Result<String> {
    let args = vec!["print".to_string(), domain.to_string()];
    let (exit_code, out, err) = launchctl_ok(to_slice_str!(args), false)?;
    if exit_code == 0 {
        return Ok(out);
    }
    match exit_code {
        exit_code => Err(Error::LaunchdError(format!(
            "`launchctl {}' failed with exit code {:#?}: {}\n{}",
            args.join(" "),
            exit_code,
            err,
            out
        ))),
    }
}
pub fn list_active_agents_and_daemons_by_domain(
    uid: &Uid,
) -> crate::Result<std::collections::BTreeMap<String, (i64, Option<i64>)>> {
    let mut map = std::collections::BTreeMap::<String, (i64, Option<i64>)>::new();
    let domains = vec![
        agent_or_daemon_prefix(None, false),
        agent_or_daemon_prefix(Some(uid.clone()), false),
        agent_or_daemon_prefix(Some(uid.clone()), true),
    ];
    for domain in domains {
        for (pid, status, service) in parse_services(&launchctl_print(&domain)?)? {
            map.insert(format!("{}/{}", domain.to_string(), service.to_string()), (pid, status));
        }
    }
    Ok(map)
}
pub fn list_active_agents_and_daemons(
    uid: &Uid,
    include_system_uids: bool,
) -> crate::Result<Vec<(String, String, i64, Option<i64>)>> {
    let mut services = Vec::<(String, String, i64, Option<i64>)>::new();
    let mut domains = vec![
        agent_or_daemon_prefix(None, false),
        agent_or_daemon_prefix(Some(uid.clone()), false),
        agent_or_daemon_prefix(Some(uid.clone()), true),
    ];
    if include_system_uids {
        for suid in crate::salient_system_uids() {
            domains.push(agent_or_daemon_prefix(Some(suid.clone()), false));
            domains.push(agent_or_daemon_prefix(Some(suid.clone()), true));
        }
    }
    for domain in domains {
        for (pid, status, service) in parse_services(&match launchctl_print(&domain) {
            Ok(services) => services,
            Err(_) => continue,
        })? {
            services.push((domain.to_string(), service.to_string(), pid, status));
        }
    }
    Ok(services)
}

#[cfg(test)]
mod tests {
    use crate::{agent_or_daemon, agent_or_daemon_prefix, Uid};

    #[test]
    fn test_agent_or_daemon_prefix() {
        assert_eq!(agent_or_daemon_prefix(None, false), "system");
        assert_eq!(agent_or_daemon_prefix(Some(Uid::from(242)), false), "user/242");
        assert_eq!(agent_or_daemon_prefix(Some(Uid::from(202)), true), "gui/202");
    }
    #[test]
    fn test_agent_or_daemon() {
        assert_eq!(
            agent_or_daemon("com.apple.calaccessd", None, false),
            "system/com.apple.calaccessd"
        );
        assert_eq!(
            agent_or_daemon("com.apple.calaccessd", Some(Uid::from(242)), false),
            "user/242/com.apple.calaccessd"
        );
        assert_eq!(
            agent_or_daemon("com.apple.calaccessd", Some(Uid::from(202)), true),
            "gui/202/com.apple.calaccessd"
        );
    }
}
