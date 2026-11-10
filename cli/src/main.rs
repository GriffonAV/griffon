use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::thread;

use ipc_protocol::ipc_payload_interface::{
    InterfaceRequest, InterfaceResponse, alloc_request_id, format_uuid_bytes, parse_uuid_16,
    recv_interface_response, send_interface_request,
};
use logger::{LogLevel, Logger};

static LOGGER: Logger = Logger::new("CLI", LogLevel::Info, None); // Bumped to Info to reduce typing interruptions
// static LOGGER_NETWORK: Logger = Logger::new("CLI-NETWORK", LogLevel::Error, None);

const DAEMON_SOCK_PATH: &str = if cfg!(debug_assertions) {
    "./griffon.sock"
} else {
    "/run/griffon/griffon.sock"
};

fn print_help() {
    println!("\n=== Griffon CLI ===");
    println!("\nCommands:");
    println!("  help");
    println!("      Show this help message");
    println!("  refresh");
    println!("      Refresh and display the plugin list from the daemon");
    println!("  switch_status <plugin_uuid>");
    println!("      Enable or disable a plugin");
    println!("  switch_notification <plugin_uuid>");
    println!("      Enable or disable notifications for a plugin");
    println!("  call <plugin_uuid> <fn_name> [json_payload]");
    println!("      Call a plugin function with an optional JSON payload");
    println!("\nExamples:");
    println!("  call 550e8400-e29b-41d4-a716-446655440000 scan {{\"target\":\"127.0.0.1\"}}");
    println!("  call 550e8400-e29b-41d4-a716-446655440000 get_containers");
    println!("  exit | quit\n");
}

fn start_reader_thread(mut read_sock: UnixStream) {
    thread::spawn(move || {
        loop {
            match recv_interface_response(&mut read_sock) {
                Ok(resp) => match resp {
                    InterfaceResponse::Ok { request_id: _ } => {}
                    InterfaceResponse::CallAccepted { request_id: _ } => {}
                    InterfaceResponse::Plugins { plugins, .. } => {
                        println!(
                            "\n{0: <38} | {1: <15} | {2: <10} | FUNCTIONS",
                            "UUID", "NAME", "STATUS"
                        );
                        println!("{:-<38}-+-{:-<15}-+-{:-<10}-+-{:-<40}", "", "", "", "");

                        for plugin in plugins {
                            let uuid = format_uuid_bytes(&plugin.plugin_uuid);
                            let status = if plugin.status { "Active" } else { "Disabled" };
                            let funcs = plugin.functions.join(", ");

                            println!(
                                "{0: <38} | {1: <15} | {2: <10} | {3}",
                                uuid, plugin.name, status, funcs
                            );
                        }
                        println!();
                        print!("$> ");
                        let _ = io::stdout().flush();
                    }
                    InterfaceResponse::Error {
                        request_id,
                        message,
                    } => {
                        println!("\n[ERROR] Request {} failed: {}", request_id, message);
                        print!("$> ");
                        let _ = io::stdout().flush();
                    }
                    InterfaceResponse::CallResult {
                        request_id,
                        ok,
                        output,
                    } => {
                        let status = if ok { "SUCCESS" } else { "FAILED" };
                        println!("\n[CALL RESULT {}] {}: \n{}", request_id, status, output);
                        print!("$> ");
                        let _ = io::stdout().flush();
                    }
                    InterfaceResponse::SwitchDone { request_id, enable } => {
                        println!("\n[SWITCH {}] Plugin enabled: {}", request_id, enable);
                        print!("$> ");
                        let _ = io::stdout().flush();
                    }
                },
                Err(e) => {
                    println!("\n[FATAL] Connection to daemon lost: {}", e);
                    std::process::exit(1);
                }
            }
        }
    });
}

fn main() -> io::Result<()> {
    let mut id_request: u32 = 0;

    let mut sock = UnixStream::connect(DAEMON_SOCK_PATH).unwrap_or_else(|e| {
        LOGGER.error(format!(
            "Failed to connect to daemon at {}: {}",
            DAEMON_SOCK_PATH, e
        ));
        std::process::exit(1);
    });

    println!("Connected to Griffon Daemon. Type 'help' for commands.");

    let read_sock = sock.try_clone()?;
    start_reader_thread(read_sock);

    loop {
        // The prompt is back!
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

        // Split into at most 4 parts: command, uuid, function, and the rest (JSON payload)
        let mut parts = trimmed.splitn(4, char::is_whitespace);
        let cmd = parts.next().unwrap();

        match cmd {
            "help" | "--help" | "-h" => {
                print_help();
            }
            "refresh" => {
                let id_request_to_use = alloc_request_id(id_request);
                send_interface_request(
                    &mut sock,
                    &InterfaceRequest::RefreshPlugins {},
                    id_request_to_use,
                )?;
                id_request = id_request_to_use;
            }
            "exit" | "quit" => {
                return Ok(());
            }
            "switch_status" | "switch_notification" => {
                let plugin_uuid_str = parts.next();
                if plugin_uuid_str.is_none() {
                    LOGGER.error(format!("Usage: {} <plugin_uuid>", cmd));
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
                let req = if cmd == "switch_status" {
                    InterfaceRequest::SwitchStatusPlugin { plugin_uuid }
                } else {
                    InterfaceRequest::SwitchStatusNotification { plugin_uuid }
                };

                send_interface_request(&mut sock, &req, id_request_to_use)?;
                id_request = id_request_to_use;
            }
            "call" => {
                let plugin_uuid_str = parts.next();
                let fn_name = parts.next();
                let json_payload = parts.next().unwrap_or("").trim(); // Grab the rest of the line as a single string

                if plugin_uuid_str.is_none() || fn_name.is_none() {
                    LOGGER.error("Usage: call <plugin_uuid> <fn_name> [json_payload]");
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

                // Pack the single JSON payload as the first and only argument for the daemon
                let args = if json_payload.is_empty() {
                    vec![]
                } else {
                    vec![json_payload.to_string()]
                };

                send_interface_request(
                    &mut sock,
                    &InterfaceRequest::CallPlugin {
                        plugin_uuid,
                        fn_name: fn_name.unwrap().to_string(),
                        args,
                    },
                    id_request_to_use,
                )?;

                id_request = id_request_to_use;
            }
            _ => {
                LOGGER.error("Invalid command. Type 'help' for a list of commands.");
            }
        }
    }
}
