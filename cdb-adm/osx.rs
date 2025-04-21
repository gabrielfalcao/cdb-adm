const PAM_D_SUDO_PATH: &'static str = "/etc/pam.d/sudo";

pub fn write_pamd_touch_id(quiet: bool) -> crate::Result<()> {
    escalate()?;
    if !supports_touch_id() {
        return Err(crate::Error::SystemError(format!("does not support touch id")));
    }
    let path = iocore::Path::raw(PAM_D_SUDO_PATH);
    path.write(include_bytes!("./osx/pam.d/sudo"))?;
    Ok(())
}
pub fn supports_touch_id() -> crate::Result<bool> {
    for path in iocore::Path::raw("/usr/lib/pam").list()? {
        if !path.is_file() {
            continue;
        }
        if path.name().starts_with("pam_tid.so") {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn cleanup(settings: &crate::Settings) -> crate::Result<()> {
    escalate()?;
    // /Library/Preferences/Logging/
    for path in iocore::Path::raw("/Library/Logs/DiagnosticReports")? {
        backup_and_delete_path(&path, settings)?
    }
    Ok(())
}
pub fn backup_and_delete_path(
    path: &iocore::Path,
    settings: &crate::Settings,
) -> crate::Result<()> {
    escalate()?;
    let target_path = settings.backup_path.join(path.name().strip_prefix("/"));
    target_path.write(path.read_bytes()?)?;
    Ok(())
}
