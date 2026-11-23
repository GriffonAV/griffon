use std::collections::HashMap;
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::Duration;

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

        let mut pending_calls: HashMap<u32, PendingCall> = HashMap::new();

        loop {
            match task_rx.recv_timeout(Duration::from_millis(50)) {
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

                Err(RecvTimeoutError::Timeout) => {}

                Err(RecvTimeoutError::Disconnected) => {
                    LOGGER_DISPATCHER.warn("Task channel disconnected; stopping dispatcher");
                    break;
                }
            }

            while let Some(event) = pm.try_recv_event() {
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

                    continue;
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

#[cfg(test)]
mod notification_switch {
    use super::*;

    #[test]
    fn returns_a_response_with_the_request_id() {
        let _guard = crate::notification::TEST_CONFIG_LOCK.lock().unwrap();
        let response = handle_switch_notification(102, [8u8; 16]);

        match response {
            InterfaceResponse::SwitchDone { request_id, .. }
            | InterfaceResponse::Error { request_id, .. } => assert_eq!(request_id, 102),
            _ => panic!("Unexpected response type"),
        }
    }
}

#[cfg(test)]
mod plugin_events {
    use super::*;
    use plugin_manager::PluginEvent;

    #[test]
    fn converts_plugin_events_into_interface_responses() {
        let uuid = [0u8; 16];

        let result = PluginEvent::Result {
            pid: 1,
            request_id: 1,
            ok: true,
            output: "ok".into(),
        };
        assert!(matches!(
            handle_plugin_event_response(10, uuid, "p", "f", result),
            InterfaceResponse::CallResult {
                request_id: 10,
                ok: true,
                output
            } if output == "ok"
        ));

        let error = PluginEvent::Error {
            pid: 1,
            request_id: 1,
            message: "err".into(),
        };
        assert!(matches!(
            handle_plugin_event_response(11, uuid, "p", "f", error),
            InterfaceResponse::CallResult {
                request_id: 11,
                ok: false,
                output
            } if output == "err"
        ));

        let closed = PluginEvent::Closed {
            pid: 1,
            reason: "exit".into(),
        };
        assert!(matches!(
            handle_plugin_event_response(12, uuid, "p", "f", closed),
            InterfaceResponse::Error { request_id: 12, message }
                if message == "Plugin closed: exit"
        ));

        assert!(matches!(
            handle_plugin_event_response(13, uuid, "p", "f", PluginEvent::Heartbeat { pid: 1 }),
            InterfaceResponse::Error { request_id: 13, message }
                if message == "Unexpected message type while waiting for response"
        ));
    }
}

#[cfg(test)]
mod response_delivery {
    use super::*;

    #[test]
    fn sends_response_to_client() {
        let (tx, rx) = std::sync::mpsc::channel();
        send_response(
            tx,
            InterfaceResponse::Error {
                request_id: 1,
                message: "test".into(),
            },
            "test_context",
        );

        assert!(matches!(
            rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            InterfaceResponse::Error { request_id: 1, .. }
        ));
    }

    #[test]
    fn handles_disconnected_client() {
        let (tx, rx) = std::sync::mpsc::channel();
        drop(rx);

        send_response(
            tx,
            InterfaceResponse::Error {
                request_id: 7,
                message: "client disconnected".into(),
            },
            "test_context",
        );
    }
}

#[cfg(test)]
mod task_routing {
    use super::*;
    use std::sync::Mutex;

    static RUNNER_ENV_LOCK: Mutex<()> = Mutex::new(());

    struct RunnerEnvRestore(Option<std::ffi::OsString>);

    impl Drop for RunnerEnvRestore {
        fn drop(&mut self) {
            match self.0.take() {
                Some(path) => unsafe { std::env::set_var("GRIFFON_RUNNER_BINARY", path) },
                None => unsafe { std::env::remove_var("GRIFFON_RUNNER_BINARY") },
            }
        }
    }

    #[test]
    fn routes_plugin_tasks_without_launching_external_runner() {
        let _guard = RUNNER_ENV_LOCK.lock().unwrap();
        let _restore_env = RunnerEnvRestore(std::env::var_os("GRIFFON_RUNNER_BINARY"));
        let temp_dir = tempfile::tempdir().expect("create plugin directory");
        // The plugin directory is empty, so this path is only used to
        // construct PluginManager; the dispatcher never launches it.
        unsafe { std::env::set_var("GRIFFON_RUNNER_BINARY", temp_dir.path().join("fake-runner")) };

        let plugin_dir: &'static str = Box::leak(
            temp_dir
                .path()
                .to_string_lossy()
                .into_owned()
                .into_boxed_str(),
        );
        let (task_tx, task_rx) = std::sync::mpsc::channel();
        start_dispatcher(task_rx, plugin_dir);

        let (switch_tx, switch_rx) = std::sync::mpsc::channel();
        task_tx
            .send(crate::types::DaemonTask::SwitchStatusPlugin {
                request_id: 21,
                plugin_uuid: [2; 16],
                reply_tx: switch_tx,
            })
            .unwrap();

        let (refresh_tx, refresh_rx) = std::sync::mpsc::channel();
        task_tx
            .send(crate::types::DaemonTask::RefreshPlugins {
                request_id: 22,
                reply_tx: refresh_tx,
            })
            .unwrap();

        let (call_tx, call_rx) = std::sync::mpsc::channel();
        task_tx
            .send(crate::types::DaemonTask::CallPlugin {
                request_id: 23,
                plugin_uuid: [2; 16],
                fn_name: "missing_function".into(),
                args: Vec::new(),
                reply_tx: call_tx,
            })
            .unwrap();
        drop(task_tx);

        assert!(matches!(
            switch_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            InterfaceResponse::Error { request_id: 21, .. }
        ));
        assert!(matches!(
            refresh_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            InterfaceResponse::Plugins { request_id: 22, plugins } if plugins.is_empty()
        ));
        assert!(matches!(
            call_rx.recv_timeout(Duration::from_secs(1)).unwrap(),
            InterfaceResponse::Error { request_id: 23, .. }
        ));
    }
}
