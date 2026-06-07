use std::sync::mpsc::{self, Sender};
use std::thread;

use ipc_protocol::ipc_payload_interface::{InterfaceResponse, format_uuid_bytes};
use ipc_protocol::ipc_payload_runner::CallPayload;
use logger::Logger;

use crate::notification::{NotificationConfig, send_plugin_response_notification};
use crate::types::DaemonTask;

static LOGGER_DISPATCHER: Logger = if cfg!(debug_assertions) {
    Logger::new("DISPATCHER", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "DISPATCHER",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

pub fn start_dispatcher(task_rx: mpsc::Receiver<DaemonTask>, plugin_dir_path: &'static str) {
    thread::spawn(move || {
        LOGGER_DISPATCHER.debug("Started");

        let mut pm = plugin_manager::PluginManager::new(plugin_dir_path);
        pm.scan_dir();

        while let Ok(task) = task_rx.recv() {
            match task {
                DaemonTask::SwitchStatusNotification {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                } => {
                    let response = handle_switch_notification(request_id, plugin_uuid);
                    send_response(reply_tx, response, "notification switch");
                }

                DaemonTask::SwitchStatusPlugin {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                } => {
                    let response = handle_switch_plugin(&mut pm, request_id, plugin_uuid);
                    send_response(reply_tx, response, "plugin switch");
                }

                DaemonTask::RefreshPlugins {
                    request_id,
                    reply_tx,
                } => {
                    let response = handle_refresh_plugins(&mut pm, request_id);
                    send_response(reply_tx, response, "plugin refresh");
                }

                DaemonTask::CallPlugin {
                    request_id,
                    plugin_uuid,
                    fn_name,
                    args,
                    reply_tx,
                } => {
                    let function_name = fn_name.clone();
                    let call = CallPayload { fn_name, args };

                    let response =
                        handle_call_plugin(&mut pm, request_id, plugin_uuid, function_name, call);

                    send_response(reply_tx, response, "plugin call");
                }
            }
        }
    });
}

fn handle_switch_notification(request_id: u32, plugin_uuid: [u8; 16]) -> InterfaceResponse {
    LOGGER_DISPATCHER.debug(format!(
        "Switching notification status for plugin {}",
        format_uuid_bytes(&plugin_uuid)
    ));

    match NotificationConfig::switch_plugin_notification(plugin_uuid) {
        Ok(enable) => InterfaceResponse::SwitchDone { request_id, enable },
        Err(e) => InterfaceResponse::Error {
            request_id,
            message: format!(
                "Failed to switch notification status for plugin {}: {e}",
                format_uuid_bytes(&plugin_uuid)
            ),
        },
    }
}

fn handle_switch_plugin(
    pm: &mut plugin_manager::PluginManager,
    request_id: u32,
    plugin_uuid: [u8; 16],
) -> InterfaceResponse {
    LOGGER_DISPATCHER.debug(format!(
        "Switch status plugin {}",
        format_uuid_bytes(&plugin_uuid)
    ));

    match pm.switch_status_plugin(plugin_uuid) {
        Ok(enable) => InterfaceResponse::SwitchDone { request_id, enable },
        Err(e) => InterfaceResponse::Error {
            request_id,
            message: format!(
                "Failed to switch plugin {}: {e}",
                format_uuid_bytes(&plugin_uuid)
            ),
        },
    }
}

fn handle_refresh_plugins(
    pm: &mut plugin_manager::PluginManager,
    request_id: u32,
) -> InterfaceResponse {
    LOGGER_DISPATCHER.debug("Refreshing plugins");

    pm.scan_dir();

    InterfaceResponse::Plugins {
        request_id,
        plugins: pm.list_plugins(),
    }
}

fn handle_call_plugin(
    pm: &mut plugin_manager::PluginManager,
    request_id: u32,
    plugin_uuid: [u8; 16],
    function_name: String,
    call: CallPayload,
) -> InterfaceResponse {
    LOGGER_DISPATCHER.debug(format!(
        "Executing plugin {} function {} arg {:?}",
        format_uuid_bytes(&plugin_uuid),
        function_name,
        call.args
    ));

    let plugin_name = pm
        .get_plugin_name(plugin_uuid)
        .unwrap_or_else(|| format_uuid_bytes(&plugin_uuid));

    let plugin_request_id = match pm.send_call(plugin_uuid, call) {
        Ok(plugin_request_id) => plugin_request_id,
        Err(e) => {
            LOGGER_DISPATCHER.error(format!("Send call failed: {e}"));

            return InterfaceResponse::Error {
                request_id,
                message: format!("send_call failed: {e}"),
            };
        }
    };

    let plugin_event = match pm.wait_for_response(plugin_request_id) {
        Ok(plugin_event) => plugin_event,
        Err(e) => {
            return InterfaceResponse::CallResult {
                request_id,
                ok: false,
                output: format!("wait_for_response failed: {e}"),
            };
        }
    };

    handle_plugin_event_response(
        request_id,
        plugin_uuid,
        &plugin_name,
        &function_name,
        plugin_event,
    )
}

fn handle_plugin_event_response(
    request_id: u32,
    plugin_uuid: [u8; 16],
    plugin_name: &str,
    function_name: &str,
    plugin_event: plugin_manager::PluginEvent,
) -> InterfaceResponse {
    match plugin_event {
        plugin_manager::PluginEvent::Result { ok, output, .. } => {
            send_plugin_notification(plugin_uuid, plugin_name, function_name, ok);

            InterfaceResponse::CallResult {
                request_id,
                ok,
                output,
            }
        }

        plugin_manager::PluginEvent::Error { message, .. } => {
            send_plugin_notification(plugin_uuid, plugin_name, function_name, false);

            InterfaceResponse::CallResult {
                request_id,
                ok: false,
                output: message,
            }
        }

        plugin_manager::PluginEvent::Closed { reason, .. } => InterfaceResponse::Error {
            request_id,
            message: format!("Plugin closed: {reason}"),
        },

        _ => InterfaceResponse::Error {
            request_id,
            message: "Unexpected message type while waiting for response".to_string(),
        },
    }
}

fn send_plugin_notification(
    plugin_uuid: [u8; 16],
    plugin_name: &str,
    function_name: &str,
    ok: bool,
) {
    if let Err(e) = send_plugin_response_notification(plugin_uuid, plugin_name, function_name, ok) {
        LOGGER_DISPATCHER.error(format!("Failed to send notification: {e}"));
    }
}

fn send_response(reply_tx: Sender<InterfaceResponse>, response: InterfaceResponse, context: &str) {
    LOGGER_DISPATCHER.debug(format!("Sending {context} response: {response:?}"));

    if let Err(e) = reply_tx.send(response) {
        LOGGER_DISPATCHER.error(format!(
            "Failed to send {context} response to client thread: {e}"
        ));
    }
}
