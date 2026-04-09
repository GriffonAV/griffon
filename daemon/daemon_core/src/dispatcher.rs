use std::sync::mpsc;
use std::thread;

use ipc_protocol::ipc_payload_interface::{InterfaceResponse, format_uuid_bytes};
use ipc_protocol::ipc_payload_runner::CallPayload;
use logger::Logger;

use crate::types::DaemonTask;

static LOGGER_DISPATCHER: Logger = Logger::new(
    "DISPATCHER",
    logger::LogLevel::Debug,
    Some("/var/log/griffon/griffon-daemon.log"),
);

pub fn start_dispatcher(task_rx: mpsc::Receiver<DaemonTask>, plugin_dir_path: &'static str) {
    thread::spawn(move || {
        LOGGER_DISPATCHER.debug("Started");

        let mut pm = plugin_manager::PluginManager::new(plugin_dir_path);
        pm.scan_dir();

        while let Ok(task) = task_rx.recv() {
            match task {
                DaemonTask::StartPlugin {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                } => {
                    LOGGER_DISPATCHER
                        .debug(format!("Start plugin {}", format_uuid_bytes(&plugin_uuid)));

                    let response = match pm.enable_plugin(plugin_uuid) {
                        Ok(_) => InterfaceResponse::Ok,
                        Err(e) => InterfaceResponse::Error {
                            request_id,
                            message: format!(
                                "Failed to start plugin {}: {e}",
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
                DaemonTask::StopPlugin {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                } => {
                    LOGGER_DISPATCHER
                        .debug(format!("Stop plugin {}", format_uuid_bytes(&plugin_uuid)));

                    let response = match pm.disable_plugin(plugin_uuid) {
                        Ok(_) => InterfaceResponse::Ok,
                        Err(e) => InterfaceResponse::Error {
                            request_id,
                            message: format!(
                                "Failed to stop plugin {}: {e}",
                                format_uuid_bytes(&plugin_uuid)
                            ),
                        },
                    };

                    if let Err(e) = reply_tx.send(response) {
                        LOGGER_DISPATCHER.error(format!(
                            "Failed to send stop response to client thread: {e}"
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

                    let call = CallPayload { fn_name, args };

                    let response = match pm.send_call(plugin_uuid, call) {
                        Ok(plugin_request_id) => match pm.wait_for_response(plugin_request_id) {
                            Ok(plugin_event) => match plugin_event {
                                plugin_manager::PluginEvent::Result { ok, output, .. } => {
                                    InterfaceResponse::CallResult {
                                        request_id,
                                        ok,
                                        output,
                                    }
                                }
                                plugin_manager::PluginEvent::Error { message, .. } => {
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
