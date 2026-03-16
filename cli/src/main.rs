use std::io;
use std::io::Write;
use std::os::unix::net::UnixStream;
use uuid::Uuid;

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest,
    InterfaceResponse,
    send_interface_request,
    recv_interface_response,
};
use logger::{LogLevel, Logger};

static LOGGER: Logger = Logger::new("CLI", LogLevel::Debug);
static LOGGER_NETWORK: Logger = Logger::new("CLI-NETWORK", LogLevel::Debug);
const DAEMON_SOCK_PATH: &str = "/run/griffon/daemon.sock";

fn main() -> io::Result<()> {
    LOGGER_NETWORK.debug("Client try connected");
    let mut sock = UnixStream::connect(DAEMON_SOCK_PATH)?;
    LOGGER_NETWORK.info("Client connected");

    send_interface_request(&mut sock, &InterfaceRequest::Ping)?;
    LOGGER_NETWORK.debug("Ping sent");

    let resp = recv_interface_response(&mut sock)?;
    LOGGER_NETWORK.debug(format!("Response received: {:?}", resp));

    match resp {
        InterfaceResponse::Pong => {
            LOGGER_NETWORK.info("Pong received");
        }
        other => {
            LOGGER_NETWORK.error(format!("Unexpected response: {:?}", other));
        }
    }

    loop {
        print!("$> ");
        io::stdout().flush()?;
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
            "info" => {
                //TODO
                LOGGER.debug("INFO");
                let _plugins = [""];
                for _plugin in _plugins {
                    // println!("- PID: {} | NAME: {} | PATH: {} | FUNCTIONS: {:?}", plugin.pid, plugin.name, plugin.path.display(), plugin.functions);
                }
            }
            "refresh" => {
                LOGGER.debug("REFRESH");
                // TODO : Refresh list
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

                let pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        LOGGER.error("Invalid PID");
                        continue;
                    }
                };
                // TODO : Restart plugin
            }
            "kill" => {
                LOGGER.debug("KILL");
                let pid_str = parts.next();

                if pid_str.is_none() {
                    LOGGER.error("Usage: kill <pid>");
                    continue;
                }

                let pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        LOGGER.error("Invalid PID");
                        continue;
                    }
                };
                // TODO : KILL
            }


            "call" => {
                LOGGER.debug("CALL");
                let plugin_uuid_str = parts.next();
                let rest = parts.next();

                if plugin_uuid_str.is_none() || rest.is_none() {
                    LOGGER.error("Usage: call <plugin_uuid> <fn_name> <arg1|arg2|...>");
                    continue;
                }

                let plugin_uuid: [u8; 16] = match Uuid::parse_str(plugin_uuid_str.unwrap()) {
                    Ok(uuid) => *uuid.as_bytes(),
                    Err(_) => {
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


                send_interface_request(&mut sock, &InterfaceRequest::CallPlugin {plugin_uuid, fn_name, args})?;
                // TODO : Send call to the daemon_core    Ok(())


              /*  match pm.send_call(pid, call_payload) {
                    Ok(req_id) => {
                        println!("[CORE] CALL sent (request_id={req_id})");
                        match pm.wait_for_response(req_id) {
                            Ok(ev) => println!("[CORE] RESPONSE: {:?}", ev),
                            Err(e) => eprintln!("[CORE](ERROR) wait_for_response failed: {e}"),
                        }
                    },
                    Err(e) => println!("[CORE](ERROR) Failed to send CALL: {e}"),
                }*/
            }

            "" => {}
            other => {
                LOGGER.error("Invalid command");
            }
        }
    }
}