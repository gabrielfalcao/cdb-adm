use crate::{Error, Result};

pub fn parse_services(data: &str) -> Result<Vec<(i64, Option<i64>, String)>> {
    let mut services = Vec::new();
    let mut in_services = false;
    for line in data.lines() {
        if !in_services && line.trim_start().starts_with("services = ") {
            in_services = true;
        } else if in_services {
            if line.trim() == "}" {
                break;
            } else {
                services.push(extract_service_info_opt(line).ok_or_else(|| {
                    Error::ParseError(format!("service info not found in {:#?}", line))
                })?)
            }
        }
    }
    Ok(services)
}

pub fn extract_service_name(line: &str) -> Result<String> {
    let (_, _, service) = extract_service_info_opt(line)
        .ok_or_else(|| Error::ParseError(format!("service name not found in {:#?}", line)))?;
    Ok(service)
}

pub fn extract_service_info_opt(line: &str) -> Option<(i64, Option<i64>, String)> {
    let service_regex =
        regex::Regex::new(r"^\s+(?<pid>\d+)\s+(?<status>[0-9-]+)\s+(?<service>\S+)").unwrap();
    let caps = service_regex.captures(line)?;

    let pid_s = caps.name("pid")?.as_str().to_string();
    let status_s = caps.name("status")?.as_str().to_string();
    let service = caps.name("service")?.as_str().to_string();
    let pid = i64::from_str_radix(pid_s.as_str(), 10).unwrap();
    let status = i64::from_str_radix(status_s.as_str(), 10).ok();
    Some((pid, status, service))
}
