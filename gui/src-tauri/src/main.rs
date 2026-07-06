use ipc_protocol::ipc_payload_interface::{
    alloc_request_id, parse_uuid_16, recv_interface_response, send_interface_request,
    InterfaceRequest, InterfaceResponse,
};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use tauri::Manager;
use tauri::{Emitter, State};

mod daemon;
use crate::daemon::{get_daemon_status, DaemonConnection};
use logger::{LogLevel, Logger};

mod manifests;
mod plugin_history;
mod plugin_installer;

use manifests::load_plugin_manifest;
use manifests::PluginManifest;

static LOGGER: Logger = Logger::new("GUI", LogLevel::Debug, None);
static LOGGER_NETWORK: Logger = Logger::new("GUI-NETWORK", LogLevel::Debug, None);

const PLUGIN_MANIFEST_DIR: &str = if cfg!(debug_assertions) {
    "../../.config/griffon"
} else {
    "/usr/lib/griffon/plugins"
};

#[derive(Serialize)]
struct Plugin {
    file_name: String,
    uuid: String,
    display_name: String,
    version: String,
    author: String,
    description: String,
    notifications_enabled: bool,
}

const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    // "/tmp/griffon-dev.sock"
    // Use a fixed path in the Griffon directory to avoid issues with some IDEs that create random temp directories
    "../../griffon.sock"
} else {
    "/run/griffon/griffon.sock"
};

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize, Default)]
struct GriffonDaemonConfig {
    #[serde(default)]
    general: GriffonGeneralConfig,

    #[serde(default)]
    plugins: HashMap<String, GriffonPluginConfig>,
}

#[derive(Debug, Deserialize)]
struct GriffonGeneralConfig {
    #[serde(default = "default_true")]
    notifications_enabled: bool,
}

impl Default for GriffonGeneralConfig {
    fn default() -> Self {
        Self {
            notifications_enabled: true,
        }
    }
}

#[derive(Debug, Deserialize)]
struct GriffonPluginConfig {
    #[serde(default = "default_true")]
    notifications_enabled: bool,
}

fn daemon_config_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("../../daemon/config_griffon_daemon.json")
    } else {
        PathBuf::from("/etc/griffon/config_griffon_daemon.json")
    }
}

fn load_daemon_config() -> GriffonDaemonConfig {
    let path = daemon_config_path();

    match std::fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<GriffonDaemonConfig>(&content) {
            Ok(config) => config,
            Err(e) => {
                LOGGER.error(format!(
                    "Failed to parse daemon config '{}': {}",
                    path.display(),
                    e
                ));
                GriffonDaemonConfig::default()
            }
        },
        Err(e) => {
            LOGGER.error(format!(
                "Failed to read daemon config '{}': {}",
                path.display(),
                e
            ));
            GriffonDaemonConfig::default()
        }
    }
}

fn plugin_notifications_enabled(config: &GriffonDaemonConfig, plugin_uuid: &str) -> bool {
    let plugin_enabled = config
        .plugins
        .get(plugin_uuid)
        .map(|plugin| plugin.notifications_enabled)
        .unwrap_or(config.general.notifications_enabled);

    config.general.notifications_enabled && plugin_enabled
}

fn format_name(name: &str) -> String {
    name.replace(' ', "_").to_lowercase()
}

fn sanitize_plugin_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err("Plugin name cannot be empty".to_string());
    }

    if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains("..") {
        return Err("Invalid plugin name".to_string());
    }

    let stem = Path::new(trimmed)
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid plugin name".to_string())?;

    let formatted_stem = format_name(stem);

    if formatted_stem.is_empty() {
        return Err("Plugin name cannot be empty".to_string());
    }

    if !formatted_stem
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("Invalid plugin name".to_string());
    }

    Ok(formatted_stem)
}

#[tauri::command]
fn delete_plugin(name: String) -> Result<(), String> {
    let plugin_name = sanitize_plugin_name(&name)?;

    LOGGER.info(format!("Deleting plugin: {}", plugin_name));

    let toml_path = Path::new(PLUGIN_MANIFEST_DIR).join(format!("{}.toml", plugin_name));
    let so_path = Path::new(PLUGIN_MANIFEST_DIR).join(format!("{}.so", plugin_name));

    let mut deleted_files = Vec::new();

    if toml_path.exists() {
        std::fs::remove_file(&toml_path).map_err(|e| {
            format!(
                "Failed to delete plugin manifest '{}': {}",
                toml_path.display(),
                e
            )
        })?;

        deleted_files.push(toml_path.display().to_string());
    }

    if so_path.exists() {
        std::fs::remove_file(&so_path).map_err(|e| {
            format!(
                "Failed to delete plugin shared library '{}': {}",
                so_path.display(),
                e
            )
        })?;

        deleted_files.push(so_path.display().to_string());
    }

    if deleted_files.is_empty() {
        return Err(format!("Plugin '{}' was not found", plugin_name));
    }

    LOGGER.info(format!(
        "Deleted plugin '{}' files: {:?}",
        plugin_name, deleted_files
    ));

    Ok(())
}

#[tauri::command]
fn get_plugin_manifest(name: String) -> Result<PluginManifest, String> {
    LOGGER.info(format!("Loading plugin manifest of: {}", name));
    let name = format_name(&name);
    let path = format!("{PLUGIN_MANIFEST_DIR}/{name}.toml");
    load_plugin_manifest(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_plugins() -> Result<Vec<Plugin>, String> {
    LOGGER.debug(format!(
        "Listing plugins from directory: {}",
        PLUGIN_MANIFEST_DIR
    ));

    let entries = std::fs::read_dir(PLUGIN_MANIFEST_DIR).map_err(|e| e.to_string())?;
    let mut plugins = Vec::new();

    let daemon_config = load_daemon_config();

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            continue;
        }

        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };

        let Some(path_str) = path.to_str() else {
            LOGGER.error(format!("Invalid manifest path: {:?}", path));
            continue;
        };

        let manifest = match load_plugin_manifest(path_str) {
            Ok(manifest) => manifest,
            Err(e) => {
                LOGGER.error(format!(
                    "Failed to load plugin manifest {}: {}",
                    path.display(),
                    e
                ));
                continue;
            }
        };

        plugins.push(Plugin {
            file_name: stem.to_string(),
            uuid: manifest.plugin.uuid.clone(),
            display_name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            author: manifest.plugin.author.clone(),
            description: manifest.plugin.description.clone(),
            notifications_enabled: plugin_notifications_enabled(
                &daemon_config,
                &manifest.plugin.uuid,
            ),
        });
    }

    LOGGER.debug(format!(
        "Found plugins: {:?}",
        plugins
            .iter()
            .map(|p| format!("{} ({})", p.display_name, p.file_name))
            .collect::<Vec<_>>()
    ));

    plugins.sort_by(|a, b| a.display_name.cmp(&b.display_name));

    Ok(plugins)
}
#[tauri::command]
fn call_plugin(
    state: State<'_, DaemonConnection>,
    plugin_uuid: String,
    fn_name: String,
    args: Vec<String>,
    request_id: u32,
) -> Result<(), String> {
    call_plugin_inner(&state, &plugin_uuid, &fn_name, args, request_id)
}

fn call_plugin_inner(
    conn: &DaemonConnection,
    plugin_uuid_str: &str,
    fn_name: &str,
    args: Vec<String>,
    request_id: u32,
) -> Result<(), String> {
    LOGGER.debug("CALL Sent");

    let mut sock_guard = conn.0.lock().map_err(|e| e.to_string())?;

    let mut sock = sock_guard
        .as_mut()
        .ok_or_else(|| "Daemon not connected".to_string())?;

    let plugin_uuid = match parse_uuid_16(Some(plugin_uuid_str)) {
        Some(uuid) => uuid,
        None => {
            LOGGER.error("Invalid UUID format");
            return Err("Invalid UUID".to_string());
        }
    };

    if fn_name.trim().is_empty() {
        LOGGER.error("Function name cannot be empty");
        return Err("Function name cannot be empty".to_string());
    }

    send_interface_request(
        &mut sock,
        &InterfaceRequest::CallPlugin {
            plugin_uuid,
            fn_name: fn_name.to_string(),
            args,
        },
        request_id,
    )
    .map_err(|e| e.to_string())?;

    LOGGER_NETWORK.debug(format!("Call request sent with request_id={request_id}"));

    Ok(())
}

#[tauri::command]
fn switch_status_plugin(
    state: State<'_, DaemonConnection>,
    plugin_uuid: String,
    request_id: u32,
) -> Result<(), String> {
    switch_status_plugin_inner(&state, &plugin_uuid, request_id)
}

fn switch_status_plugin_inner(
    conn: &DaemonConnection,
    plugin_uuid_str: &str,
    request_id: u32,
) -> Result<(), String> {
    LOGGER.debug("SWITCH STATUS");

    let mut sock_guard = conn.0.lock().map_err(|e| e.to_string())?;

    let mut sock = sock_guard
        .as_mut()
        .ok_or_else(|| "Daemon not connected".to_string())?;

    let plugin_uuid = parse_uuid_16(Some(plugin_uuid_str)).ok_or_else(|| {
        LOGGER.error("Invalid UUID format");
        "Invalid UUID".to_string()
    })?;

    send_interface_request(
        &mut sock,
        &InterfaceRequest::SwitchStatusPlugin { plugin_uuid },
        request_id,
    )
    .map_err(|e| e.to_string())?;

    LOGGER_NETWORK.debug(format!(
        "Switch status plugins sent with request_id={request_id}"
    ));

    Ok(())
}

#[tauri::command]
fn switch_status_notification(
    state: State<'_, DaemonConnection>,
    plugin_uuid: String,
    request_id: u32,
) -> Result<(), String> {
    switch_status_notification_inner(&state, &plugin_uuid, request_id)
}

fn switch_status_notification_inner(
    conn: &DaemonConnection,
    plugin_uuid_str: &str,
    request_id: u32,
) -> Result<(), String> {
    LOGGER.debug("SWITCH NOTIFICATION STATUS");

    let mut sock_guard = conn.0.lock().map_err(|e| e.to_string())?;

    let mut sock = sock_guard
        .as_mut()
        .ok_or_else(|| "Daemon not connected".to_string())?;

    let plugin_uuid = parse_uuid_16(Some(plugin_uuid_str)).ok_or_else(|| {
        LOGGER.error("Invalid UUID format");
        "Invalid UUID".to_string()
    })?;

    send_interface_request(
        &mut sock,
        &InterfaceRequest::SwitchStatusNotification { plugin_uuid },
        request_id,
    )
    .map_err(|e| e.to_string())?;

    LOGGER_NETWORK.debug(format!(
        "Switch notification status sent with request_id={request_id}"
    ));

    Ok(())
}

#[tauri::command]
fn refresh_plugin(state: State<'_, DaemonConnection>) -> Result<(), String> {
    refresh_plugin_inner(&state)
}

fn refresh_plugin_inner(conn: &DaemonConnection) -> Result<(), String> {
    LOGGER.debug("REFRESH");

    let mut sock_guard = conn.0.lock().map_err(|e| e.to_string())?;
    let mut id_guard = conn.1.lock().map_err(|e| e.to_string())?;

    let mut sock = sock_guard
        .as_mut()
        .ok_or_else(|| "Daemon not connected".to_string())?;

    let next_request_id = alloc_request_id(*id_guard);

    send_interface_request(
        &mut sock,
        &InterfaceRequest::RefreshPlugins {},
        next_request_id,
    )
    .map_err(|e| e.to_string())?;

    let _ = list_plugins();
    LOGGER_NETWORK.debug(format!(
        "Refresh plugins sent with request_id={next_request_id}"
    ));

    *id_guard = next_request_id;

    Ok(())
}

struct ReconnectSender(mpsc::Sender<()>);

#[tauri::command]
fn force_reconnect(
    conn: tauri::State<'_, DaemonConnection>,
    sender: tauri::State<'_, ReconnectSender>,
) {
    LOGGER_NETWORK.info("Manual reconnect triggered");

    if let Ok(mut sock_guard) = conn.0.lock() {
        if let Some(sock) = sock_guard.take() {
            let _ = sock.shutdown(std::net::Shutdown::Both);
        }
    }

    let _ = sender.0.send(());
}

fn run_reader_loop(mut read_sock: UnixStream, app_handle: tauri::AppHandle) {
    LOGGER_NETWORK.debug("Reader loop started");

    loop {
        match recv_interface_response(&mut read_sock) {
            Ok(resp) => {
                match resp {
                    // ... Keep all your existing Ok(resp) match arms exactly as they are ...
                    InterfaceResponse::SwitchDone { request_id, enable } => {
                        let _ = app_handle.emit(
                            "plugin-switch-done",
                            serde_json::json!({ "request_id": request_id, "enable": enable }),
                        );
                    }
                    InterfaceResponse::Ok { request_id: _ } => {}
                    InterfaceResponse::CallAccepted { request_id: _ } => {}
                    InterfaceResponse::Plugins {
                        request_id: _,
                        plugins: _,
                    } => {
                        let _ = app_handle.emit("plugins-updated", ());
                    }
                    InterfaceResponse::Error {
                        request_id: _,
                        message: _,
                    } => {}
                    InterfaceResponse::CallResult {
                        request_id,
                        ok,
                        output,
                    } => {
                        let _ = app_handle.emit("plugin-call-result", serde_json::json!({ "request_id": request_id, "ok": ok, "output": output }));
                    }
                }
            }
            Err(e) => {
                let error_msg = format!("Reader thread stopped, failed to receive response: {}", e);
                LOGGER_NETWORK.error(&error_msg);

                // 1. Emit the disconnect event to the frontend
                let _ = app_handle.emit(
                    "daemon-status",
                    serde_json::json!({
                        "status": "Disconnected",
                        "error": error_msg
                    }),
                );

                // 2. Clear the connection from the app state so commands know it's disconnected
                let state = app_handle.state::<DaemonConnection>();
                if let Ok(mut sock_guard) = state.0.lock() {
                    *sock_guard = None;
                }

                // 3. Break the loop to trigger the reconnect sleep
                break;
            }
        }
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .manage(DaemonConnection(Mutex::new(None), Mutex::new(0)))
        .setup(|app| {
            let app_handle = app.handle().clone();

            thread::spawn({
                let app_handle = app_handle.clone();
                move || loop {
                    LOGGER_NETWORK.debug(format!(
                        "Attempting to connect to daemon at: {}",
                        DAEMON_SOCK_PATH
                    ));

                    match UnixStream::connect(DAEMON_SOCK_PATH) {
                        Ok(stream) => {
                            let state = app_handle.state::<DaemonConnection>();

                            {
                                let mut sock_guard = state.0.lock().unwrap();
                                *sock_guard = Some(stream.try_clone().unwrap());
                                LOGGER_NETWORK.info("Successfully connected to Griffon Daemon");
                            }

                            let _ = app_handle.emit(
                                "daemon-status",
                                serde_json::json!({
                                    "status": "Connected"
                                }),
                            );

                            run_reader_loop(stream.try_clone().unwrap(), app_handle.clone());
                        }
                        Err(e) => {
                            let error_msg = format!("Failed to connect: {}", e);
                            LOGGER_NETWORK.error(&error_msg);

                            let _ = app_handle.emit(
                                "daemon-status",
                                serde_json::json!({
                                    "status": "Disconnected",
                                    "error": error_msg
                                }),
                            );
                        }
                    }

                    LOGGER_NETWORK.info("Waiting 10 seconds before reconnecting...");
                    thread::sleep(std::time::Duration::from_secs(10));
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_daemon_status,
            list_plugins,
            delete_plugin,
            get_plugin_manifest,
            refresh_plugin,
            switch_status_plugin,
            switch_status_notification,
            call_plugin,
            plugin_history::get_plugin_history,
            plugin_installer::install_plugin_zip,
            force_reconnect,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
