use nix::libc;
use nix::sys::socket::{AddressFamily, SockFlag, SockType, socketpair};
use std::fs::read_dir;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::collections::VecDeque;
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use ipc_protocol::ipc_payload::{CallPayload, Message, recv_message, send_message};


#[derive(Debug, Clone)]
pub enum PluginEvent {
    Result {
        pid: u32,
        request_id: u32,
        ok: bool,
        output: String,
    },
    Error {
        pid: u32,
        request_id: u32,
        message: String,
    },
    Heartbeat {
        pid: u32,
    },
    Closed {
        pid: u32,
        reason: String,
    },
}

#[derive(Debug)]
struct RunningPlugin {
    process: Child,
    fd: std::os::unix::net::UnixStream,
    pub plugin_info: PluginInfo,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub pid: u32,
    pub name: String,
    pub path: PathBuf,
    pub functions: Vec<String>,
}

#[derive(Debug, PartialEq, PartialOrd, Copy, Clone)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct PluginManager {
    pub plugins_dir: PathBuf,
    runner_binary: PathBuf,
    plugins_list: Vec<RunningPlugin>,
    pub log_level: LogLevel,
    next_request_id: u32,

    events_tx: Sender<PluginEvent>,
    events_rx: Receiver<PluginEvent>,
    pending_events: VecDeque<PluginEvent>,
}

const RUNNER_ENV: &str = "GRIFFON_RUNNER_BINARY";

fn resolve_runner_binary() -> Result<PathBuf, String> {
    if let Ok(p) = std::env::var(RUNNER_ENV) {
        return Ok(PathBuf::from(p));
    }

    let exe = std::env::current_exe().map_err(|e| format!("current_exe: {e}"))?;
    let exe_dir = exe.parent().ok_or("exe has no parent")?;

    let profile = if cfg!(debug_assertions) { "debug" } else { "release" };
    let candidate = exe_dir.join("../..").join("target").join(profile).join("runner");

    if candidate.exists() { Ok(candidate) }
    else { Err(format!("runner not found at {:?}", candidate)) }
}

impl PluginManager {
    pub fn new<P: AsRef<Path>>(dir: P, log_level: LogLevel) -> Self {
        let runner_binary = resolve_runner_binary().expect("runner binary not found");
        let (events_tx, events_rx) = mpsc::channel();
        Self {
            plugins_dir: dir.as_ref().to_path_buf(),
            runner_binary,
            plugins_list: Vec::new(),
            log_level,
            next_request_id: 0,
            events_tx,
            events_rx,
            pending_events: VecDeque::new(),
        }
    }

    fn log(&self, level: LogLevel, msg: &str) {
        if level >= self.log_level {
            match level {
                LogLevel::Debug => println!("[PLUGIN_MANAGER](DEBUG) {msg}"),
                LogLevel::Info => println!("[PLUGIN_MANAGER](INFO) {msg}"),
                LogLevel::Warn => println!("[PLUGIN_MANAGER](WARN) {msg}"),
                LogLevel::Error => eprintln!("[PLUGIN_MANAGER](ERROR) {msg}"),
            }
        }
    }

    fn alloc_request_id(&mut self) -> u32 {
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id == 0 {
            self.next_request_id = 1;
        }
        self.next_request_id
    }

    pub fn list_plugins(&self) -> Vec<PluginInfo> {
        self.plugins_list
            .iter()
            .map(|p| p.plugin_info.clone())
            .collect()
    }

    pub fn scan_dir(&mut self) {
        let mut current_paths = Vec::new();
        use std::env;

        println!("[debug] cwd = {:?}", env::current_dir().unwrap());
        println!("[debug] plugins_dir = {:?}", self.plugins_dir);
        println!("[debug] plugins_dir exists = {}", self.plugins_dir.exists());

        for entry in read_dir(&self.plugins_dir).expect("Bad plugin directory") {
            let path = entry.unwrap().path();
            if Self::is_shared_library(&path) {
                current_paths.push(path.clone());
                self.check_plugin(&path);
            }
        }

        let mut i = 0;
        while i < self.plugins_list.len() {
            if !current_paths.contains(&self.plugins_list[i].plugin_info.path) {
                self.remove_plugin_at(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn restart_plugin(&mut self, pid: u32) {
        if let Some(plugin) = self.plugins_list.iter().find(|p| p.process.id() == pid) {
            let path = plugin.plugin_info.path.clone();
            self.kill_plugin(pid);
            self.check_plugin(&path);
            self.log(LogLevel::Debug, &format!("Plugin PID {pid} restarted"));
        } else {
            self.log(LogLevel::Error, &format!("No plugin found with PID {pid}"));
        }
    }

    pub fn kill_plugin(&mut self, pid: u32) {
        if let Some(pos) = self.plugins_list.iter().position(|p| p.process.id() == pid) {
            let mut plugin = self.plugins_list.remove(pos);
            match plugin.process.kill() {
                Ok(_) => self.log(LogLevel::Debug, &format!("Plugin PID {pid} killed")),
                Err(e) => self.log(
                    LogLevel::Error,
                    &format!("Failed to kill plugin PID {pid}: {e}"),
                ),
            }
        } else {
            self.log(LogLevel::Warn, &format!("No plugin found with PID {pid}"));
        }
    }

    pub fn send_call(&mut self, pid: u32, call: CallPayload) -> io::Result<u32> {
        let request_id = self.alloc_request_id();

        let msg = Message::Call {
            request_id,
            data: call,
        };

        let plugin = self
            .plugins_list
            .iter_mut()
            .find(|p| p.process.id() == pid)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "plugin pid not found"))?;

        send_message(&mut plugin.fd, msg)?;

        Ok(request_id)
    }

    pub fn try_recv_event(&mut self) -> Option<PluginEvent> {
        if let Some(ev) = self.pending_events.pop_front() {
            return Some(ev);
        }

        match self.events_rx.try_recv() {
            Ok(ev) => Some(ev),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn wait_for_response(&mut self, request_id: u32) -> io::Result<PluginEvent> {
        if let Some(pos) = self.pending_events.iter().position(|ev| {
            matches!(
            ev,
            PluginEvent::Result { request_id: rid, .. }
                | PluginEvent::Error { request_id: rid, .. }
                if *rid == request_id
        )
        }) {
            return Ok(self.pending_events.remove(pos).unwrap());
        }

        loop {
            let ev = self.events_rx.recv().map_err(|_| {
                io::Error::new(io::ErrorKind::BrokenPipe, "event channel disconnected")
            })?;

            let is_match = matches!(
            ev,
            PluginEvent::Result { request_id: rid, .. }
                | PluginEvent::Error { request_id: rid, .. }
                if rid == request_id
        );

            if is_match {
                return Ok(ev);
            }

            self.pending_events.push_back(ev);
        }
    }

    fn check_plugin(&mut self, path: &Path) {
        let already_running = self.plugins_list.iter().any(|p| p.plugin_info.path == path);
        if already_running {
            self.log(
                LogLevel::Debug,
                &format!("Plugin already running {}", path.display()),
            );
            return;
        }

        let running = match Self::launch_runner(self, path) {
            Ok(r) => r,
            Err(msg) => {
                self.log(
                    LogLevel::Error,
                    &format!(
                        "Failed to launch runner for plugin {}: {msg}",
                        path.display()
                    ),
                );
                return;
            }
        };

        self.log(
            LogLevel::Debug,
            &format!("New plugin found {}", path.display()),
        );

        self.plugins_list.push(running);

        let handshake_res = {
            let last = self.plugins_list.last_mut().unwrap();
            read_plugin_messages(last, self.log_level, self.events_tx.clone())
        };

        if let Err(e) = handshake_res {
            let mut bad = self.plugins_list.pop().unwrap();
            self.log(
                LogLevel::Error,
                &format!(
                    "Handshake failed for plugin {} (pid={}): {e}",
                    bad.plugin_info.name, bad.plugin_info.pid
                ),
            );
            let _ = bad.process.kill();
        }
    }

    fn remove_plugin_at(&mut self, index: usize) {
        let mut plugin = self.plugins_list.remove(index);
        self.log(
            LogLevel::Debug,
            &format!("Plugin removed {}", plugin.plugin_info.name),
        );
        if let Err(e) = plugin.process.kill() {
            self.log(LogLevel::Error, &format!("Failed to kill plugin PID : {e}"));
        }
    }

    fn is_shared_library(path: &Path) -> bool {
        path.is_file() && path.extension().map_or(false, |ext| ext == "so")
    }

    fn launch_runner(&self, plugin_path: &Path) -> Result<RunningPlugin, String> {
        let path = plugin_path.display().to_string();
        let tmp_name = plugin_path.display().to_string();
        let name = tmp_name.rsplit('/').next().unwrap().to_string();

        let (core_fd, runner_fd) = socketpair(
            AddressFamily::Unix,
            SockType::Stream,
            None,
            SockFlag::empty(),
        )
        .map_err(|e| format!("socketpair failed: {e}"))?;

        let mut cmd = Command::new(&self.runner_binary);
        cmd.arg(path);

        unsafe {
            cmd.pre_exec(move || {
                if libc::dup2(runner_fd.as_raw_fd(), 3) == -1 {
                    return Err(io::Error::last_os_error());
                }
                Ok(())
            });
        }

        let child = cmd
            .spawn()
            .map_err(|e| format!("failed to spawn runner: {e}"))?;
        let core_stream =
            unsafe { std::os::unix::net::UnixStream::from_raw_fd(core_fd.into_raw_fd()) };

        let plugininfo = PluginInfo {
            pid: child.id(),
            name,
            path: plugin_path.to_path_buf(),
            functions: Vec::new(),
        };

        self.log(
            LogLevel::Info,
            &format!(
                "Plugin {} ({}) has been started.",
                plugininfo.name, plugininfo.pid
            ),
        );

        Ok(RunningPlugin {
            process: child,
            fd: core_stream,
            plugin_info: plugininfo,
        })
    }
}

fn read_plugin_messages(plugin: &mut RunningPlugin, log_level: LogLevel, events_tx: Sender<PluginEvent>) -> io::Result<()> {
    let mut fd_clone = plugin
        .fd
        .try_clone()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Failed to clone fd: {e}")))?;

    let pid = plugin.process.id();

    send_message(&mut fd_clone, Message::Hello)?;

    let hello_ok = match recv_message(&mut fd_clone)? {
        Message::HelloOk(p) => p,
        other => {
            if log_level >= LogLevel::Error {
                eprintln!(
                    "[PLUGIN_MANAGER](ERROR) Plugin ({pid}) Expected HelloOk, got: {:?}",
                    other
                );
            }
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "expected HelloOk",
            ));
        }
    };

    plugin.plugin_info.name = hello_ok.name;
    plugin.plugin_info.functions = hello_ok.functions;

    let name = plugin.plugin_info.name.clone();

    if log_level >= LogLevel::Info {
        println!(
            "[PLUGIN_MANAGER](INFO) Plugin {name} ({pid}) handshake OK, functions={:?}",
            plugin.plugin_info.functions
        );
    }

    std::thread::spawn(move || {
        loop {
            let msg = match recv_message(&mut fd_clone) {
                Ok(m) => m,
                Err(e) => {
                    if log_level >= LogLevel::Info {
                        println!(
                            "[PLUGIN_MANAGER](INFO) Plugin {name} ({pid}) closed / recv error: {e}"
                        );
                    }
                    let _ = events_tx.send(PluginEvent::Closed {
                        pid,
                        reason: e.to_string(),
                    });
                    break;
                }
            };

            match msg {
                Message::Result { request_id, data } => {
                    if log_level >= LogLevel::Info {
                        println!(
                            "[PLUGIN_MANAGER](INFO) Plugin {name} ({pid}) RESULT id={request_id} ok={} output={}",
                            data.ok, data.output
                        );
                    }
                    let _ = events_tx.send(PluginEvent::Result {
                        pid,
                        request_id,
                        ok: data.ok,
                        output: data.output,
                    });
                }
                Message::Error { request_id, data } => {
                    if log_level >= LogLevel::Error {
                        eprintln!(
                            "[PLUGIN_MANAGER](ERROR) Plugin {name} ({pid}) ERROR id={request_id} code={} message={}",
                            data.code, data.message
                        );
                    }
                    let _ = events_tx.send(PluginEvent::Error {
                        pid,
                        request_id,
                        message: data.message,
                    });
                }
                Message::Heartbeat => {
                    if log_level >= LogLevel::Debug {
                        println!("[PLUGIN_MANAGER](DEBUG) Plugin {name} ({pid}) HEARTBEAT");
                    }
                }
                other => {
                    if log_level >= LogLevel::Debug {
                        println!(
                            "[PLUGIN_MANAGER](DEBUG) Plugin {name} ({pid}) MSG: {:?}",
                            other
                        );
                    }
                }
            }
        }
    });

    Ok(())
}
