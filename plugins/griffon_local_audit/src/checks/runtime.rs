use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::path::{Path, PathBuf};

use crate::{AuditState, Finding, InventoryItem};

use super::{mode_string, read_limited};

const MAX_PROCESSES: usize = 4096;
const MAX_CAPABILITY_PROCESS_GROUPS: usize = 64;
const MAX_SYSTEMD_ENTRIES: usize = 2048;
const MAX_UNIX_SOCKETS: usize = 512;

pub(super) fn run(state: &mut AuditState) {
    check_processes(Path::new("/proc"), state);
    check_systemd_units(state);
    check_unix_sockets(Path::new("/proc/net/unix"), state);
}

fn check_processes(proc_root: &Path, state: &mut AuditState) {
    let Ok(entries) = fs::read_dir(proc_root) else {
        state.diagnostic(format!(
            "Could not list {} for process checks.",
            proc_root.display()
        ));
        return;
    };
    let mut pids: Vec<u32> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().to_string_lossy().parse().ok())
        .collect();
    pids.sort_unstable();
    pids.truncate(MAX_PROCESSES);

    let mut reported_paths = HashSet::new();
    let mut capability_processes: BTreeMap<(String, u32, u64, u64), (usize, u32)> = BTreeMap::new();
    for pid in pids {
        let directory = proc_root.join(pid.to_string());
        let Ok(status) = read_limited(&directory.join("status"), 32 * 1024) else {
            continue;
        };
        let Some(process) = parse_process_status(pid, &status) else {
            continue;
        };
        if process.is_kernel_thread {
            continue;
        }

        if process.cap_eff != 0 || process.cap_amb != 0 {
            let entry = capability_processes
                .entry((
                    process.name.clone(),
                    process.uid,
                    process.cap_eff,
                    process.cap_amb,
                ))
                .or_insert((0, pid));
            entry.0 += 1;
        }

        if process.uid != 0 {
            continue;
        }

        let Ok(executable_link) = fs::read_link(directory.join("exe")) else {
            continue;
        };
        let display = executable_link.to_string_lossy().into_owned();
        let deleted = display.ends_with(" (deleted)");
        let executable = if deleted {
            PathBuf::from(display.trim_end_matches(" (deleted)"))
        } else {
            executable_link
        };

        if deleted {
            state.push(Finding::new(
                "Medium",
                "Processes",
                format!("Root process {} runs a deleted executable", process.name),
                format!("PID {pid}: {display}"),
                "Identify the owning package or deployment and restart the process from a verified executable.",
            ));
        }

        if !reported_paths.insert(executable.clone()) {
            continue;
        }
        inspect_root_process_executable(&process, &executable, state);
    }

    for ((name, uid, cap_eff, cap_amb), (count, example_pid)) in capability_processes
        .into_iter()
        .take(MAX_CAPABILITY_PROCESS_GROUPS)
    {
        state.push_inventory(InventoryItem::new(
            "Processes",
            if count == 1 {
                format!("{name} (PID {example_pid})")
            } else {
                format!("{name} ({count} processes, e.g. PID {example_pid})")
            },
            format!("uid={uid}, effective capabilities=0x{cap_eff:x}, ambient=0x{cap_amb:x}"),
        ));
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ProcessStatus {
    pid: u32,
    name: String,
    uid: u32,
    cap_eff: u64,
    cap_amb: u64,
    is_kernel_thread: bool,
}

fn parse_process_status(pid: u32, contents: &str) -> Option<ProcessStatus> {
    let mut name = None;
    let mut uid = None;
    let mut cap_eff = 0;
    let mut cap_amb = 0;
    let mut is_kernel_thread = false;
    for line in contents.lines().take(256) {
        if let Some(value) = line.strip_prefix("Name:") {
            name = Some(value.trim().to_string());
        } else if let Some(value) = line.strip_prefix("Uid:") {
            uid = value.split_whitespace().next()?.parse().ok();
        } else if let Some(value) = line.strip_prefix("CapEff:") {
            cap_eff = u64::from_str_radix(value.trim(), 16).unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("CapAmb:") {
            cap_amb = u64::from_str_radix(value.trim(), 16).unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("Kthread:") {
            is_kernel_thread = value.trim() == "1";
        }
    }
    Some(ProcessStatus {
        pid,
        name: name?,
        uid: uid?,
        cap_eff,
        cap_amb,
        is_kernel_thread,
    })
}

fn inspect_root_process_executable(
    process: &ProcessStatus,
    executable: &Path,
    state: &mut AuditState,
) {
    let Ok(metadata) = fs::metadata(executable) else {
        return;
    };
    let mode = metadata.mode() & 0o7777;
    if metadata.uid() != 0 {
        state.push(Finding::new(
            "High",
            "Processes",
            format!("Root process {} executes a non-root-owned file", process.name),
            format!(
                "PID {}: {} owner UID {}, mode {mode:04o}",
                process.pid,
                executable.display(),
                metadata.uid()
            ),
            "Verify the executable provenance and require ownership by a trusted privileged account.",
        ));
    }
    if mode & 0o022 != 0 {
        state.push(Finding::new(
            if mode & 0o002 != 0 { "Critical" } else { "High" },
            "Processes",
            format!("Root process executable is writable: {}", executable.display()),
            format!("PID {}, mode {mode:04o}", process.pid),
            "Remove group/other write permissions and verify the executable against its package source.",
        ));
    }

    let Some(parent) = executable.parent() else {
        return;
    };
    let Ok(parent_metadata) = fs::metadata(parent) else {
        return;
    };
    let parent_mode = parent_metadata.mode() & 0o7777;
    let sticky_protects_entry = parent_mode & 0o1000 != 0 && metadata.uid() == 0;
    if parent_mode & 0o022 != 0 && !sticky_protects_entry {
        state.push(Finding::new(
            if parent_mode & 0o002 != 0 {
                "Critical"
            } else {
                "High"
            },
            "Processes",
            "Root process executable has a writable parent directory",
            format!(
                "PID {}: {} mode {}",
                process.pid,
                parent.display(),
                mode_string(&parent_metadata)
            ),
            "Restrict writes to the executable's parent directory and verify directory ownership.",
        ));
    }
}

fn check_systemd_units(state: &mut AuditState) {
    let roots = [
        Path::new("/etc/systemd/system"),
        Path::new("/run/systemd/system"),
        Path::new("/usr/lib/systemd/system"),
        Path::new("/lib/systemd/system"),
    ];
    let mut canonical_roots = HashSet::new();
    let mut inspected = 0usize;
    let mut reported_execs = HashSet::new();

    for root in roots {
        let Ok(root) = root.canonicalize() else {
            continue;
        };
        if !canonical_roots.insert(root.clone()) {
            continue;
        }
        inspect_systemd_directory(&root, state);
        let Ok(entries) = fs::read_dir(&root) else {
            continue;
        };
        for entry in entries.flatten() {
            inspected += 1;
            if inspected > MAX_SYSTEMD_ENTRIES {
                state.truncated = true;
                state.diagnostic(format!(
                    "Systemd review stopped after {MAX_SYSTEMD_ENTRIES} entries."
                ));
                return;
            }
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.ends_with(".service.d") {
                inspect_systemd_directory(&path, state);
                continue;
            }
            if !name.ends_with(".service") {
                continue;
            }
            inspect_systemd_unit(&path, state, &mut reported_execs);
        }
    }
}

fn inspect_systemd_directory(path: &Path, state: &mut AuditState) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return;
    }
    let mode = metadata.mode() & 0o7777;
    if mode & 0o022 != 0 {
        state.push(Finding::new(
            if mode & 0o002 != 0 {
                "Critical"
            } else {
                "High"
            },
            "Systemd",
            "Systemd unit directory is writable by non-root principals",
            format!("{} mode {mode:04o}", path.display()),
            "Restrict unit and drop-in directory writes to trusted administrators.",
        ));
    }
}

fn inspect_systemd_unit(
    path: &Path,
    state: &mut AuditState,
    reported_execs: &mut HashSet<PathBuf>,
) {
    let Ok(link_metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if link_metadata.file_type().is_symlink() {
        if fs::read_link(path).is_ok_and(|target| target == Path::new("/dev/null")) {
            state.push_inventory(InventoryItem::new(
                "Systemd",
                path.display().to_string(),
                "masked unit",
            ));
        }
        return;
    }
    let metadata = link_metadata;
    let mode = metadata.mode() & 0o7777;
    if metadata.uid() != 0 || mode & 0o022 != 0 {
        state.push(Finding::new(
            "High",
            "Systemd",
            "Unsafe systemd service-file ownership or permissions",
            format!(
                "{} owner UID {}, mode {mode:04o}",
                path.display(),
                metadata.uid()
            ),
            "Require root ownership and remove group/other write permissions.",
        ));
    }

    let Ok(contents) = read_limited(path, 64 * 1024) else {
        return;
    };
    let unit = parse_service_unit(&contents);
    if !unit.runs_as_root || unit.isolated_root {
        return;
    }
    for executable in unit.executables {
        if !reported_execs.insert(executable.clone()) {
            continue;
        }
        let Ok(metadata) = fs::metadata(&executable) else {
            continue;
        };
        let mode = metadata.mode() & 0o7777;
        if metadata.uid() != 0 || mode & 0o022 != 0 {
            state.push(Finding::new(
                "High",
                "Systemd",
                "Root systemd service executes an unsafe file",
                format!(
                    "{} -> {} owner UID {}, mode {mode:04o}",
                    path.display(),
                    executable.display(),
                    metadata.uid()
                ),
                "Restore trusted ownership and remove group/other write permissions.",
            ));
        }
        let Some(parent) = executable.parent() else {
            continue;
        };
        let Ok(parent_metadata) = fs::metadata(parent) else {
            continue;
        };
        let parent_mode = parent_metadata.mode() & 0o7777;
        let sticky_protects_entry = parent_mode & 0o1000 != 0 && metadata.uid() == 0;
        if parent_mode & 0o022 != 0 && !sticky_protects_entry {
            state.push(Finding::new(
                "High",
                "Systemd",
                "Root systemd service executable is replaceable through its parent directory",
                format!(
                    "{} -> {} mode {}",
                    path.display(),
                    parent.display(),
                    mode_string(&parent_metadata)
                ),
                "Restrict parent-directory writes to trusted administrators.",
            ));
        }
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
struct ServiceUnit {
    runs_as_root: bool,
    isolated_root: bool,
    executables: Vec<PathBuf>,
}

fn parse_service_unit(contents: &str) -> ServiceUnit {
    let mut in_service = false;
    let mut user = None;
    let mut dynamic_user = false;
    let mut isolated_root = false;
    let mut executables = Vec::new();

    for raw_line in contents.lines().take(2048) {
        let line = raw_line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            in_service = line.eq_ignore_ascii_case("[Service]");
            continue;
        }
        if !in_service || line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        match key.trim() {
            "User" => user = Some(value.trim().to_string()),
            "DynamicUser" => dynamic_user = value.trim().eq_ignore_ascii_case("yes"),
            "RootDirectory" | "RootImage" if !value.trim().is_empty() => isolated_root = true,
            "ExecStart" | "ExecStartPre" | "ExecStartPost" | "ExecReload" | "ExecStop"
            | "ExecStopPost" => {
                if let Some(path) = parse_exec_path(value) {
                    executables.push(path);
                }
            }
            _ => {}
        }
    }
    let runs_as_root = !dynamic_user
        && user
            .as_deref()
            .is_none_or(|user| user.is_empty() || user == "root" || user == "0");
    ServiceUnit {
        runs_as_root,
        isolated_root,
        executables,
    }
}

fn parse_exec_path(value: &str) -> Option<PathBuf> {
    let value = value
        .trim_start_matches(['-', '+', '!', ':', '@'])
        .trim_start();
    let path = if let Some(value) = value.strip_prefix('"') {
        value.split('"').next()?
    } else {
        value.split_whitespace().next()?
    };
    let path = PathBuf::from(path);
    path.is_absolute().then_some(path)
}

fn check_unix_sockets(path: &Path, state: &mut AuditState) {
    let Ok(contents) = read_limited(path, 1024 * 1024) else {
        return;
    };
    let mut seen = HashSet::new();
    for socket in parse_unix_socket_paths(&contents)
        .into_iter()
        .take(MAX_UNIX_SOCKETS)
    {
        if !seen.insert(socket.clone()) {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&socket) else {
            continue;
        };
        if !metadata.file_type().is_socket() {
            continue;
        }
        let mode = metadata.mode() & 0o7777;
        state.push_inventory(InventoryItem::new(
            "Unix sockets",
            socket.display().to_string(),
            format!("owner UID {}, mode {mode:04o}", metadata.uid()),
        ));
        let Some(parent) = socket.parent() else {
            continue;
        };
        let Ok(parent_metadata) = fs::metadata(parent) else {
            continue;
        };
        let parent_mode = parent_metadata.mode() & 0o7777;
        if unix_socket_parent_is_unprotected(&parent_metadata) {
            state.push(Finding::new(
                "Medium",
                "Unix sockets",
                "Unix socket is located in an unprotected world-writable directory",
                format!(
                    "{}; parent {} mode {parent_mode:04o}",
                    socket.display(),
                    parent.display()
                ),
                "Move the socket to a protected runtime directory or add appropriate sticky-bit and ownership controls.",
            ));
        }
    }
}

fn unix_socket_parent_is_unprotected(metadata: &fs::Metadata) -> bool {
    let mode = metadata.mode() & 0o7777;
    mode & 0o002 != 0 && mode & 0o1000 == 0
}

fn parse_unix_socket_paths(contents: &str) -> Vec<PathBuf> {
    contents
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            let path = fields.get(7)?;
            path.starts_with('/').then(|| PathBuf::from(path))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    fn process_status_parser_reads_identity_and_capabilities() {
        let status = "Name:\tworker\nUid:\t0\t0\t0\t0\nCapEff:\t0000000000200000\nCapAmb:\t0000000000000000\nKthread:\t0\n";

        let process = parse_process_status(42, status).unwrap();

        assert_eq!(process.name, "worker");
        assert_eq!(process.uid, 0);
        assert_eq!(process.cap_eff, 0x20_0000);
        assert!(!process.is_kernel_thread);
    }

    #[test]
    fn process_status_parser_identifies_kernel_threads() {
        let status = "Name:\tkworker/0:0\nUid:\t0\t0\t0\t0\nCapEff:\t000001ffffffffff\nCapAmb:\t0000000000000000\nKthread:\t1\n";

        let process = parse_process_status(7, status).unwrap();

        assert!(process.is_kernel_thread);
    }

    #[test]
    fn systemd_parser_limits_review_to_root_host_services() {
        let unit = parse_service_unit(
            "[Service]\nUser=root\nExecStart=-/usr/bin/example --serve\nRootImage=\n",
        );

        assert!(unit.runs_as_root);
        assert!(!unit.isolated_root);
        assert_eq!(unit.executables, vec![PathBuf::from("/usr/bin/example")]);
    }

    #[test]
    fn unix_socket_parser_ignores_abstract_sockets() {
        let input = "Num RefCount Protocol Flags Type St Inode Path\n000: 2 0 10000 1 01 42 /run/test.sock\n001: 2 0 10000 1 01 43 @abstract\n";

        assert_eq!(
            parse_unix_socket_paths(input),
            vec![PathBuf::from("/run/test.sock")]
        );
    }

    #[test]
    fn masked_systemd_unit_is_not_reported_as_writable() {
        let directory = tempfile::tempdir().unwrap();
        let unit = directory.path().join("masked.service");
        symlink("/dev/null", &unit).unwrap();
        let mut state = AuditState::new(25);

        inspect_systemd_unit(&unit, &mut state, &mut HashSet::new());

        assert!(state.findings.is_empty());
        assert!(
            state
                .inventory
                .iter()
                .any(|item| item.details == "masked unit")
        );
    }

    #[test]
    fn socket_parent_policy_allows_protected_and_sticky_directories() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("runtime");
        fs::create_dir(&path).unwrap();

        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(!unix_socket_parent_is_unprotected(
            &fs::metadata(&path).unwrap()
        ));

        fs::set_permissions(&path, fs::Permissions::from_mode(0o1777)).unwrap();
        assert!(!unix_socket_parent_is_unprotected(
            &fs::metadata(&path).unwrap()
        ));

        fs::set_permissions(&path, fs::Permissions::from_mode(0o777)).unwrap();
        assert!(unix_socket_parent_is_unprotected(
            &fs::metadata(&path).unwrap()
        ));
    }
}
