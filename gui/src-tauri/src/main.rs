use ipc_protocol;
use plugin_manager::{LogLevel, PluginManager};
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;

mod manifests;
use manifests::load_plugin_manifest;
use manifests::PluginManifest;

// static PLUGIN_DIR: &str = "../../plugins";
static PLUGIN_DIR: &str = "../../target/debug";
static PLUGIN_MANIFEST_DIR: &str = "../../.config/griffon";

struct PMState(pub Mutex<PluginManager>);

#[derive(Serialize)]
struct PluginInfo {
    pid: u32,
    name: String,
    functions: Vec<String>,
}

#[tauri::command]
fn list_plugins_cmd(pm: State<PMState>) -> Vec<PluginInfo> {
    let plugins = pm.0.lock().unwrap().list_plugins();
    plugins
        .into_iter()
        .map(|p| PluginInfo {
            pid: p.pid,
            name: p.name.clone(),
            functions: p.functions.clone(),
        })
        .collect()
}

#[tauri::command]
fn list_plugins(pm: State<PMState>) -> Vec<String> {
    let pm = pm.0.lock().unwrap();
    pm.list_plugins()
        .into_iter()
        .map(|p| format!("{}: {}", p.pid, p.name))
        .collect()
}

#[tauri::command]
fn refresh_plugins(pm: State<PMState>) {
    pm.0.lock().unwrap().scan_dir();
}

#[tauri::command]
fn message_plugin(pid: u32, msg: String, pm: State<PMState>) {
    let args = Vec::new(); // TODO: Handle param
    let call_payload = ipc_protocol::ipc_payload::CallPayload { fn_name: msg, args };
    match pm.0.lock().unwrap().send_call(pid, call_payload) {
        Ok(req_id) => {
            println!("[GUI] CALL sent (request_id={req_id})");
            match pm.0.lock().unwrap().wait_for_response(req_id) {
                Ok(ev) => println!("[GUI] RESPONSE: {:?}", ev),
                Err(e) => eprintln!("[GUI](ERROR) wait_for_response failed: {e}"),
            }
        }
        Err(e) => println!("[GUI](ERROR) Failed to send CALL: {e}"),
    };
}

//utils format name to folder name
// Test Name2 -> test_name2
fn format_name(name: &str) -> String {
    name.replace(' ', "_").to_lowercase()
}

#[tauri::command]
fn get_plugin_manifest(pid: u32, pm: State<PMState>) -> Result<PluginManifest, String> {
    let plugins = pm.0.lock().unwrap().list_plugins();
    let plugin_name = plugins.into_iter().find(|p| p.pid == pid).unwrap().name;
    let plugin_name = format_name(&plugin_name);
    let path = format!("{PLUGIN_MANIFEST_DIR}/{plugin_name}/{plugin_name}.toml");
    load_plugin_manifest(&path).map_err(|e| e.to_string())
}

fn main() {
    let mut pm = PluginManager::new(PLUGIN_DIR, LogLevel::Info);
    pm.scan_dir();

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(PMState(Mutex::new(pm)))
        .invoke_handler(tauri::generate_handler![
            list_plugins,
            refresh_plugins,
            message_plugin,
            list_plugins_cmd,
            get_plugin_manifest
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
