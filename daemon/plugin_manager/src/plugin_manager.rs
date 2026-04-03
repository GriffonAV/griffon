use nix::libc;
use nix::sys::socket::{AddressFamily, SockFlag, SockType, socketpair};
use std::collections::VecDeque;
use std::fs::read_dir;
use std::io;
use std::os::fd::{AsRawFd, FromRawFd, IntoRawFd};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use ipc_protocol::ipc_payload_interface::PluginInfoDto;
use ipc_protocol::ipc_payload_runner::{
    CallPayload, Message, format_uuid_bytes, recv_message, send_message,
};
use logger::Logger;

static LOGGER_PM: Logger = Logger::new("PLUGIN_MANAGER", logger::LogLevel::Debug);
static LOGGER_PM_NETWORK: Logger =
    Logger::new("PLUGIN_MANAGER-RUNNER-NETWORK", logger::LogLevel::Debug);

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
struct ManagedPlugin {
    process: Option<Child>,
    fd: Option<UnixStream>,
    enabled: bool,
    pub plugin_info: PluginInfoDto,
}

pub struct PluginManager {
    pub plugins_dir: PathBuf,
    runner_binary: PathBuf,
    plugins_list: Vec<ManagedPlugin>,
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

    let candidate = if cfg!(debug_assertions) {
        exe_dir
            .join("../")
            .join("debug")
            .join("griffonav-daemon-runner")
    } else {
        PathBuf::from("/usr/bin/griffonav-daemon-runner")
    };

    if candidate.exists() {
        Ok(candidate)
    } else {
        Err(format!("runner not found at {:?}", candidate))
    }
}

impl PluginManager {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        let runner_binary = resolve_runner_binary().expect("runner binary not found");
        let (events_tx, events_rx) = mpsc::channel();

        Self {
            plugins_dir: dir.as_ref().to_path_buf(),
            runner_binary,
            plugins_list: Vec::new(),
            next_request_id: 0,
            events_tx,
            events_rx,
            pending_events: VecDeque::new(),
        }
    }

    fn alloc_request_id(&mut self) -> u32 {
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id == 0 {
            self.next_request_id = 1;
        }
        self.next_request_id
    }

    pub fn list_plugins(&self) -> Vec<PluginInfoDto> {
        self.plugins_list
            .iter()
            .map(|p| p.plugin_info.clone())
            .collect()
    }

    pub fn scan_dir(&mut self) {
        let mut current_paths = Vec::new();
        use std::env;

        LOGGER_PM.debug(format!("Scanning dir {:?}", env::current_dir().unwrap()));
        LOGGER_PM.debug(format!(
            "Plugin dir {:?} | Exists {}",
            self.plugins_dir,
            self.plugins_dir.exists()
        ));

        for entry in read_dir(&self.plugins_dir).expect("Bad plugin directory") {
            let path = entry.unwrap().path();
            if Self::is_shared_library(&path) {
                current_paths.push(path.clone());
                self.check_plugin(&path);
            }
        }

        let mut i = 0;
        while i < self.plugins_list.len() {
            let path = PathBuf::from(&self.plugins_list[i].plugin_info.path);
            if !current_paths.contains(&path) {
                self.remove_plugin_at(i);
            } else {
                i += 1;
            }
        }
    }

    pub fn enable_plugin(&mut self, uuid: [u8; 16]) -> io::Result<()> {
        let pos = self
            .plugins_list
            .iter()
            .position(|p| p.plugin_info.plugin_uuid == uuid)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "plugin not found"))?;

        if self.plugins_list[pos].enabled {
            LOGGER_PM.debug(format!(
                "Plugin {} already enabled",
                format_uuid_bytes(&uuid)
            ));
            return Ok(());
        }

        let plugin_path = PathBuf::from(&self.plugins_list[pos].plugin_info.path);

        let (child, fd, mut info) = self.launch_runner(&plugin_path).map_err(io::Error::other)?;

        {
            let plugin = &mut self.plugins_list[pos];
            plugin.process = Some(child);
            plugin.fd = Some(fd);
            plugin.enabled = true;

            info.path = plugin.plugin_info.path.clone();
            plugin.plugin_info.pid = info.pid;
            plugin.plugin_info.name = info.name;
            plugin.plugin_info.status = true;
        }

        let handshake_res = {
            let plugin = &mut self.plugins_list[pos];
            read_plugin_messages(plugin, self.events_tx.clone())
        };

        if let Err(e) = handshake_res {
            LOGGER_PM_NETWORK.error(format!(
                "Plugin {} {:?} enable handshake failed: {e}",
                self.plugins_list[pos].plugin_info.name,
                format_uuid_bytes(&self.plugins_list[pos].plugin_info.plugin_uuid)
            ));

            if let Some(process) = self.plugins_list[pos].process.as_mut() {
                let _ = process.kill();
            }
            self.plugins_list[pos].process = None;
            self.plugins_list[pos].fd = None;
            self.plugins_list[pos].enabled = false;
            self.plugins_list[pos].plugin_info.status = true;

            return Err(e);
        }

        LOGGER_PM.info(format!(
            "Plugin {} {:?} enabled",
            self.plugins_list[pos].plugin_info.name,
            format_uuid_bytes(&self.plugins_list[pos].plugin_info.plugin_uuid)
        ));

        Ok(())
    }

    pub fn disable_plugin(&mut self, uuid: [u8; 16]) -> io::Result<()> {
        let pos = self
            .plugins_list
            .iter()
            .position(|p| p.plugin_info.plugin_uuid == uuid)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "plugin not found"))?;

        if !self.plugins_list[pos].enabled {
            LOGGER_PM.debug(format!(
                "Plugin {} already disabled",
                format_uuid_bytes(&uuid)
            ));
            return Ok(());
        }

        if let Some(process) = self.plugins_list[pos].process.as_mut() {
            process.kill()?;
        }

        self.plugins_list[pos].process = None;
        self.plugins_list[pos].fd = None;
        self.plugins_list[pos].enabled = false;
        self.plugins_list[pos].plugin_info.status = false;

        LOGGER_PM.info(format!("Plugin {} disabled", format_uuid_bytes(&uuid)));

        Ok(())
    }

    pub fn is_plugin_enabled(&self, uuid: [u8; 16]) -> bool {
        self.plugins_list
            .iter()
            .find(|p| p.plugin_info.plugin_uuid == uuid)
            .map(|p| p.enabled)
            .unwrap_or(false)
    }

    pub fn send_call(&mut self, uuid: [u8; 16], call: CallPayload) -> io::Result<u32> {
        let request_id = self.alloc_request_id();

        let msg = Message::Call {
            request_id,
            data: call,
        };

        let plugin = self
            .plugins_list
            .iter_mut()
            .find(|p| p.plugin_info.plugin_uuid == uuid)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "plugin not found"))?;

        if !plugin.enabled {
            return Err(io::Error::other("plugin is disabled"));
        }

        let fd = plugin
            .fd
            .as_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "plugin fd missing"))?;

        send_message(fd, msg)?;
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
        let path_str = path.to_string_lossy();

        let already_known = self
            .plugins_list
            .iter()
            .any(|p| p.plugin_info.path == path_str);

        if already_known {
            LOGGER_PM.debug(format!("Plugin already known {}", path.display()));
            return;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown_plugin")
            .to_string();

        let managed = ManagedPlugin {
            process: None,
            fd: None,
            enabled: false,
            plugin_info: PluginInfoDto {
                pid: 0,
                name: file_name,
                plugin_uuid: [0; 16],
                status: true,
                path: path.display().to_string(),
                functions: Vec::new(),
            },
        };

        self.plugins_list.push(managed);
        LOGGER_PM.info(format!("New plugin discovered {}", path.display()));

        if let Some(last) = self.plugins_list.last() {
            let uuid = last.plugin_info.plugin_uuid;
            if uuid == [0; 16] {
                let index = self.plugins_list.len() - 1;
                let plugin_path = PathBuf::from(&self.plugins_list[index].plugin_info.path);

                let enable_res = {
                    let (child, fd, info) =
                        match self.launch_runner(&plugin_path).map_err(io::Error::other) {
                            Ok(v) => v,
                            Err(e) => {
                                LOGGER_PM.error(format!(
                                    "Failed to launch runner: {}: {e}",
                                    plugin_path.display()
                                ));
                                return;
                            }
                        };

                    self.plugins_list[index].process = Some(child);
                    self.plugins_list[index].fd = Some(fd);
                    self.plugins_list[index].enabled = true;
                    self.plugins_list[index].plugin_info.pid = info.pid;
                    self.plugins_list[index].plugin_info.name = info.name;

                    read_plugin_messages(&mut self.plugins_list[index], self.events_tx.clone())
                };

                if let Err(e) = enable_res {
                    LOGGER_PM_NETWORK.error(format!(
                        "Plugin {} handshake failed: {e}",
                        self.plugins_list[index].plugin_info.name
                    ));

                    if let Some(process) = self.plugins_list[index].process.as_mut() {
                        let _ = process.kill();
                    }

                    self.plugins_list[index].process = None;
                    self.plugins_list[index].fd = None;
                    self.plugins_list[index].enabled = false;
                }
            }
        }
    }

    fn remove_plugin_at(&mut self, index: usize) {
        let mut plugin = self.plugins_list.remove(index);

        LOGGER_PM.info(format!("Plugin {} removed", plugin.plugin_info.name));

        if plugin.enabled
            && let Some(process) = plugin.process.as_mut()
            && let Err(e) = process.kill()
        {
            LOGGER_PM.error(format!(
                "Failed to kill plugin {}: {}",
                plugin.plugin_info.name, e
            ));
        }
    }

    fn is_shared_library(path: &Path) -> bool {
        path.is_file() && path.extension().is_some_and(|ext| ext == "so")
    }

    fn launch_runner(
        &self,
        plugin_path: &Path,
    ) -> Result<(Child, UnixStream, PluginInfoDto), String> {
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

        let core_stream = unsafe { UnixStream::from_raw_fd(core_fd.into_raw_fd()) };

        let plugin_info = PluginInfoDto {
            pid: child.id(),
            name,
            status: true,
            plugin_uuid: [0; 16],
            path: plugin_path.display().to_string(),
            functions: Vec::new(),
        };

        LOGGER_PM.info(format!(
            "Plugin {} ({}) has been started.",
            plugin_info.name, plugin_info.pid
        ));

        Ok((child, core_stream, plugin_info))
    }
}

fn read_plugin_messages(
    plugin: &mut ManagedPlugin,
    events_tx: Sender<PluginEvent>,
) -> io::Result<()> {
    let fd = plugin
        .fd
        .as_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::BrokenPipe, "plugin fd missing"))?;

    let mut fd_clone = fd
        .try_clone()
        .map_err(|e| io::Error::other(format!("Failed to clone fd: {e}")))?;

    let pid = plugin.process.as_ref().map(|p| p.id()).unwrap_or(0);

    send_message(&mut fd_clone, Message::Hello)?;

    LOGGER_PM_NETWORK.debug("Send Hello done");

    let hello_ok = match recv_message(&mut fd_clone) {
        Ok(message) => match message {
            Message::HelloOk(p) => {
                LOGGER_PM_NETWORK.debug(format!(
                    "HelloOk received from {:?} ({}) function = {:?}",
                    format_uuid_bytes(&p.uuid),
                    p.name,
                    p.functions
                ));
                p
            }
            other => {
                LOGGER_PM_NETWORK.error(format!(
                    "received unexpected message for hello: {:?}",
                    other
                ));
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "expected HelloOk",
                ));
            }
        },
        Err(e) => {
            LOGGER_PM_NETWORK.error(format!("recv_message failed: {}", e));
            return Err(e);
        }
    };

    plugin.plugin_info.name = hello_ok.name;
    plugin.plugin_info.functions = hello_ok.functions;
    plugin.plugin_info.plugin_uuid = hello_ok.uuid;
    plugin.plugin_info.pid = pid;

    let name = plugin.plugin_info.name.clone();
    let plugin_uuid = plugin.plugin_info.plugin_uuid;

    LOGGER_PM_NETWORK.info(format!(
        "Plugin {name} {:?} ({pid}) handshake OK, functions={:?}",
        format_uuid_bytes(&plugin_uuid),
        plugin.plugin_info.functions
    ));

    std::thread::spawn(move || {
        loop {
            let msg = match recv_message(&mut fd_clone) {
                Ok(m) => m,
                Err(e) => {
                    LOGGER_PM_NETWORK.info(format!("{name} ({pid}) closed / recv error: {e}"));
                    let _ = events_tx.send(PluginEvent::Closed {
                        pid,
                        reason: e.to_string(),
                    });
                    break;
                }
            };

            match msg {
                Message::Result { request_id, data } => {
                    LOGGER_PM_NETWORK.info(format!(
                        "Plugin {name} ({pid}) RESULT id={request_id} ok={} output={}",
                        data.ok, data.output
                    ));
                    let _ = events_tx.send(PluginEvent::Result {
                        pid,
                        request_id,
                        ok: data.ok,
                        output: data.output,
                    });
                }
                Message::Error { request_id, data } => {
                    LOGGER_PM_NETWORK.error(format!(
                        "Plugin {name} ({pid}) ERROR id={request_id} code={} message={}",
                        data.code, data.message
                    ));
                    let _ = events_tx.send(PluginEvent::Error {
                        pid,
                        request_id,
                        message: data.message,
                    });
                }
                Message::Heartbeat => {
                    LOGGER_PM_NETWORK.debug(format!("Plugin {name} ({pid}) HEARTBEAT OK"));
                    let _ = events_tx.send(PluginEvent::Heartbeat { pid });
                }
                other => {
                    LOGGER_PM_NETWORK.info(format!(
                        "Plugin {name} ({pid}) UNKNOWN message received : {:?}",
                        other
                    ));
                }
            }
        }
    });

    Ok(())
}
