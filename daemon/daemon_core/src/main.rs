use std::fs;
use std::io;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use serde::{Deserialize, Serialize};
use ipc_protocol;
use plugin_manager::{LogLevel, PluginManager};
use logger;

use logger::Logger;
static PLUGIN_DIR_PATH: &str = "./plugins";

static LOGGER_NETWORK: Logger = Logger::new("DAEMON-INTERFACE-NETWORK", logger::LogLevel::Debug);
static LOGGER_CORE: Logger = Logger::new("DAEMON-CORE", logger::LogLevel::Debug);

const DAEMON_SOCK_PATH: &str = "/run/griffon/daemon.sock";

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfoDto {
    pub pid: u32,
    pub name: String,
    pub path: String,
    pub functions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonResponse {
    Pong,
    Ok,
    Error { message: String },
    Plugins(Vec<PluginInfoDto>),
    CallAccepted { request_id: u32 },
}

fn setup_listener() -> io::Result<UnixListener> {
    let path = Path::new(DAEMON_SOCK_PATH);

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        LOGGER_NETWORK.debug(&format!(
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
    LOGGER_NETWORK.debug(&format!("Socket bound on {}", path.display()));

    Ok(listener)
}

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest,
    InterfaceResponse,
    recv_interface_request,
    send_interface_response,
};
use uuid::Uuid;

fn handle_client(mut stream: UnixStream) -> io::Result<()> {
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
            InterfaceRequest::Ping => InterfaceResponse::Pong,

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
                InterfaceResponse::CallAccepted {
                    request_id: frame.request_id,
                }
            }

            _ => InterfaceResponse::Error {
                message: "Request not implemented yet".to_string(),
            },
        };

        send_interface_response(&mut stream, &resp)?;
        LOGGER_NETWORK.debug(format!("Response sent: {:?}", resp));
    }
}

fn main() -> io::Result<()> {
    let mut pm = plugin_manager::PluginManager::new(PLUGIN_DIR_PATH);
    LOGGER_NETWORK.debug("TESTS");
    let listener = setup_listener()?;
    LOGGER_NETWORK.debug("Setup listener finish");

    pm.scan_dir();
    pm.list_plugins();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    LOGGER_NETWORK.debug("New thread for client");
                    if let Err(e) = handle_client(stream) {
                        LOGGER_NETWORK.error(format!("client error :{e}"));
                    }
                });
            }
            Err(e) => {
                LOGGER_NETWORK.error(format!("stream error :{e}"));
            }
        }
    }

    Ok(())
}
