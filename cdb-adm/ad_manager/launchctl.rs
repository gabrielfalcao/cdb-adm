use std::process::{Command, Stdio};

use crate::{Error, Result, Uid};

pub fn turn_off_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<()> {
    // println!("next input turns off '{}'. [ENTER] to continue", agent_or_daemon(&ad, uid.clone()));
    // let mut line = String::new();
    // std::io::stdin().read_line(&mut line).unwrap();
    bootout_agent_or_daemon(&ad, uid.clone())?;
    disable_agent_or_daemon(&ad, uid)?;
    Ok(())
}

pub fn agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> String {
    match uid {
        Some(uid) => format!("gui/{}/{}", uid, ad),
        None => format!("system/{}", ad),
    }
}

pub fn boot_up_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<()> {
    kickstart_agent_or_daemon(&ad, uid.clone())?;
    enable_agent_or_daemon(&ad, uid)?;
    Ok(())
}

pub fn launchctl(
    subcommand: impl std::fmt::Display,
    ad: impl std::fmt::Display,
    uid: Option<Uid>,
) -> Result<i32> {
    let command = format!("launchctl {} {}", &subcommand, &agent_or_daemon(ad, uid));
    let (exit_code, _, err) = shell_command_string_output(&command, "/System")?;
    if exit_code != 0 {
        Err(Error::LaunchdError(format!(
            "`{}' failed with exit code {:#?}: {}",
            &command, exit_code, err
        )))
    } else {
        Ok(exit_code)
    }
}

pub fn bootout_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("bootout", ad, uid)?)
}
pub fn disable_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("disable", ad, uid)?)
}

pub fn kickstart_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("kickstart", ad, uid)?)
}
pub fn enable_agent_or_daemon(ad: impl std::fmt::Display, uid: Option<Uid>) -> Result<i32> {
    Ok(launchctl("enable", ad, uid)?)
}

pub fn shell_command_string_output(
    command: impl std::fmt::Display,
    current_dir: impl std::fmt::Display,
) -> Result<(i32, String, String)> {
    let args = command
        .to_string()
        .split(" ")
        .map(|arg| arg.trim().to_string())
        .collect::<Vec<String>>();
    let mut cmd = Command::new(args[0].clone());
    let cmd = cmd.current_dir(current_dir.to_string());
    let cmd = cmd.args(args[1..].to_vec());
    let cmd = cmd.stdin(Stdio::null());
    let cmd = cmd.stdout(Stdio::piped());
    let cmd = cmd.stderr(Stdio::piped());
    let child = cmd.spawn()?;
    let output = child.wait_with_output()?;
    let status = output.status.code().unwrap_or_default();
    Ok((
        status,
        String::from_utf8(output.stdout).unwrap_or_default(),
        String::from_utf8(output.stderr).unwrap_or_default(),
    ))
}
