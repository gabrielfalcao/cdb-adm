use std::process::{Command, Stdio};

use crate::{Error, Result, Uid};

pub fn turn_off_agent_or_daemon(
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    silent_warnings: bool,
) -> Result<()> {
    // println!("next input turns off '{}'. [ENTER] to continue", agent_or_daemon(&ad, uid.clone()));
    // let mut line = String::new();
    // std::io::stdin().read_line(&mut line).unwrap();
    match bootout_agent_or_daemon(&ad, uid.clone()) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("bootout {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    match disable_agent_or_daemon(&ad, uid) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("disable {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    Ok(())
}

pub fn agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> String {
    match uid {
        Some(uid) => format!("gui/{}/{}", uid, ad),
        None => format!("system/{}", ad),
    }
}

pub fn boot_up_agent_or_daemon(
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
    silent_warnings: bool,
) -> Result<()> {
    match bootstrap_agent_or_daemon(&ad, uid.clone()) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("bootstrap {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    match enable_agent_or_daemon(&ad, uid) {
        Ok(_) => {},
        Err(Error::LaunchdServiceNotRunning(e)) =>
            if !silent_warnings {
                eprintln!("enable {}[warning] {}", &ad, e);
            },
        Err(e) => return Err(e),
    };
    Ok(())
}

pub fn launchctl(
    subcommand: impl std::fmt::Display,
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
) -> Result<i32> {
    let args = vec![subcommand.to_string(), agent_or_daemon(&ad, uid)];
    let (exit_code, _, err) = launchctl_ok(
        &args
            .iter()
            .filter(|domain| !domain.is_empty())
            .map(|domain| domain.as_str())
            .collect::<Vec<&str>>(),
    )?;

    match exit_code {
        0 => Ok(exit_code.try_into().unwrap_or_default()),
        3 | 125 => Err(Error::LaunchdServiceNotRunning(agent_or_daemon(&ad, uid).to_string())),
        exit_code => Err(Error::LaunchdError(format!(
            "`launchctl {}' failed with exit code {:#?}: {}",
            args.join(" "),
            exit_code,
            err
        ))),
    }
}

pub fn bootout_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("bootout", ad, uid)?)
}
pub fn disable_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("disable", ad, uid)?)
}
pub fn bootstrap_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("bootstrap", ad, uid)?)
}
pub fn enable_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("enable", ad, uid)?)
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
