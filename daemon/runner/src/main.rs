use abi_stable::library::lib_header_from_path;
use abi_stable::std_types::{RResult, RString, Tuple2};
use std::io;
use std::os::fd::FromRawFd;
use std::path::{Path, PathBuf};
use std::process::exit;

use ipc_protocol::ipc_payload_runner::{
    CallPayload, ErrorPayload, HelloOkPayload, Message, ResultPayload, recv_message, send_message,
};
use logger::{LogLevel, Logger};
use plugin_interface::{PluginRef, PluginRoot_Ref};

static LOGGER_RUNNER: Logger = Logger::new("RUNNER", logger::LogLevel::Debug);
static LOGGER_RUNNER_NETWORK: Logger = Logger::new("RUNNER-NETWORK", logger::LogLevel::Debug);

struct LoadedPlugin {
    root: PluginRoot_Ref,
    path: PathBuf,
    uuid: RString,
    functions: RString,
    name: RString,
}

fn spawn_start_if_exists(plugin: &mut LoadedPlugin) {
    let file_name = plugin.path.display().to_string();

    let plugin_ref: &PluginRef = &plugin.root.plugin();
    let init_fn = match plugin_ref.init() {
        Some(f) => f,
        None => {
            eprintln!("[RUNNER {file_name}] ERROR: init() is None");
            return;
        }
    };

    let init_result = init_fn();

    match init_result {
        RResult::ROk(vec) => {
            LOGGER_RUNNER.info(format!("init() returned ok with {} entries", vec.len()));

            for Tuple2(key, value) in vec {
                if key == "function" {
                    plugin.functions = value.clone();
                }
                if key == "name" {
                    plugin.name = value.clone();
                }
                if key == "UUID" {
                    plugin.uuid = value.clone();
                }
                if LOGGER_RUNNER.level() == LogLevel::Debug {
                    println!("    {} => {}", key.as_str(), value.as_str());
                }
            }
        }
        RResult::RErr(err_msg) => {
            eprintln!("[RUNNER {file_name}] init() ERROR: {}", err_msg.as_str());
        }
    }
}

fn load_plugin(path: &Path) -> Result<LoadedPlugin, String> {
    let header = lib_header_from_path(path).map_err(|e| format!("header load failed: {e}"))?;

    let root: PluginRoot_Ref = header
        .init_root_module::<PluginRoot_Ref>()
        .map_err(|e| format!("init root failed: {e}"))?;

    Ok(LoadedPlugin {
        root,
        path: path.to_path_buf(),
        uuid: "".into(),
        functions: "".into(),
        name: "".into(),
    })
}

fn is_shared_library(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|ext| ext == "so")
}

fn parse_functions(s: &str) -> Vec<String> {
    s.split(['/', ','])
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn build_hello_ok(plugin: &LoadedPlugin, fallback_name: &str) -> HelloOkPayload {
    let name = if plugin.name.as_str().trim().is_empty() {
        fallback_name.to_string()
    } else {
        plugin.name.as_str().to_string()
    };

    let uuid = plugin.uuid.as_str().to_string();

    let functions = parse_functions(plugin.functions.as_str());

    LOGGER_RUNNER_NETWORK.debug(format!(
        "name: {name}, uuid: {uuid}, functions: {:?}",
        functions
    ));
    HelloOkPayload {
        name,
        uuid,
        functions,
    }
}

/// - args vide  => "fn:ping"
/// - args [2]   => "fn:ping 2"
/// - args [a,b] => "fn:scan_file a b"
fn call_to_wire(call: &CallPayload) -> String {
    // this part will change in the v2
    if call.args.is_empty() {
        format!("fn:{}", call.fn_name)
    } else {
        format!("fn:{} {}", call.fn_name, call.args.join(" "))
    }
}

fn handle_call(plugin: &LoadedPlugin, call: CallPayload) -> io::Result<String> {
    let plugin_ref: &PluginRef = &plugin.root.plugin();
    let handle_fn = plugin_ref.handle_message().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "plugin handle_message() is None",
        )
    })?;

    let wire = call_to_wire(&call);
    let response: RString = handle_fn(RString::from(wire));

    Ok(response.as_str().to_string())
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

    let mut plugin = load_plugin(so_path).unwrap_or_else(|e| {
        eprintln!("[RUNNER](ERROR) Failed to load plugin: {e}");
        exit(1);
    });

    spawn_start_if_exists(&mut plugin);

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
                let payload = build_hello_ok(&plugin, &fallback_name);
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
