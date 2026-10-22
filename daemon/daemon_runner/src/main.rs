use ipc_protocol::ipc_payload_runner::{
    CallPayload, ErrorPayload, HelloOkPayload, Message, ResultPayload, recv_message, send_message,
};
use logger::{LogLevel, Logger};
use plugin_host::{LoadedPlugin, PluginInfo, is_shared_library};
use std::io;
use std::os::fd::FromRawFd;
use std::path::Path;
use std::process::exit;
use uuid::Uuid;

static LOGGER_RUNNER: Logger = if cfg!(debug_assertions) {
    Logger::new("RUNNER", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "RUNNER",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

static LOGGER_RUNNER_NETWORK: Logger = if cfg!(debug_assertions) {
    Logger::new("RUNNER-NETWORK", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "RUNNER-NETWORK",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

/// Calls the plugin's `init()`. On failure the runner keeps going with empty
/// metadata (name falls back to the file name, UUID to nil).
fn init_plugin(plugin: &LoadedPlugin) -> PluginInfo {
    let file_name = plugin.path().display().to_string();

    match plugin.init() {
        Ok(info) => {
            LOGGER_RUNNER.info(format!(
                "init() returned ok with {} entries",
                info.entries.len()
            ));
            if LOGGER_RUNNER.level() == LogLevel::Debug {
                for (key, value) in &info.entries {
                    println!("    {key} => {value}");
                }
            }
            info
        }
        Err(err_msg) => {
            eprintln!("[RUNNER {file_name}] init() ERROR: {err_msg}");
            PluginInfo::default()
        }
    }
}

fn build_hello_ok(info: &PluginInfo, fallback_name: &str) -> HelloOkPayload {
    let name = if info.name.trim().is_empty() {
        fallback_name.to_string()
    } else {
        info.name.clone()
    };

    let uuid_string = info.uuid.clone();

    let functions = info.functions.clone();

    LOGGER_RUNNER_NETWORK.debug(format!(
        "name: {name}, uuid: {uuid_string}, functions: {:?}",
        functions
    ));

    let uuid: [u8; 16] = Uuid::parse_str(&uuid_string)
        .unwrap_or_else(|_| Uuid::nil())
        .into_bytes();

    HelloOkPayload {
        name,
        uuid,
        functions,
    }
}

fn handle_call(plugin: &LoadedPlugin, call: CallPayload) -> io::Result<String> {
    plugin
        .call(&call.fn_name, &call.args)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("[RUNNER](ERROR) Bad runner usage: runner <plugin.so>");
        exit(1);
    }

    let so_path = Path::new(&args[1]);
    let tmp_name = so_path.display().to_string();
    let fallback_name = tmp_name.rsplit('/').next().unwrap().to_string();

    if !is_shared_library(so_path) {
        eprintln!(
            "[RUNNER](ERROR): {} is not a compatible plugin.",
            so_path.display()
        );
        exit(1);
    }

    let plugin = LoadedPlugin::load(so_path).unwrap_or_else(|e| {
        eprintln!("[RUNNER](ERROR) Failed to load plugin: {e}");
        exit(1);
    });

    let info = init_plugin(&plugin);

    let fd = 3;
    let mut sock = unsafe { std::os::unix::net::UnixStream::from_raw_fd(fd) };

    loop {
        let msg = match recv_message(&mut sock) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("[RUNNER {fallback_name}](INFO) IPC closed / recv error: {e}");
                break;
            }
        };

        match msg {
            Message::Hello => {
                let payload = build_hello_ok(&info, &fallback_name);
                if let Err(e) = send_message(&mut sock, Message::HelloOk(payload)) {
                    eprintln!("[RUNNER {fallback_name}](ERROR) failed to send HelloOk: {e}");
                    break;
                }
            }

            Message::Call { request_id, data } => match handle_call(&plugin, data) {
                Ok(output) => {
                    let res = ResultPayload { ok: true, output };
                    if let Err(e) = send_message(
                        &mut sock,
                        Message::Result {
                            request_id,
                            data: res,
                        },
                    ) {
                        eprintln!(
                            "[RUNNER {fallback_name}](ERROR) failed to send Result (id={request_id}): {e}"
                        );
                        break;
                    }
                }
                Err(e) => {
                    let err = ErrorPayload {
                        code: 1,
                        message: e.to_string(),
                    };
                    let _ = send_message(
                        &mut sock,
                        Message::Error {
                            request_id,
                            data: err,
                        },
                    );
                }
            },

            Message::Heartbeat => {
                // TODO : Heartbeat
            }

            other => {
                eprintln!(
                    "[RUNNER {fallback_name}](WARN) Unexpected message from core: {:?}",
                    other
                );
            }
        }
    }
}
