pub struct Uid(u64);
impl Default for Uid {
    fn default() -> Uid {
        Uid(501)
    }
}
impl Into<u64> for Uid {
    fn into(self) -> u64 {
        self.0
    }
}
impl From<u64> for Uid {
    fn from(u: u64) -> Uid {
        Uid(u)
    }
}
impl From<Option<u64>> for Uid {
    fn from(u: Option<u64>) -> Uid {
        match u {
            Some(u) => Uid(u),
            None => Uid::default(),
        }
    }
}
impl Copy for Uid {}
impl Clone for Uid {
    fn clone(&self) -> Uid {
        Uid(self.0)
    }
}
impl std::str::FromStr for Uid {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Uid, crate::Error> {
        Ok(Uid(u64::from_str_radix(s, 10)?))
    }
}
impl std::fmt::Display for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::fmt::Debug for Uid {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Uid({})", self.0)
    }
}

pub fn list_agents_and_daemons(
    uid: Option<Uid>,
    with_qualifier: bool,
    user: bool,
    library: bool,
    system: bool,
    quiet: bool,
) -> crate::Result<Vec<String>> {
    let uid = uid.unwrap_or_default();
    use iocore::Path;
    let mut agents_and_daemons = Vec::<String>::new();

    for path in list_agents_and_daemons_paths(user, library, system)? {
        let prefix = if path.to_string().starts_with("/System") {
            format!("system")
        } else {
            format!("gui/{}", &uid)
        };
        let label = match determine_agent_or_daemon_label(&Path::raw(&path)) {
            Ok(label) => label,
            Err(error) => {
                if !quiet {
                    eprintln!("\x1b[1;38;5;220m{} skipped: {}\x1b[0m", &path, error.to_string());
                }
                continue;
            },
        };
        if with_qualifier {
            agents_and_daemons.push(format!("{}/{}", prefix, label));
        } else {
            agents_and_daemons.push(label);
        }
    }
    Ok(agents_and_daemons)
}

pub fn list_agents_and_daemons_paths(
    user: bool,
    library: bool,
    system: bool,
) -> crate::Result<Vec<iocore::Path>> {
    use iocore::Path;
    let mut agents_and_daemons_paths = Vec::<Path>::new();
    let mut search_paths = Vec::<Path>::new();
    if user {
        search_paths.push(Path::new("~/Library/LaunchAgents"));
        search_paths.push(Path::new("~/Library/LaunchDaemons"));
    }
    if library {
        search_paths.push(Path::new("/Library/LaunchAgents"));
        search_paths.push(Path::new("/Library/LaunchDaemons"));
    }
    if system {
        search_paths.push(Path::new("/System/Library/LaunchAgents"));
        search_paths.push(Path::new("/System/Library/LaunchDaemons"));
    }

    for path in search_paths {
        for agent_or_daemon_path in path.list()? {
            if let Some(extension) = agent_or_daemon_path.extension() {
                if extension == ".plist" {
                    agents_and_daemons_paths.push(agent_or_daemon_path.clone());
                }
            }
        }
    }
    Ok(agents_and_daemons_paths)
}

pub fn determine_agent_or_daemon_label(
    agent_or_daemon_path: &iocore::Path,
) -> crate::Result<String> {
    use plist::{from_file, Dictionary, Value};
    let fallback_label = agent_or_daemon_path.without_extension().name();
    if let Some(extension) = agent_or_daemon_path.extension() {
        if extension == ".plist" {
            let agent_or_daemon_plist =
                from_file::<std::path::PathBuf, Dictionary>(agent_or_daemon_path.to_path_buf())?;
            let label = agent_or_daemon_plist
                .get("Label")
                .map(|label| match label {
                    Value::String(label) => label.to_string(),
                    _ => fallback_label.to_string(),
                })
                .unwrap_or_else(|| fallback_label.to_string());
            return Ok(label);
        } else {
            Ok(fallback_label)
        }
    } else {
        Ok(fallback_label)
    }
}
