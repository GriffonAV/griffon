use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::{AuditState, Finding, InventoryItem};

use super::{mode_string, read_limited, read_small};

pub(super) fn run(state: &mut AuditState) {
    check_system_identity(state);
    check_security_modules(state);
    check_extended_sysctls(state);
    check_mount_hardening(Path::new("/proc/self/mountinfo"), state);
    check_nfs_exports(Path::new("/etc/exports"), state);
    check_privileged_path(state);
    check_pending_reboot(state);
}

fn check_system_identity(state: &mut AuditState) {
    if let Ok(contents) = read_small(Path::new("/etc/os-release")) {
        let values = parse_key_values(&contents);
        let name = values
            .get("PRETTY_NAME")
            .or_else(|| values.get("NAME"))
            .map(|value| value.trim_matches('"'))
            .unwrap_or("Linux");
        state.push_inventory(InventoryItem::new(
            "System",
            name,
            format!("architecture: {}", std::env::consts::ARCH),
        ));
    }

    if let Ok(kernel) = fs::read_to_string("/proc/sys/kernel/osrelease") {
        state.push_inventory(InventoryItem::new("System", "Kernel", kernel.trim()));
    }

    let container = detect_container();
    state.push_inventory(InventoryItem::new(
        "System",
        "Execution environment",
        container.unwrap_or_else(|| "No common container marker detected".to_string()),
    ));
}

fn detect_container() -> Option<String> {
    if Path::new("/.dockerenv").exists() {
        return Some("Docker-compatible container marker present".to_string());
    }
    if Path::new("/run/.containerenv").exists() {
        return Some("OCI/Podman container marker present".to_string());
    }
    let cgroup = fs::read_to_string("/proc/1/cgroup").ok()?;
    for marker in ["docker", "kubepods", "containerd", "lxc", "podman"] {
        if cgroup.contains(marker) {
            return Some(format!("container cgroup marker: {marker}"));
        }
    }
    None
}

fn parse_key_values(contents: &str) -> HashMap<String, String> {
    contents
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }
            let (key, value) = line.split_once('=')?;
            Some((key.trim().to_string(), value.trim().to_string()))
        })
        .collect()
}

fn check_security_modules(state: &mut AuditState) {
    let lsm = fs::read_to_string("/sys/kernel/security/lsm")
        .map(|value| value.trim().to_string())
        .unwrap_or_default();
    if !lsm.is_empty() {
        state.push_inventory(InventoryItem::new(
            "Protections",
            "Linux security modules",
            &lsm,
        ));
        if !["apparmor", "selinux", "smack", "landlock"]
            .iter()
            .any(|module| lsm.split(',').any(|active| active == *module))
        {
            state.push(Finding::new(
                "Low",
                "Protections",
                "No major mandatory access-control module is active",
                format!("/sys/kernel/security/lsm = {lsm}"),
                "Evaluate AppArmor or SELinux against the system's compatibility requirements.",
            ));
        }
    }

    if let Ok(enforce) = fs::read_to_string("/sys/fs/selinux/enforce") {
        let enforce = enforce.trim();
        state.push_inventory(InventoryItem::new(
            "Protections",
            "SELinux",
            if enforce == "1" {
                "enforcing"
            } else {
                "permissive"
            },
        ));
        if enforce == "0" {
            state.push(Finding::new(
                "Medium",
                "Protections",
                "SELinux is permissive",
                "/sys/fs/selinux/enforce = 0",
                "Investigate policy denials and move to enforcing mode when the policy is ready.",
            ));
        }
    }

    if let Ok(value) = fs::read_to_string("/sys/module/apparmor/parameters/enabled") {
        state.push_inventory(InventoryItem::new(
            "Protections",
            "AppArmor",
            if value.trim().eq_ignore_ascii_case("Y") {
                "enabled"
            } else {
                "disabled"
            },
        ));
    }

    if let Ok(value) = fs::read_to_string("/sys/kernel/security/lockdown") {
        let value = value.trim();
        state.push_inventory(InventoryItem::new("Protections", "Kernel lockdown", value));
        if value.contains("[none]") {
            state.push(Finding::new(
                "Low",
                "Protections",
                "Kernel lockdown is disabled",
                format!("/sys/kernel/security/lockdown = {value}"),
                "Consider integrity or confidentiality lockdown on systems that use Secure Boot.",
            ));
        }
    }

    if let Some(enabled) = secure_boot_state() {
        state.push_inventory(InventoryItem::new(
            "Protections",
            "UEFI Secure Boot",
            if enabled { "enabled" } else { "disabled" },
        ));
        if !enabled {
            state.push(Finding::new(
                "Low",
                "Protections",
                "UEFI Secure Boot is disabled",
                "EFI variables report SecureBoot=0",
                "Consider enabling Secure Boot where the boot chain and kernel modules support it.",
            ));
        }
    }

    if let Ok(tainted) = fs::read_to_string("/proc/sys/kernel/tainted")
        && let Ok(value) = tainted.trim().parse::<u64>()
        && value != 0
    {
        state.push_inventory(InventoryItem::new(
            "Protections",
            "Kernel taint flags",
            value.to_string(),
        ));
        if value & (1 << 12) != 0 {
            state.push(Finding::new(
                "Low",
                "Protections",
                "Kernel reports an externally built module",
                format!("kernel taint mask: {value}"),
                "Confirm that out-of-tree modules are expected, maintained and trusted.",
            ));
        }
        if value & (1 << 13) != 0 {
            state.push(Finding::new(
                "Medium",
                "Protections",
                "Kernel reports an unsigned module",
                format!("kernel taint mask: {value}"),
                "Identify the unsigned module and verify its provenance and signing policy.",
            ));
        }
    }
}

fn secure_boot_state() -> Option<bool> {
    let directory = fs::read_dir("/sys/firmware/efi/efivars").ok()?;
    for entry in directory.flatten().take(1024) {
        if !entry
            .file_name()
            .to_string_lossy()
            .starts_with("SecureBoot-")
        {
            continue;
        }
        let value = fs::read(entry.path()).ok()?;
        return value.get(4).map(|byte| *byte == 1);
    }
    None
}

fn check_extended_sysctls(state: &mut AuditState) {
    let minimums = [
        (
            "/proc/sys/fs/protected_symlinks",
            1,
            "Medium",
            "Protected symlinks are disabled",
        ),
        (
            "/proc/sys/fs/protected_hardlinks",
            1,
            "Medium",
            "Protected hardlinks are disabled",
        ),
        (
            "/proc/sys/fs/protected_regular",
            1,
            "Low",
            "Protected regular files are disabled",
        ),
        (
            "/proc/sys/fs/protected_fifos",
            1,
            "Low",
            "Protected FIFOs are disabled",
        ),
        (
            "/proc/sys/kernel/unprivileged_bpf_disabled",
            1,
            "Medium",
            "Unprivileged eBPF is enabled",
        ),
        (
            "/proc/sys/kernel/perf_event_paranoid",
            2,
            "Low",
            "Performance events are broadly accessible",
        ),
        (
            "/proc/sys/vm/mmap_min_addr",
            32_768,
            "Low",
            "Low-address mappings are weakly restricted",
        ),
        (
            "/proc/sys/net/ipv4/tcp_syncookies",
            1,
            "Low",
            "TCP SYN cookies are disabled",
        ),
        (
            "/proc/sys/net/ipv4/conf/all/rp_filter",
            1,
            "Low",
            "Reverse-path filtering is disabled",
        ),
    ];
    for (path, minimum, severity, title) in minimums {
        check_sysctl_minimum(path, minimum, severity, title, state);
    }

    let zeros = [
        (
            "/proc/sys/fs/suid_dumpable",
            "Medium",
            "Set-ID programs may produce core dumps",
        ),
        (
            "/proc/sys/net/ipv4/conf/all/accept_redirects",
            "Low",
            "IPv4 ICMP redirects are accepted",
        ),
        (
            "/proc/sys/net/ipv4/conf/all/send_redirects",
            "Low",
            "IPv4 ICMP redirects are sent",
        ),
        (
            "/proc/sys/net/ipv4/conf/all/accept_source_route",
            "Medium",
            "IPv4 source routing is accepted",
        ),
        (
            "/proc/sys/net/ipv6/conf/all/accept_redirects",
            "Low",
            "IPv6 ICMP redirects are accepted",
        ),
        (
            "/proc/sys/net/ipv6/conf/all/accept_source_route",
            "Medium",
            "IPv6 source routing is accepted",
        ),
    ];
    for (path, severity, title) in zeros {
        check_sysctl_zero(path, severity, title, state);
    }

    if let Ok(value) = fs::read_to_string("/proc/sys/net/ipv4/ip_forward") {
        state.push_inventory(InventoryItem::new(
            "Network policy",
            "IPv4 forwarding",
            if value.trim() == "1" {
                "enabled"
            } else {
                "disabled"
            },
        ));
    }
}

fn check_sysctl_minimum(
    path: &str,
    minimum: i64,
    severity: &'static str,
    title: &'static str,
    state: &mut AuditState,
) {
    let Ok(value) = fs::read_to_string(path) else {
        return;
    };
    let Ok(observed) = value.trim().parse::<i64>() else {
        state.diagnostic(format!("Unexpected value in {path}."));
        return;
    };
    if observed < minimum {
        state.push(Finding::new(
            severity,
            "Kernel hardening",
            title,
            format!("{path} = {observed}; expected at least {minimum}"),
            "Review the matching sysctl against the host role and persist a hardened value.",
        ));
    }
}

fn check_sysctl_zero(
    path: &str,
    severity: &'static str,
    title: &'static str,
    state: &mut AuditState,
) {
    let Ok(value) = fs::read_to_string(path) else {
        return;
    };
    let Ok(observed) = value.trim().parse::<i64>() else {
        state.diagnostic(format!("Unexpected value in {path}."));
        return;
    };
    if observed != 0 {
        state.push(Finding::new(
            severity,
            "Kernel hardening",
            title,
            format!("{path} = {observed}; expected 0"),
            "Review the matching sysctl against the host role and persist the intended value.",
        ));
    }
}

fn check_mount_hardening(path: &Path, state: &mut AuditState) {
    let Ok(contents) = read_limited(path, 1024 * 1024) else {
        state.diagnostic(format!(
            "Could not read bounded mount information from {}.",
            path.display()
        ));
        return;
    };
    for mount in parse_mountinfo(&contents).into_iter().take(512) {
        if matches!(
            mount.mount_point.as_str(),
            "/" | "/tmp" | "/var/tmp" | "/dev/shm"
        ) {
            state.push_inventory(InventoryItem::new(
                "Mounts",
                &mount.mount_point,
                format!("{}; {}", mount.filesystem, format_options(&mount.options)),
            ));
        }

        if matches!(mount.mount_point.as_str(), "/tmp" | "/var/tmp" | "/dev/shm") {
            let missing: Vec<_> = ["nodev", "nosuid"]
                .into_iter()
                .filter(|option| !mount.options.contains(*option))
                .collect();
            if !missing.is_empty() {
                state.push(Finding::new(
                    "Low",
                    "Mounts",
                    format!("Temporary mount {} lacks hardening options", mount.mount_point),
                    format!(
                        "missing: {}; options: {}",
                        missing.join(","),
                        format_options(&mount.options)
                    ),
                    "Add nodev/nosuid where compatible with applications using this temporary filesystem.",
                ));
            }
        }

        if matches!(mount.filesystem.as_str(), "nfs" | "nfs4" | "cifs" | "smb3") {
            state.push_inventory(InventoryItem::new(
                "Mounts",
                format!("Network filesystem: {}", mount.mount_point),
                format!("{}; {}", mount.filesystem, format_options(&mount.options)),
            ));
            if mount.options.contains("rw") && !mount.options.contains("nosuid") {
                state.push(Finding::new(
                    "Low",
                    "Mounts",
                    format!("Writable network mount {} permits set-ID semantics", mount.mount_point),
                    format!(
                        "{} options: {}",
                        mount.filesystem,
                        format_options(&mount.options)
                    ),
                    "Use nosuid on writable network filesystems unless set-ID behavior is explicitly required.",
                ));
            }
        }
    }
}

fn format_options(options: &HashSet<String>) -> String {
    let mut options: Vec<_> = options.iter().map(String::as_str).collect();
    options.sort_unstable();
    options.join(",")
}

#[derive(Debug, PartialEq, Eq)]
struct MountRecord {
    mount_point: String,
    filesystem: String,
    options: HashSet<String>,
}

fn parse_mountinfo(contents: &str) -> Vec<MountRecord> {
    contents
        .lines()
        .filter_map(|line| {
            let fields: Vec<_> = line.split_whitespace().collect();
            let separator = fields.iter().position(|field| *field == "-")?;
            if separator < 6 || fields.len() <= separator + 3 {
                return None;
            }
            let mut options = HashSet::new();
            for field in [fields[5], fields[separator + 3]] {
                options.extend(field.split(',').map(str::to_string));
            }
            Some(MountRecord {
                mount_point: decode_mount_field(fields[4]),
                filesystem: fields[separator + 1].to_string(),
                options,
            })
        })
        .collect()
}

fn decode_mount_field(value: &str) -> String {
    value
        .replace("\\040", " ")
        .replace("\\011", "\t")
        .replace("\\012", "\n")
        .replace("\\134", "\\")
}

fn check_nfs_exports(path: &Path, state: &mut AuditState) {
    let Ok(contents) = read_limited(path, 16 * 1024) else {
        return;
    };
    for (index, raw_line) in contents.lines().take(128).enumerate() {
        let line = raw_line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        if line.contains("no_root_squash") {
            state.push(Finding::new(
                "High",
                "NFS",
                "NFS export disables root squashing",
                format!("{}:{} contains no_root_squash", path.display(), index + 1),
                "Remove no_root_squash unless a narrowly scoped, documented requirement exists.",
            ));
        }
        if line.contains("*(") && line.contains("rw") {
            state.push(Finding::new(
                "Medium",
                "NFS",
                "Writable NFS export appears available to all clients",
                format!(
                    "{}:{} uses a wildcard client with rw",
                    path.display(),
                    index + 1
                ),
                "Restrict clients and use strong authentication and identity mapping.",
            ));
        }
    }
}

fn check_privileged_path(state: &mut AuditState) {
    let Some(path) = std::env::var_os("PATH") else {
        state.push(Finding::new(
            "High",
            "Environment",
            "The daemon process has no PATH",
            "PATH is unset",
            "Set an explicit, minimal PATH for the daemon service.",
        ));
        return;
    };
    let path = path.to_string_lossy();
    // SAFETY: geteuid has no preconditions and does not mutate process state.
    let effective_uid = unsafe { libc::geteuid() };
    if effective_uid != 0 {
        state.push_inventory(InventoryItem::new(
            "Environment",
            "Runner PATH",
            format!("effective UID {effective_uid}; privileged PATH findings are not applicable"),
        ));
        return;
    }
    let mut seen = HashSet::<PathBuf>::new();
    for component in path.split(':') {
        if component.is_empty() || component == "." {
            state.push(Finding::new(
                "High",
                "Environment",
                "Daemon PATH includes the current directory",
                format!("PATH={path}"),
                "Use absolute, trusted directories only in the daemon PATH.",
            ));
            continue;
        }
        let directory = PathBuf::from(component);
        if !directory.is_absolute() {
            state.push(Finding::new(
                "High",
                "Environment",
                "Daemon PATH includes a relative directory",
                component,
                "Use absolute, trusted directories only in the daemon PATH.",
            ));
            continue;
        }
        let canonical = directory.canonicalize().unwrap_or(directory);
        if !seen.insert(canonical.clone()) {
            continue;
        }
        let Ok(metadata) = fs::metadata(&canonical) else {
            continue;
        };
        if metadata.mode() & 0o022 != 0 {
            state.push(Finding::new(
                "High",
                "Environment",
                "Daemon PATH contains a writable directory",
                format!("{} mode {}", canonical.display(), mode_string(&metadata)),
                "Remove group/other write access or remove the directory from the service PATH.",
            ));
        }
    }
}

fn check_pending_reboot(state: &mut AuditState) {
    for path in [
        Path::new("/run/reboot-required"),
        Path::new("/var/run/reboot-required"),
    ] {
        if path.exists() {
            state.push(Finding::new(
                "Medium",
                "Updates",
                "A reboot is required to finish applying updates",
                path.display().to_string(),
                "Schedule a controlled reboot after validating service availability and change windows.",
            ));
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mountinfo_parser_combines_mount_and_superblock_options() {
        let records =
            parse_mountinfo("36 25 0:32 / /dev/shm rw,nosuid,nodev - tmpfs tmpfs rw,size=1024k\n");

        assert_eq!(records.len(), 1);
        assert_eq!(records[0].mount_point, "/dev/shm");
        assert!(records[0].options.contains("nosuid"));
        assert!(records[0].options.contains("size=1024k"));
    }

    #[test]
    fn nfs_parser_flags_no_root_squash_without_exposing_contents() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("exports");
        fs::write(&path, "/srv *(rw,no_root_squash)\n").unwrap();
        let mut state = AuditState::new(25);

        check_nfs_exports(&path, &mut state);

        assert!(
            state
                .findings
                .iter()
                .any(|finding| finding.category == "NFS")
        );
        assert!(
            state
                .findings
                .iter()
                .all(|finding| !finding.evidence.contains("/srv"))
        );
    }
}
