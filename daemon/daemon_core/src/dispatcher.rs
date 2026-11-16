use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::thread;

use crossbeam_channel::{Receiver, select};

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

struct PendingCall {
    request_id: u32,
    plugin_uuid: [u8; 16],
    plugin_name: String,
    function_name: String,
    reply_tx: Sender<InterfaceResponse>,
}

pub fn start_dispatcher(task_rx: Receiver<DaemonTask>, plugin_dir_path: &'static str) {
    thread::spawn(move || {
        LOGGER_DISPATCHER.debug("Started");

        let mut pm = plugin_manager::PluginManager::new(plugin_dir_path);
        pm.scan_dir();
        let event_rx = pm.event_receiver();

        let mut pending_calls: HashMap<u32, PendingCall> = HashMap::new();

        loop {
            select! {
                recv(task_rx) -> task => match task {
                    Ok(task) => match task {
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

                        let plugin_name = pm
                            .get_plugin_name(plugin_uuid)
                            .unwrap_or_else(|| format_uuid_bytes(&plugin_uuid));

                        match pm.send_call(plugin_uuid, call) {
                            Ok(plugin_request_id) => {
                                LOGGER_DISPATCHER.debug(format!(
                                    "Plugin call sent: client request={}, plugin request={}",
                                    request_id, plugin_request_id,
                                ));

                                pending_calls.insert(
                                    plugin_request_id,
                                    PendingCall {
                                        request_id,
                                        plugin_uuid,
                                        plugin_name,
                                        function_name,
                                        reply_tx,
                                    },
                                );
                            }

                            Err(e) => {
                                LOGGER_DISPATCHER.error(format!("Send call failed: {e}"));

                                send_response(
                                    reply_tx,
                                    InterfaceResponse::Error {
                                        request_id,
                                        message: format!("send_call failed: {e}"),
                                    },
                                    "plugin call",
                                );
                            }
                        }
                    }
                    },
                    Err(_) => {
                        LOGGER_DISPATCHER.warn("Task channel disconnected; stopping dispatcher");
                        break;
                    }
                },
                recv(event_rx) -> event => match event {
                    Ok(event) => handle_plugin_event(event, &mut pending_calls),
                    Err(_) => {
                        LOGGER_DISPATCHER.error("Plugin event channel disconnected; stopping dispatcher");
                        break;
                    }
                },
            }
        }
    });
}

fn handle_plugin_event(
    event: plugin_manager::PluginEvent,
    pending_calls: &mut HashMap<u32, PendingCall>,
) {
    let plugin_request_id = match &event {
        plugin_manager::PluginEvent::Result { request_id, .. }
        | plugin_manager::PluginEvent::Error { request_id, .. } => Some(*request_id),
        _ => None,
    };

    if let Some(plugin_request_id) = plugin_request_id {
        match pending_calls.remove(&plugin_request_id) {
            Some(pending) => {
                let response = handle_plugin_event_response(
                    pending.request_id,
                    pending.plugin_uuid,
                    &pending.plugin_name,
                    &pending.function_name,
                    event,
                );

                send_response(pending.reply_tx, response, "plugin call");
            }
            None => {
                LOGGER_DISPATCHER.warn(format!(
                    "Received an event for unknown plugin request {}",
                    plugin_request_id,
                ));
            }
        }

        return;
    }

    match event {
        plugin_manager::PluginEvent::Closed { pid, reason } => {
            LOGGER_DISPATCHER.error(format!("Plugin runner {pid} closed: {reason}",));
        }

        plugin_manager::PluginEvent::Heartbeat { pid } => {
            LOGGER_DISPATCHER.debug(format!("Heartbeat received from runner {pid}",));
        }

        _ => {}
    }
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
    LOGGER_DISPATCHER.debug(format!("Sending {context} response"));

    if let Err(e) = reply_tx.send(response) {
        LOGGER_DISPATCHER.error(format!(
            "Failed to send {context} response to client thread: {e}"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ipc_protocol::ipc_payload_interface::InterfaceResponse;
    use plugin_manager::PluginEvent;

    #[test]
    fn test_handle_switch_notification() {
        let dummy_uuid = [8u8; 16];
        let response = handle_switch_notification(102, dummy_uuid);

        match response {
            InterfaceResponse::SwitchDone { request_id, .. } => assert_eq!(request_id, 102),
            InterfaceResponse::Error { request_id, .. } => assert_eq!(request_id, 102),
            _ => panic!("Unexpected response type"),
        }
    }

    #[test]
    fn test_send_response_helper() {
        let (tx, rx) = std::sync::mpsc::channel();
        let response = InterfaceResponse::Error {
            request_id: 1,
            message: "test".to_string(),
        };

        send_response(tx, response, "test_context");

        let received = rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap();
        match received {
            InterfaceResponse::Error { request_id, .. } => assert_eq!(request_id, 1),
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_handle_plugin_event_response_all_variants() {
        let uuid = [0u8; 16];

        let ev1 = PluginEvent::Result {
            pid: 1,
            request_id: 1,
            ok: true,
            output: "ok".into(),
        };
        let r1 = handle_plugin_event_response(10, uuid, "p", "f", ev1);
        assert!(matches!(r1, InterfaceResponse::CallResult { ok: true, .. }));

        let ev2 = PluginEvent::Error {
            pid: 1,
            request_id: 1,
            message: "err".into(),
        };
        let r2 = handle_plugin_event_response(11, uuid, "p", "f", ev2);
        assert!(matches!(
            r2,
            InterfaceResponse::CallResult { ok: false, .. }
        ));

        let ev3 = PluginEvent::Closed {
            pid: 1,
            reason: "exit".into(),
        };
        let r3 = handle_plugin_event_response(12, uuid, "p", "f", ev3);
        assert!(matches!(r3, InterfaceResponse::Error { .. }));

        let ev4 = PluginEvent::Heartbeat { pid: 1 };
        let r4 = handle_plugin_event_response(13, uuid, "p", "f", ev4);
        assert!(matches!(r4, InterfaceResponse::Error { .. }));
    }

    #[test]
    fn plugin_event_is_forwarded_without_polling() {
        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let mut pending_calls = HashMap::from([(
            5,
            PendingCall {
                request_id: 77,
                plugin_uuid: [0; 16],
                plugin_name: "audit".to_string(),
                function_name: "run".to_string(),
                reply_tx,
            },
        )]);

        handle_plugin_event(
            PluginEvent::Result {
                pid: 1,
                request_id: 5,
                ok: true,
                output: "done".to_string(),
            },
            &mut pending_calls,
        );

        assert!(pending_calls.is_empty());
        assert!(matches!(
            reply_rx.try_recv(),
            Ok(InterfaceResponse::CallResult {
                request_id: 77,
                ok: true,
                ..
            })
        ));
    }
}
