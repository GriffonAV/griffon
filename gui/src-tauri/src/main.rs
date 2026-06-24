use ipc_protocol::ipc_payload_interface::{
    alloc_request_id, format_uuid_bytes, parse_uuid_16, recv_interface_response,
    send_interface_request, InterfaceRequest, InterfaceResponse,
};
use serde::Serialize;
use std::os::unix::net::UnixStream;
use std::path::Path;
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
}

const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    // "/tmp/griffon-dev.sock"
    // Use a fixed path in the Griffon directory to avoid issues with some IDEs that create random temp directories
    "../../griffon.sock"
} else {
    "/run/griffon/griffon.sock"
};

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

fn start_reader_thread(mut read_sock: UnixStream, app_handle: tauri::AppHandle) {
    thread::spawn(move || {
        LOGGER_NETWORK.debug("Reader thread started");

        loop {
            match recv_interface_response(&mut read_sock) {
                Ok(resp) => match resp {
                    InterfaceResponse::SwitchDone { request_id, enable } => {
                        LOGGER_NETWORK.info(format!(
                            "SwitchDone received {} enable:{}",
                            request_id, enable
                        ));

                        let _ = app_handle.emit(
                            "plugin-switch-done",
                            serde_json::json!({
                                "request_id": request_id,
                                "enable": enable
                            }),
                        );
                    }
                    InterfaceResponse::Ok { request_id } => {
                        LOGGER_NETWORK.info(format!("Ok received {}", request_id));
                        // let _ = app_handle.emit(
                        //     "daemon-ok",
                        //     serde_json::json!({
                        //         "request_id": request_id
                        //     }),
                        // ); I comment it because annoying
                    }
                    InterfaceResponse::CallAccepted { request_id } => {
                        LOGGER_NETWORK.info(format!(
                            "Call accepted received for request_id={request_id}"
                        ));
                    }
                    InterfaceResponse::Plugins {
                        request_id,
                        plugins,
                    } => {
                        LOGGER_NETWORK.info(format!(
                            "Plugins list received: {} plugin(s) from request_id={request_id}",
                            plugins.len()
                        ));
                        for plugin in plugins {
                            println!(
                                "- UUID: {:?} | NAME: {} | PATH: {} | FUNCTIONS: {:?} | STATUS: {}",
                                format_uuid_bytes(&plugin.plugin_uuid),
                                plugin.name,
                                plugin.path,
                                plugin.functions,
                                plugin.status
                            );
                        }
                        let _ = app_handle.emit("plugins-updated", ());
                    }
                    InterfaceResponse::Error {
                        request_id,
                        message,
                    } => {
                        LOGGER_NETWORK.error(format!("Request {request_id} error={message}"));
                    }
                    InterfaceResponse::CallResult {
                        request_id,
                        ok,
                        output,
                    } => {
                        LOGGER_NETWORK
                            .info(format!("Call {request_id} result={ok} output={output}"));
                        let _ = app_handle.emit(
                            "plugin-call-result",
                            serde_json::json!({
                                "request_id": request_id,
                                "ok": ok,
                                "output": output
                            }),
                        );
                    }
                },
                Err(e) => {
                    LOGGER_NETWORK.error(format!(
                        "Reader thread stopped, failed to receive response: {e}"
                    ));
                    break;
                }
            }
        }
    });
} // TMP FROM CLI TO DEBUG

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
                move || {
                    thread::sleep(std::time::Duration::from_millis(500));

                    // print DAEMON_SOCK_PATH for debug
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
                                LOGGER_NETWORK.debug(format!("socket: {:?}", sock_guard));
                            }
                            let _ = app_handle.emit("daemon-status", "Connected");

                            start_reader_thread(stream.try_clone().unwrap(), app_handle.clone());

                            // TMP FOR TEST
                            // if let Err(e) = refresh_plugin_inner(&state) {
                            //     LOGGER_NETWORK.error(format!("Failed to refresh plugins: {e}"));
                            // }
                            // let vec_empty = Vec::new();
                            // if let Err(e) = call_plugin_inner(
                            //     &state,
                            //     "6e9e800a-0d0c-4f74-8265-7b9ab0234582",
                            //     "ping",
                            //     vec_empty,
                            // ) {
                            //     LOGGER_NETWORK
                            //         .error(format!("Failed to switch status plugins: {e}"));
                            // }
                            // if let Err(e) = switch_status_plugin_inner(
                            //     &state,
                            //     "6e9e800a-0d0c-4f74-8265-7b9ab0234582",
                            // ) {
                            //     LOGGER_NETWORK
                            //         .error(format!("Failed to switch status plugins: {e}"));
                            // }
                            // END OF TMP FOR TEST
                        }
                        Err(e) => {
                            LOGGER_NETWORK.error(format!("Failed to connect: {}", e));
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
            delete_plugin,
            get_plugin_manifest,
            refresh_plugin,
            switch_status_plugin,
            call_plugin,
            plugin_history::get_plugin_history,
            plugin_installer::install_plugin_files,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
