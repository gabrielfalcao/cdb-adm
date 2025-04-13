pub fn turn_off_mdutil() -> crate::Result<()> {
    mdutil(&["-i", "off"])?;
    mdutil(&["-d"])?;
    Ok(())
}
pub fn mdutil(args: &[&str]) -> crate::Result<(i64, String, String)> {
    use std::process::{Command, Stdio};
    let mut cmd = Command::new("mdutil");
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
