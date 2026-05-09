use std::io;
use std::os::unix::net::UnixStream;
use std::thread;

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest, InterfaceResponse, alloc_request_id, format_uuid_bytes, parse_uuid_16,
    recv_interface_response, send_interface_request,
};
use logger::{LogLevel, Logger};

static LOGGER: Logger = Logger::new("CLI", LogLevel::Debug, None);
static LOGGER_NETWORK: Logger = Logger::new("CLI-NETWORK", LogLevel::Debug, None);
const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    "/tmp/griffon-dev.sock"
} else {
    "/run/griffon/griffon.sock"
};

fn print_help() {
    println!();
    println!("GriffonAV CLI");
    println!();
    println!("Usage:");
    println!("  griffonav-cli");
    println!();
    println!("Commands:");
    println!("  help");
    println!("      Show this help message");
    println!();
    println!("  refresh");
    println!("      Refresh and display the plugin list from the daemon");
    println!();
    println!("  switch_status <plugin_uuid>");
    println!("      Enable or disable a plugin depending on its current status");
    println!();
    println!("  call <plugin_uuid> <fn_name> <arg1|arg2|...>");
    println!("      Call a plugin function with optional arguments");
    println!();
    println!("      Example:");
    println!("        call 550e8400-e29b-41d4-a716-446655440000 scan /tmp");
    println!("        call 550e8400-e29b-41d4-a716-446655440000 clean cache|true");
    println!();
    println!("  exit | quit");
    println!("      Exit the CLI");
    println!();
}

fn start_reader_thread(mut read_sock: UnixStream) {
    thread::spawn(move || {
        LOGGER_NETWORK.debug("Reader thread started");

        loop {
            match recv_interface_response(&mut read_sock) {
                Ok(resp) => match resp {
                    InterfaceResponse::Ok { request_id: _ } => {
                        LOGGER_NETWORK.info("Ok received");
                    }
                    InterfaceResponse::CallAccepted { request_id } => {
                        LOGGER_NETWORK.info(format!(
                            "Call accepted received for request_id={request_id}"
                        ));
                    }
                    InterfaceResponse::Plugins {
                        request_id,
                        plugins,
                    } => {
                        LOGGER_NETWORK.info(format!(
                            "Plugins list received: {} plugin(s) from request_id={request_id}",
                            plugins.len()
                        ));
                        for plugin in plugins {
                            println!(
                                "- UUID: {:?} | NAME: {} | PATH: {} | FUNCTIONS: {:?} | STATUS: {}",
                                format_uuid_bytes(&plugin.plugin_uuid),
                                plugin.name,
                                plugin.path,
                                plugin.functions,
                                plugin.status
                            );
                        }
                    }
                    InterfaceResponse::Error {
                        request_id,
                        message,
                    } => {
                        LOGGER_NETWORK.error(format!("Request {request_id} error={message}"));
                    }
                    InterfaceResponse::CallResult {
                        request_id,
                        ok,
                        output,
                    } => {
                        LOGGER_NETWORK
                            .info(format!("Call {request_id} result={ok} output={output}"));
                    }
                    InterfaceResponse::SwitchDone { request_id } => {
                        LOGGER_NETWORK.info(format!("Switch done with request_id = {request_id}"));
                    }
                },
                Err(e) => {
                    LOGGER_NETWORK.error(format!(
                        "Reader thread stopped, failed to receive response: {e}"
                    ));
                    break;
                }
            }
        }
    });
}

fn main() -> io::Result<()> {
    let mut id_request: u32 = 0;

    LOGGER_NETWORK.debug("Client try connected");
    let mut sock = UnixStream::connect(DAEMON_SOCK_PATH)?;
    LOGGER_NETWORK.info("Client connected");

    let read_sock = sock.try_clone()?;
    start_reader_thread(read_sock);

    loop {
        // print!("$> ");
        // io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            LOGGER.error("Could not read input");
            continue;
        }

        let trimmed = input.trim();
        if trimmed.is_empty() {
            continue;
        }

        let mut parts = trimmed.splitn(3, ' ');
        let cmd = parts.next().unwrap();

        match cmd {
            "help" | "--help" | "-h" => {
                print_help();
            }
            "refresh" => {
                LOGGER.debug("REFRESH");
                let id_request_to_use = alloc_request_id(id_request);

                send_interface_request(
                    &mut sock,
                    &InterfaceRequest::RefreshPlugins {},
                    id_request_to_use,
                )?;

                LOGGER_NETWORK.debug(format!(
                    "Refresh plugins sent with request_id={id_request_to_use}"
                ));

                id_request = id_request_to_use;
            }
            "exit" | "quit" => {
                LOGGER.debug("QUIT");
                return Ok(());
            }
            "restart" => {
                LOGGER.debug("RESTART");
                let pid_str = parts.next();

                if pid_str.is_none() {
                    LOGGER.error("Usage: restart <PID>");
                    continue;
                }

                let _pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        LOGGER.error("Invalid PID");
                        continue;
                    }
                };

                // TODO
            }
            "switch_status" => {
                LOGGER.debug("START");
                let plugin_uuid_str = parts.next();

                if plugin_uuid_str.is_none() {
                    LOGGER.error("Usage: start <plugin_uuid>");
                    continue;
                }

                let plugin_uuid = match parse_uuid_16(plugin_uuid_str) {
                    Some(uuid) => uuid,
                    None => {
                        LOGGER.error("Invalid UUID format");
                        continue;
                    }
                };

                let id_request_to_use = alloc_request_id(id_request);

                send_interface_request(
                    &mut sock,
                    &InterfaceRequest::SwitchStatusPlugin { plugin_uuid },
                    id_request_to_use,
                )?;

                LOGGER_NETWORK.debug(format!(
                    "Switch status plugin sent with request_id={id_request_to_use}"
                ));

                id_request = id_request_to_use;
            }
            "call" => {
                LOGGER.debug("CALL Sent");
                let plugin_uuid_str = parts.next();
                let rest = parts.next();

                if plugin_uuid_str.is_none() || rest.is_none() {
                    LOGGER.error("Usage: call <plugin_uuid> <fn_name> <arg1|arg2|...>");
                    continue;
                }

                let plugin_uuid = match parse_uuid_16(plugin_uuid_str) {
                    Some(uuid) => uuid,
                    None => {
                        LOGGER.error("Invalid UUID format");
                        continue;
                    }
                };

                let rest = rest.unwrap();
                let mut rest_parts = rest.splitn(2, ' ');
                let fn_name = match rest_parts.next() {
                    Some(f) if !f.is_empty() => f.to_string(),
                    _ => {
                        LOGGER.error("Usage: call <plugin_uuid> <fn_name> <arg1|arg2|...>");
                        continue;
                    }
                };

                let args_raw = rest_parts.next().unwrap_or("");
                let args: Vec<String> = if args_raw.is_empty() {
                    Vec::new()
                } else {
                    args_raw
                        .split('|')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                        .collect()
                };

                let id_request_to_use = alloc_request_id(id_request);

                send_interface_request(
                    &mut sock,
                    &InterfaceRequest::CallPlugin {
                        plugin_uuid,
                        fn_name,
                        args,
                    },
                    id_request_to_use,
                )?;

                LOGGER_NETWORK.debug(format!(
                    "Call request sent with request_id={id_request_to_use}"
                ));

                id_request = id_request_to_use;
            }
            _ => {
                LOGGER.error("Invalid command");
            }
        }
    }
}
