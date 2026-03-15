use std::io;
use std::io::Write;
use plugin_manager::{PluginManager, LogLevel};
use ipc_protocol;

static PLUGIN_DIR_PATH: &str = "./plugins";

fn drain_events(pm: &mut PluginManager) {
    while let Some(ev) = pm.try_recv_event() {
        println!("[CORE] EVENT: {:?}", ev);
    }
}

fn main() {
    let mut pm = PluginManager::new(PLUGIN_DIR_PATH, LogLevel::Info);

    pm.scan_dir();
    pm.list_plugins();

    loop {
        print!("$> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("[CORE](ERROR) Read Error");
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
                let plugins = pm.list_plugins();
                for plugin in plugins {
                    println!("- PID: {} | NAME: {} | PATH: {} | FUNCTIONS: {:?}", plugin.pid, plugin.name, plugin.path.display(), plugin.functions);
                }
            }
            "refresh" => {
                pm.scan_dir();
            }
            "exit" | "quit" => {
                break;
            }
            "restart" => {
                let pid_str = parts.next();

                if pid_str.is_none() {
                    println!("[CORE](INPUT ERROR) Usage: restart <PID>");
                    continue;
                }

                let pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        println!("[CORE](INPUT ERROR) Invalid PID: {pid_str:?}");
                        continue;
                    }
                };
                pm.restart_plugin(pid);
            }
            "kill" => {
                let pid_str = parts.next();

                if pid_str.is_none() {
                    println!("[CORE](INPUT ERROR) Usage: kill <PID>");
                    continue;
                }

                let pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        println!("[CORE](INPUT ERROR) Invalid PID: {pid_str:?}");
                        continue;
                    }
                };
                pm.kill_plugin(pid);
            }

            
            "call" => {
                let pid_str = parts.next();
                let rest = parts.next();

                if pid_str.is_none() || rest.is_none() {
                    println!("[CORE](INPUT ERROR) Usage: call <PID> <fn_name> <arg1|arg2|...>");
                    continue;
                }

                let pid: u32 = match pid_str.unwrap().parse() {
                    Ok(p) => p,
                    Err(_) => {
                        println!("[CORE](INPUT ERROR) Invalid PID: {pid_str:?}");
                        continue;
                    }
                };

                let rest = rest.unwrap();
                let mut rest_parts = rest.splitn(2, ' ');
                let fn_name = match rest_parts.next() {
                    Some(f) if !f.is_empty() => f.to_string(),
                    _ => {
                        println!("[CORE](INPUT ERROR) Usage: call <PID> <fn_name> <arg1|arg2|...>");
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

                let call_payload = ipc_protocol::ipc_payload::CallPayload { fn_name, args };

                match pm.send_call(pid, call_payload) {
                    Ok(req_id) => {
                        println!("[CORE] CALL sent (request_id={req_id})");
                        match pm.wait_for_response(req_id) {
                            Ok(ev) => println!("[CORE] RESPONSE: {:?}", ev),
                            Err(e) => eprintln!("[CORE](ERROR) wait_for_response failed: {e}"),
                        }
                    },
                    Err(e) => println!("[CORE](ERROR) Failed to send CALL: {e}"),
                }
            }

            "" => {}
            other => {
                println!("Unknow CMD: {}", other);
            }
        }
    }
}