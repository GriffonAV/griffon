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


#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::net::UnixStream;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use ipc_protocol::ipc_payload_interface::{
        InterfaceRequest, InterfaceResponse, send_interface_request, recv_interface_response,
    };

    #[test]
    fn test_handle_client_refresh_plugins_success() {
        let (mut client_socket, server_socket) = UnixStream::pair().expect("Failed to create socket pair");

        let (task_tx, task_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        let request_id = 42;
        let req = InterfaceRequest::RefreshPlugins;
        send_interface_request(&mut client_socket, &req, request_id)
            .expect("Failed to send interface request from client");

        let received_task = task_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("Timeout waiting for DaemonTask");

        match received_task {
            DaemonTask::RefreshPlugins { request_id: req_id, reply_tx } => {
                assert_eq!(req_id, request_id);

                let dummy_response = InterfaceResponse::Plugins {
                    request_id: req_id,
                    plugins: vec![],
                };
                reply_tx.send(dummy_response).expect("Failed to send reply to client handler");
            }
            other => panic!("Unexpected DaemonTask variant received: {:?}", other),
        }

        let response = recv_interface_response(&mut client_socket)
            .expect("Failed to receive interface response on client");

        match response {
            InterfaceResponse::Plugins { request_id: resp_req_id, plugins } => {
                assert_eq!(resp_req_id, request_id);
                assert!(plugins.is_empty());
            }
            other => panic!("Unexpected InterfaceResponse received: {:?}", other),
        }

        drop(client_socket);
        let _ = handle.join();
    }

    #[test]
    fn test_handle_client_unimplemented_request() {
        let (mut client_socket, server_socket) = UnixStream::pair().expect("Failed to create socket pair");
        let (task_tx, _task_rx) = mpsc::channel();

        thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        let request_id = 100;
        
        let req = InterfaceRequest::TestUnimplemented; 

        send_interface_request(&mut client_socket, &req, request_id)
            .expect("Failed to send interface request");

        let response = recv_interface_response(&mut client_socket)
            .expect("Failed to receive interface response on client");

        match response {
            InterfaceResponse::Error { request_id: resp_req_id, message } => {
                assert_eq!(resp_req_id, request_id);
                assert!(
                    message.contains("Request not implemented"),
                    "Message d'erreur inattendu : {}",
                    message
                );
            }
            other => panic!("Expected InterfaceResponse::Error, got {:?}", other),
        }
    }

    #[test]
    fn test_handle_client_task_tx_disconnected() {
        let (mut client_socket, server_socket) = UnixStream::pair().expect("Failed to create socket pair");
        
        let (task_tx, task_rx) = mpsc::channel::<DaemonTask>();
        drop(task_rx); 

        let handle = thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        let request_id = 200;
        let req = InterfaceRequest::RefreshPlugins;
        send_interface_request(&mut client_socket, &req,request_id)
            .expect("Failed to send interface request");

        let response = recv_interface_response(&mut client_socket)
            .expect("Failed to receive response when channel is disconnected");

        match response {
            InterfaceResponse::Error { request_id: resp_req_id, message } => {
                assert_eq!(resp_req_id, request_id);
                assert!(
                    message.contains("Failed to queue task"),
                    "Le message devrait indiquer l'échec d'envoi de la tâche : {}",
                    message
                );
            }
            other => panic!("Expected InterfaceResponse::Error, got {:?}", other),
        }

        drop(client_socket);
        let _ = handle.join();
    }

   #[test]
    fn test_setup_listener_removes_old_socket() {
        use std::path::Path;

        let path = Path::new(DAEMON_SOCK_PATH);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        std::fs::write(path, b"old socket garbage data").expect("Failed to create fake old socket");
        assert!(path.exists());

        let listener = setup_listener();

        assert!(
            listener.is_ok(),
            "setup_listener a échoué avec l'erreur: {:?}",
            listener.err()
        );
        assert!(path.exists());

        drop(listener);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn test_handle_client_all_request_types() {
        let (mut client_socket, server_socket) = UnixStream::pair().unwrap();
        let (task_tx, task_rx) = mpsc::channel();

        std::thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        let dummy_uuid = [1u8; 16];

        let req1 = InterfaceRequest::SwitchStatusNotification { plugin_uuid: dummy_uuid };
        send_interface_request(&mut client_socket, &req1, 1).unwrap();
        let task1 = task_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(task1, DaemonTask::SwitchStatusNotification { request_id: 1, .. }));

        let req2 = InterfaceRequest::SwitchStatusPlugin { plugin_uuid: dummy_uuid };
        send_interface_request(&mut client_socket, &req2, 2).unwrap();
        let task2 = task_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(task2, DaemonTask::SwitchStatusPlugin { request_id: 2, .. }));

        let req3 = InterfaceRequest::CallPlugin {
            plugin_uuid: dummy_uuid,
            fn_name: "test_fn".into(),
            args: vec!["arg1".into()],
        };
        send_interface_request(&mut client_socket, &req3, 3).unwrap();
        let task3 = task_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(task3, DaemonTask::CallPlugin { request_id: 3, .. }));
    }

    #[test]
    fn test_handle_client_invalid_payload_disconnects() {
        let (mut client_socket, server_socket) = UnixStream::pair().expect("Failed to create socket pair");
        let (task_tx, _task_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        use std::io::Write;
        let _ = client_socket.write_all(b"invalid garbage header payload data");

        let _ = handle.join();
    }

    #[test]
    fn test_handle_client_reply_channel_drop_sends_error() {
        let (mut client_socket, server_socket) = UnixStream::pair().expect("Failed to create socket pair");
        let (task_tx, task_rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let _ = handle_client(server_socket, task_tx);
        });

        let request_id = 999;
        let req = InterfaceRequest::RefreshPlugins;
        send_interface_request(&mut client_socket, &req, request_id).unwrap();

        let task = task_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        match task {
            DaemonTask::RefreshPlugins { reply_tx, .. } => {
                drop(reply_tx);
            }
            _ => panic!("Expected RefreshPlugins task"),
        }

        let response = recv_interface_response(&mut client_socket)
            .expect("Failed to receive response when reply_tx is dropped");

        match response {
            InterfaceResponse::Error { request_id: resp_id, message } => {
                assert_eq!(resp_id, request_id);
                assert!(message.contains("Dispatcher response channel closed"));
            }
            other => panic!("Expected InterfaceResponse::Error, got {:?}", other),
        }

        drop(client_socket);
        let _ = handle.join();
    }
}