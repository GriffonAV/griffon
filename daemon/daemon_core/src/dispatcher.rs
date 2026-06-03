use notify_rust::Notification;
use std::sync::mpsc;
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
        let notification_config = NotificationConfig::load();

        while let Ok(task) = task_rx.recv() {
            match task {
                DaemonTask::SwitchStatusPlugin {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                } => {
                    LOGGER_DISPATCHER.debug(format!(
                        "Switch status plugin {}",
                        format_uuid_bytes(&plugin_uuid)
                    ));

                    let response = match pm.switch_status_plugin(plugin_uuid) {
                        Ok(enable) => InterfaceResponse::SwitchDone { request_id, enable },
                        Err(e) => InterfaceResponse::Error {
                            request_id,
                            message: format!(
                                "Failed to switch plugin {}: {e}",
                                format_uuid_bytes(&plugin_uuid)
                            ),
                        },
                    };

                    if let Err(e) = reply_tx.send(response) {
                        LOGGER_DISPATCHER.error(format!(
                            "Failed to send start response to client thread: {e}"
                        ));
                    }
                }

                DaemonTask::RefreshPlugins {
                    request_id,
                    reply_tx,
                } => {
                    LOGGER_DISPATCHER.debug("Refreshing plugins");
                    pm.scan_dir();
                    let plugins = pm.list_plugins();
                    let response = InterfaceResponse::Plugins {
                        request_id,
                        plugins,
                    };

                    if let Err(e) = reply_tx.send(response) {
                        LOGGER_DISPATCHER.error(format!(
                            "Failed to send refresh response to client thread: {e}"
                        ));
                    }
                }

                DaemonTask::CallPlugin {
                    request_id,
                    plugin_uuid,
                    fn_name,
                    args,
                    reply_tx,
                } => {
                    LOGGER_DISPATCHER.debug(format!(
                        "Executing plugin {} function {} arg {:?}",
                        format_uuid_bytes(&plugin_uuid),
                        fn_name,
                        args
                    ));

                    let function_name = fn_name.clone();
                    let plugin_name = pm.get_plugin_name(plugin_uuid).unwrap_or_else(|| format_uuid_bytes(&plugin_uuid));
                    let call = CallPayload { fn_name, args };

                    let response = match pm.send_call(plugin_uuid, call) {
                        Ok(plugin_request_id) => match pm.wait_for_response(plugin_request_id) {
                            Ok(plugin_event) => match plugin_event {
                                plugin_manager::PluginEvent::Result { ok, output, .. } => {
                                    if let Err(e) = send_plugin_response_notification(
                                        plugin_uuid,
                                        &plugin_name,
                                        &function_name,
                                        ok,
                                    ) {
                                        LOGGER_DISPATCHER.error(format!("Failed to send notification: {e}"));
                                    }

                                    InterfaceResponse::CallResult {
                                        request_id,
                                        ok,
                                        output,
                                    }
                                }
                                plugin_manager::PluginEvent::Error { message, .. } => {
                                    if let Err(e) = send_plugin_response_notification(
                                        plugin_uuid,
                                        &plugin_name,
                                        &function_name,
                                        false,
                                    ) {
                                        LOGGER_DISPATCHER.error(format!("Failed to send notification: {e}"));
                                    }
                                    InterfaceResponse::CallResult {
                                        request_id,
                                        ok: false,
                                        output: message,
                                    }
                                }
                                plugin_manager::PluginEvent::Closed { reason, .. } => {
                                    InterfaceResponse::Error {
                                        request_id,
                                        message: format!("Plugin closed: {reason}"),
                                    }
                                }
                                _ => InterfaceResponse::Error {
                                    request_id,
                                    message: "Unexpected message type while waiting for response"
                                        .to_string(),
                                },
                            },
                            Err(e) => InterfaceResponse::CallResult {
                                request_id,
                                ok: false,
                                output: format!("wait_for_response failed: {e}"),
                            },
                        },
                        Err(e) => {
                            LOGGER_DISPATCHER.error(format!("Send call failed: {}", e));
                            InterfaceResponse::Error {
                                request_id,
                                message: format!("send_call failed: {e}"),
                            }
                        }
                    };
                    LOGGER_DISPATCHER
                        .debug(format!("Sending response to client thread: {response:?}"));
                    if let Err(e) = reply_tx.send(response) {
                        LOGGER_DISPATCHER
                            .error(format!("Failed to send response to client thread: {e}"));
                    }
                }
            }
        }
    });
}
