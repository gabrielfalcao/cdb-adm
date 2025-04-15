use std::process::{Command, Stdio};

use crate::{Error, Result, Uid};

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
    match launchctl(
        &args
            .iter()
            .filter(|domain| !domain.is_empty())
            .map(|domain| domain.as_str())
            .collect::<Vec<&str>>(),
    ) {
        Ok((0, _, _)) => Ok(0),
        Ok((3 | 125, _, _)) =>
            Err(Error::LaunchdServiceNotRunning(agent_or_daemon(&ad, uid, gui).to_string())),
        Ok((exit_code, _, err)) => Err(Error::LaunchdError(format!(
            "`launchctl {}' failed with exit code {:#?}: {}",
            args.join(" "),
            exit_code,
            err
        ))),
        Err(e) => Err(e),
    }
}
pub fn launchctl(args: &[&str]) -> Result<(i64, String, String)> {
    match launchctl_ok(args)? {
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
pub fn launchctl_ok(args: &[&str]) -> Result<(i64, String, String)> {
    let mut cmd = Command::new("launchctl");
    let cmd = cmd.current_dir("/System");
    let cmd = cmd.args(args);
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
