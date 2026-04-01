use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::mpsc;

use uuid::Uuid;

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest, InterfaceResponse, recv_interface_request, send_interface_response,
};
use logger::Logger;

use crate::types::DaemonTask;

static LOGGER_NETWORK: Logger = Logger::new("DAEMON-INTERFACE-NETWORK", logger::LogLevel::Debug);
static LOGGER_CORE: Logger = Logger::new("DAEMON-CORE", logger::LogLevel::Debug);

pub const DAEMON_SOCK_PATH: &str = "/run/griffon/daemon.sock";

pub fn setup_listener() -> io::Result<UnixListener> {
    let path = Path::new(DAEMON_SOCK_PATH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        LOGGER_NETWORK.debug(format!(
            "Socket parent directory ready: {}",
            parent.display()
        ));
    }

    match fs::remove_file(path) {
        Ok(_) => LOGGER_NETWORK.debug("Old socket file removed"),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            LOGGER_NETWORK.debug("No old socket file to remove")
        }
        Err(e) => return Err(e),
    }

    let listener = UnixListener::bind(path)?;
    LOGGER_NETWORK.debug(format!("Socket bound on {}", path.display()));

    Ok(listener)
}

pub fn handle_client(mut stream: UnixStream, task_tx: mpsc::Sender<DaemonTask>) -> io::Result<()> {
    LOGGER_NETWORK.debug("Client handler started");

    loop {
        let (frame, req) = match recv_interface_request(&mut stream) {
            Ok(v) => v,
            Err(e) => {
                LOGGER_NETWORK.warn(format!("Client disconnected or invalid request: {e}"));
                return Ok(());
            }
        };

        LOGGER_NETWORK.debug(format!(
            "Header: version={}, mtype={:?}, request_id={} BODY: data={:?}",
            frame.version, frame.mtype, frame.request_id, req
        ));

        let resp = match req {
            InterfaceRequest::StartPlugin { plugin_uuid } => {
                let plugin_uuid_str = Uuid::from_bytes(plugin_uuid).to_string();
                LOGGER_CORE.debug(format!("Start plugin {}", plugin_uuid_str));

                let (reply_tx, reply_rx) = mpsc::channel();

                let task = DaemonTask::StartPlugin {
                    request_id: frame.request_id,
                    plugin_uuid,
                    reply_tx,
                };

                if let Err(e) = task_tx.send(task) {
                    LOGGER_CORE.error(format!("Failed to queue task: {e}"));
                    InterfaceResponse::Error {
                        request_id: frame.request_id,
                        message: format!("Failed to queue task: {e}"),
                    }
                } else {
                    reply_rx
                        .recv()
                        .unwrap_or_else(|e| InterfaceResponse::Error {
                            request_id: frame.request_id,
                            message: format!("Dispatcher response channel closed: {e}"),
                        })
                }
            }
            InterfaceRequest::StopPlugin { plugin_uuid } => {
                let plugin_uuid_str = Uuid::from_bytes(plugin_uuid).to_string();
                LOGGER_CORE.debug(format!("Stop plugin {}", plugin_uuid_str));

                let (reply_tx, reply_rx) = mpsc::channel();

                let task = DaemonTask::StopPlugin {
                    request_id: frame.request_id,
                    plugin_uuid,
                    reply_tx,
                };

                if let Err(e) = task_tx.send(task) {
                    LOGGER_CORE.error(format!("Failed to queue task: {e}"));
                    InterfaceResponse::Error {
                        request_id: frame.request_id,
                        message: format!("Failed to queue task: {e}"),
                    }
                } else {
                    reply_rx
                        .recv()
                        .unwrap_or_else(|e| InterfaceResponse::Error {
                            request_id: frame.request_id,
                            message: format!("Dispatcher response channel closed: {e}"),
                        })
                }
            }
            InterfaceRequest::RefreshPlugins => {
                let (reply_tx, reply_rx) = mpsc::channel();

                let task = DaemonTask::RefreshPlugins {
                    request_id: frame.request_id,
                    reply_tx,
                };

                if let Err(e) = task_tx.send(task) {
                    LOGGER_CORE.error(format!("Failed to queue refresh task: {e}"));
                    InterfaceResponse::Error {
                        request_id: frame.request_id,
                        message: format!("Failed to queue refresh task: {e}"),
                    }
                } else {
                    reply_rx
                        .recv()
                        .unwrap_or_else(|e| InterfaceResponse::Error {
                            request_id: frame.request_id,
                            message: format!("Dispatcher response channel closed: {e}"),
                        })
                }
            }
            InterfaceRequest::CallPlugin {
                plugin_uuid,
                fn_name,
                args,
            } => {
                let plugin_uuid_str = Uuid::from_bytes(plugin_uuid).to_string();
                LOGGER_CORE.debug(format!(
                    "Fn {} to execute for plugin {:?} with args {:?}",
                    fn_name, plugin_uuid_str, args
                ));

                let (reply_tx, reply_rx) = mpsc::channel();

                let task = DaemonTask::CallPlugin {
                    request_id: frame.request_id,
                    plugin_uuid,
                    fn_name,
                    args,
                    reply_tx,
                };

                if let Err(e) = task_tx.send(task) {
                    LOGGER_CORE.error(format!("Failed to queue task: {e}"));
                    InterfaceResponse::Error {
                        request_id: frame.request_id,
                        message: format!("Failed to queue task: {e}"),
                    }
                } else {
                    reply_rx
                        .recv()
                        .unwrap_or_else(|e| InterfaceResponse::Error {
                            request_id: frame.request_id,
                            message: format!("Dispatcher response channel closed: {e}"),
                        })
                }
            }

            _ => InterfaceResponse::Error {
                request_id: frame.request_id,
                message: "Request not implemented yet".to_string(),
            },
        };

        send_interface_response(&mut stream, &resp)?;
        LOGGER_NETWORK.debug(format!("Response sent: {:?}", resp));
    }
}
