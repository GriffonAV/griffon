use nix::unistd::{Gid, Group, Uid, chown};
use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::sync::{Arc, Mutex, mpsc};

use uuid::Uuid;

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest, InterfaceResponse, recv_interface_request, send_interface_response,
};
use logger::Logger;

use crate::types::DaemonTask;

static LOGGER_NETWORK: Logger = if cfg!(debug_assertions) {
    Logger::new("DAEMON-INTERFACE-NETWORK", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "DAEMON-INTERFACE-NETWORK",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

static LOGGER_CORE: Logger = if cfg!(debug_assertions) {
    Logger::new("DAEMON-CORE", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "DAEMON-CORE",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

pub const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    // "/tmp/griffon-dev.sock"
    // Use a fixed path in the Griffon directory to avoid issues with some IDEs that create random temp directories
    "./griffon.sock"
} else {
    "/run/griffon/griffon.sock"
};

fn configure_socket_permissions(path: &Path) -> io::Result<()> {
    let group = Group::from_name("griffon")
        .map_err(|e| io::Error::other(format!("failed to lookup group 'griffon': {e}")))?
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "group 'griffon' does not exist"))?;

    chown(
        path,
        Some(Uid::from_raw(0)),
        Some(Gid::from_raw(group.gid.as_raw())),
    )
    .map_err(|e| {
        io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("failed to chown socket to root:griffon: {e}"),
        )
    })?;

    fs::set_permissions(path, fs::Permissions::from_mode(0o660))?;

    LOGGER_NETWORK.debug(format!(
        "Socket permissions set to root:griffon 660 for {}",
        path.display()
    ));

    Ok(())
}

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

    #[cfg(not(debug_assertions))]
    configure_socket_permissions(path)?;

    #[cfg(debug_assertions)]
    if let Err(e) = configure_socket_permissions(path) {
        LOGGER_NETWORK.warn(format!(
            "Could not configure socket permissions in debug mode: {}",
            e
        ));
    }

    Ok(listener)
}

pub fn handle_client(mut stream: UnixStream, task_tx: mpsc::Sender<DaemonTask>) -> io::Result<()> {
    LOGGER_NETWORK.debug("Client handler started");

    let writer = Arc::new(Mutex::new(stream.try_clone()?));

    loop {
        let (frame, req) = match recv_interface_request(&mut stream) {
            Ok(v) => v,
            Err(e) => {
                LOGGER_NETWORK.warn(format!("Client disconnected or invalid request: {e}"));
                return Ok(());
            }
        };

        let request_id = frame.request_id;

        LOGGER_NETWORK.debug(format!(
            "Header: version={}, mtype={:?}, request_id={} BODY: data={:?}",
            frame.version, frame.mtype, request_id, req
        ));

        let (reply_tx, reply_rx) = mpsc::channel();

        let task = match req {
            InterfaceRequest::SwitchStatusNotification { plugin_uuid } => {
                LOGGER_CORE.debug(format!(
                    "Switch notification status for plugin {}",
                    Uuid::from_bytes(plugin_uuid)
                ));

                DaemonTask::SwitchStatusNotification {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                }
            }

            InterfaceRequest::SwitchStatusPlugin { plugin_uuid } => {
                LOGGER_CORE.debug(format!(
                    "Switch status plugin {}",
                    Uuid::from_bytes(plugin_uuid)
                ));

                DaemonTask::SwitchStatusPlugin {
                    request_id,
                    plugin_uuid,
                    reply_tx,
                }
            }

            InterfaceRequest::RefreshPlugins => DaemonTask::RefreshPlugins {
                request_id,
                reply_tx,
            },

            InterfaceRequest::CallPlugin {
                plugin_uuid,
                fn_name,
                args,
            } => {
                LOGGER_CORE.debug(format!(
                    "Function {} requested for plugin {} with args {:?}",
                    fn_name,
                    Uuid::from_bytes(plugin_uuid),
                    args
                ));

                DaemonTask::CallPlugin {
                    request_id,
                    plugin_uuid,
                    fn_name,
                    args,
                    reply_tx,
                }
            }

            _ => {
                let response = InterfaceResponse::Error {
                    request_id,
                    message: "Request not implemented yet".to_string(),
                };

                send_interface_response(&mut stream, &response)?;
                continue;
            }
        };

        // Transmettre la tâche sans attendre son résultat.
        if let Err(e) = task_tx.send(task) {
            LOGGER_CORE.error(format!("Failed to queue task: {e}"));

            let response = InterfaceResponse::Error {
                request_id,
                message: format!("Failed to queue task: {e}"),
            };

            let mut socket = writer
                .lock()
                .map_err(|_| io::Error::other("Socket writer mutex poisoned"))?;

            send_interface_response(&mut *socket, &response)?;
            continue;
        }

        // Attendre la réponse dans un thread séparé afin que la lecture
        // des prochaines requêtes puisse continuer.
        let response_writer = Arc::clone(&writer);

        std::thread::spawn(move || {
            let response = reply_rx
                .recv()
                .unwrap_or_else(|e| InterfaceResponse::Error {
                    request_id,
                    message: format!("Dispatcher response channel closed: {e}"),
                });

            match response_writer.lock() {
                Ok(mut socket) => {
                    if let Err(e) = send_interface_response(&mut *socket, &response) {
                        LOGGER_NETWORK.error(format!(
                            "Failed to send response for request {}: {}",
                            request_id, e
                        ));
                    } else {
                        LOGGER_NETWORK.debug(format!(
                            "Response sent for request {}: {:?}",
                            request_id, response
                        ));
                    }
                }
                Err(e) => {
                    LOGGER_NETWORK.error(format!(
                        "Socket writer mutex poisoned for request {}: {}",
                        request_id, e
                    ));
                }
            }
        });
    }
}
