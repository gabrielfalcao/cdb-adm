use crate::{Error, Result};

pub fn parse_services(data: &str, disabled: bool) -> Result<Vec<(i64, Option<i64>, String, bool)>> {
    let mut services = Vec::new();
    let mut in_services = false;
    for line in data.lines() {
        if !in_services
            && line.trim_start().starts_with(if disabled {
                "disabled services = "
            } else {
                "services = "
            })
        {
            in_services = true;
        } else if in_services {
            if line.trim() == "}" {
                break;
            } else {
                if disabled {
                    let pid = 0;
                    let status = None;
                    let (service, enabled) =
                        extract_disabled_service_info(line).ok_or_else(|| {
                            Error::ParseError(format!(
                                "disabled service name not found in ```{}```",
                                line
                            ))
                        })?;
                    services.push((pid, status, service, enabled));
                } else {
                    let (pid, status, service) =
                        extract_service_info_opt(line).map_err(|error| {
                            Error::ParseError(format!("service info not found in ```{}```: {}", line.trim(), error.to_string()))
                        })?;
                    services.push((pid, status, service, true));
                }
            }
        }
    }
    Ok(services)
}

pub fn extract_service_name(line: &str) -> Result<String> {
    let (_, _, service) = extract_service_info_opt(line)
        .map_err(|error| Error::ParseError(format!("service name not found in {:#?}: {}", line, error.to_string())))?;
    Ok(service)
}

pub fn extract_service_info_opt(line: &str) -> Result<(i64, Option<i64>, String)> {
    let service_regex =
        regex::Regex::new("^\\s*(?<pid>\\d+)\\s+(?<status>[0-9-]+|(?:[(]\\w+[)]))\\s+(?<service>\\S+)").unwrap();
    let caps = match service_regex.captures(line) {
        Some(caps) => caps,
        None => {
            return Err(Error::ParseError(format!("regex {:#?} does not match: {:#?}", service_regex.to_string(), line)));
        }
    };

    let pid_s = caps.name("pid").expect("pid").as_str().to_string();
    let status_s = caps.name("status").expect("status").as_str().to_string();
    let service = caps.name("service").expect("service").as_str().to_string();
    let pid = i64::from_str_radix(pid_s.as_str(), 10).unwrap_or_default();
    let status = i64::from_str_radix(status_s.as_str(), 10).ok();
    Ok((pid, status, service))
}

pub fn extract_disabled_service_info(line: &str) -> Option<(String, bool)> {
    let service_regex =
        regex::Regex::new("^\\s+\"(?<service>[^\"]+)\".*?(?<enabled>disabled|enabled)")
            .unwrap();
    let caps = service_regex.captures(line)?;

    let service = caps.name("service")?.as_str().to_string();
    let enabled = caps.name("enabled").map(|h|h.as_str().to_string())?.trim() == "enabled";
    Some((service, enabled))
}
