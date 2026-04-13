use serde::Serialize;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use tauri::Emitter;
use tauri::Manager;
mod daemon;
use crate::daemon::{get_daemon_status, DaemonConnection};

mod manifests;
use manifests::load_plugin_manifest;
use manifests::PluginManifest;

const PLUGIN_MANIFEST_DIR: &str = if cfg!(debug_assertions) {
    "../../.config/griffon"
} else {
    "/usr/lib/griffonav/plugins"
};

#[derive(Serialize)]
struct Plugin {
    pid: u32,
    name: String,
}

const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    "/tmp/griffon-dev.sock"
} else {
    "/run/griffon/griffon.sock"
};

fn format_name(name: &str) -> String {
    name.replace(' ', "_").to_lowercase()
}

#[tauri::command]
fn get_plugin_manifest(name: String) -> Result<PluginManifest, String> {
    println!("Loading plugin manifest of: {}", name);
    let name = format_name(&name);
    let path = format!("{PLUGIN_MANIFEST_DIR}/{name}.toml");
    load_plugin_manifest(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_plugins() -> Result<Vec<Plugin>, String> {
    println!("Listing plugins from directory: {}", PLUGIN_MANIFEST_DIR);
    let entries = std::fs::read_dir(PLUGIN_MANIFEST_DIR).map_err(|e| e.to_string())?;
    let mut plugins = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("toml") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                plugins.push(Plugin {
                    pid: 0, // populate if you have a real pid
                    name: stem.to_string(),
                });
            }
        }
    }
    println!(
        "Found plugins: {:?}",
        plugins.iter().map(|p| &p.name).collect::<Vec<_>>()
    );
    plugins.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(plugins)
}
fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(DaemonConnection(Mutex::new(None)))
        .setup(|app| {
            let app_handle = app.handle().clone();

            std::thread::spawn({
                let app_handle = app_handle.clone();
                move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));

                    match UnixStream::connect(DAEMON_SOCK_PATH) {
                        Ok(stream) => {
                            println!("Successfully connected to Griffon Daemon");

                            if let Ok(mut guard) = app_handle.state::<DaemonConnection>().0.lock() {
                                *guard = Some(stream.try_clone().expect("Failed to clone socket"));
                            }
                        }
                        Err(e) => {
                            println!("Failed to connect: {}", e);
                            let _ = app_handle.emit("daemon-status", "Disconnected");
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_daemon_status,
            list_plugins,
            get_plugin_manifest
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
