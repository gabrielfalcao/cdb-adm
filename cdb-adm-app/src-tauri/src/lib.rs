use cdb_adm::{list_all_agents_and_daemons, Uid};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn list_agents_and_daemons() -> Vec<Vec<String>> {
    let uid = Uid::from(iocore::User::id().unwrap().uid);
    let mut ads = list_all_agents_and_daemons(&uid)
        .unwrap()
        .iter()
        .map(|(domain, service, pid, status, enabled, info)| {
            vec![
                service.to_string(),
                pid.to_string(),
                domain.to_string(),
                status.map(|h| h.to_string()).unwrap_or_else(|| "-".to_string()),
                if *enabled { "enabled" } else { "disabled" }.to_string(),
                info.clone().map(|(path, _)| path.to_string()).unwrap_or_default(),
            ]
        })
        .collect::<Vec<Vec<String>>>();
    ads.sort_by_key(|service| service[0].to_string());
    ads
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, list_agents_and_daemons])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
