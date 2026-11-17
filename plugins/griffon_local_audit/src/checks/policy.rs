use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::{AuditState, Finding, InventoryItem};

use super::{mode_string, read_limited, read_small};

const MAX_POLICY_ENTRIES: usize = 2048;

pub(super) fn run(state: &mut AuditState) {
    check_privileged_groups(Path::new("/etc/group"), state);
    check_legacy_trust_files(state);
    check_policy_permissions(state);
    check_dynamic_linker(state);
    check_security_logging(state);
    check_firewall_policy_presence(state);
    check_pam_lockout(state);
}

fn check_privileged_groups(path: &Path, state: &mut AuditState) {
    let contents = match read_small(path) {
        Ok(contents) => contents,
        Err(error) => {
            state.diagnostic(format!("Could not read {}: {error}", path.display()));
            return;
        }
    };
    let privileged = [
        ("docker", "Medium"),
        ("lxd", "Medium"),
        ("lxc", "Medium"),
        ("disk", "Medium"),
        ("shadow", "Medium"),
        ("libvirt", "Low"),
        ("kvm", "Low"),
        ("sudo", "Info"),
        ("wheel", "Info"),
        ("adm", "Info"),
    ];
    for line in contents.lines().take(1024) {
        let fields: Vec<_> = line.splitn(4, ':').collect();
        if fields.len() != 4 || fields[3].trim().is_empty() {
            continue;
        }
        let Some((_, severity)) = privileged.iter().find(|(group, _)| *group == fields[0]) else {
            continue;
        };
        let members = fields[3]
            .split(',')
            .map(str::trim)
            .filter(|member| !member.is_empty())
            .collect::<Vec<_>>();
        if members.is_empty() {
            continue;
        }
        state.push_inventory(InventoryItem::new(
            "Privileged groups",
            fields[0],
            format!("members: {}", members.join(", ")),
        ));
        if *severity != "Info" {
            state.push(Finding::new(
                severity,
                "Privileged groups",
                format!("Users belong to the {} group", fields[0]),
                format!("members: {}", members.join(", ")),
                "Confirm that every member requires this access and review membership regularly.",
            ));
        }
    }
}

fn check_legacy_trust_files(state: &mut AuditState) {
    let mut paths = vec![
        PathBuf::from("/etc/hosts.equiv"),
        PathBuf::from("/etc/ssh/shosts.equiv"),
        PathBuf::from("/root/.rhosts"),
    ];
    if let Ok(passwd) = read_small(Path::new("/etc/passwd")) {
        for line in passwd.lines().take(512) {
            let fields: Vec<_> = line.splitn(7, ':').collect();
            if fields.len() == 7 {
                let home = Path::new(fields[5]);
                if home.is_absolute() && home != Path::new("/") {
                    paths.push(home.join(".rhosts"));
                }
            }
        }
    }
    paths.sort();
    paths.dedup();
    for path in paths {
        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if metadata.file_type().is_file() && metadata.len() > 0 {
            state.push(Finding::new(
                "High",
                "Remote trust",
                "Legacy host-trust file is present",
                format!("{} mode {}", path.display(), mode_string(&metadata)),
                "Remove rhosts-style trust and use authenticated, auditable SSH access instead.",
            ));
        }
    }
}

fn check_policy_permissions(state: &mut AuditState) {
    let paths = [
        Path::new("/etc/profile"),
        Path::new("/etc/bash.bashrc"),
        Path::new("/etc/environment"),
        Path::new("/etc/profile.d"),
        Path::new("/etc/pam.d"),
        Path::new("/etc/security"),
        Path::new("/etc/sysctl.conf"),
        Path::new("/etc/sysctl.d"),
        Path::new("/etc/modules-load.d"),
        Path::new("/etc/modprobe.d"),
        Path::new("/etc/ld.so.conf.d"),
    ];
    let mut inspected = 0usize;
    for path in paths {
        inspect_policy_path(path, state);
        let Ok(entries) = fs::read_dir(path) else {
            continue;
        };
        for entry in entries.flatten() {
            inspected += 1;
            if inspected > MAX_POLICY_ENTRIES {
                state.truncated = true;
                state.diagnostic(format!(
                    "Policy permission review stopped after {MAX_POLICY_ENTRIES} entries."
                ));
                return;
            }
            inspect_policy_path(&entry.path(), state);
        }
    }
}

fn inspect_policy_path(path: &Path, state: &mut AuditState) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    if metadata.file_type().is_symlink() {
        return;
    }
    let mode = metadata.mode() & 0o7777;
    if metadata.uid() != 0 || mode & 0o022 != 0 {
        state.push(Finding::new(
            if mode & 0o002 != 0 { "Critical" } else { "High" },
            "Policy files",
            "System policy path has unsafe ownership or permissions",
            format!(
                "{} owner UID {}, mode {mode:04o}",
                path.display(),
                metadata.uid()
            ),
            "Require root ownership and remove group/other write permissions after validating the path.",
        ));
    }
}

fn check_dynamic_linker(state: &mut AuditState) {
    let preload_path = Path::new("/etc/ld.so.preload");
    if let Ok(contents) = read_limited(preload_path, 16 * 1024) {
        let entries = contents
            .lines()
            .flat_map(str::split_whitespace)
            .filter(|entry| !entry.starts_with('#'))
            .take(64)
            .collect::<Vec<_>>();
        if !entries.is_empty() {
            state.push(Finding::new(
                "Medium",
                "Dynamic linker",
                "System-wide dynamic-library preloading is configured",
                format!(
                    "{} contains {} entry or entries",
                    preload_path.display(),
                    entries.len()
                ),
                "Verify every preloaded library against an approved package or deployment source.",
            ));
            for entry in entries {
                let path = Path::new(entry);
                if path.is_absolute() {
                    inspect_linker_target(path, state);
                }
            }
        }
    }

    let mut configs = vec![PathBuf::from("/etc/ld.so.conf")];
    if let Ok(entries) = fs::read_dir("/etc/ld.so.conf.d") {
        configs.extend(
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "conf")
                })
                .take(256),
        );
    }
    let mut directories = HashSet::new();
    for config in configs {
        let Ok(contents) = read_limited(&config, 64 * 1024) else {
            continue;
        };
        for line in contents.lines().take(1024) {
            let line = line.split('#').next().unwrap_or_default().trim();
            if line.starts_with('/') && !line.contains('*') {
                directories.insert(PathBuf::from(line));
            }
        }
    }
    for directory in directories {
        inspect_linker_target(&directory, state);
    }
}

fn inspect_linker_target(path: &Path, state: &mut AuditState) {
    let Ok(metadata) = fs::metadata(path) else {
        return;
    };
    let mode = metadata.mode() & 0o7777;
    if metadata.uid() != 0 || mode & 0o022 != 0 {
        state.push(Finding::new(
            "High",
            "Dynamic linker",
            "Dynamic-linker search or preload path is writable or untrusted",
            format!(
                "{} owner UID {}, mode {mode:04o}",
                path.display(),
                metadata.uid()
            ),
            "Use only root-owned, non-writable library paths and verified preload libraries.",
        ));
    }
}

fn check_security_logging(state: &mut AuditState) {
    for path in [
        Path::new("/var/log/auth.log"),
        Path::new("/var/log/secure"),
        Path::new("/var/log/audit/audit.log"),
        Path::new("/var/log/syslog"),
        Path::new("/var/log/messages"),
    ] {
        let Ok(metadata) = fs::symlink_metadata(path) else {
            continue;
        };
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            continue;
        }
        let mode = metadata.mode() & 0o7777;
        state.push_inventory(InventoryItem::new(
            "Logging",
            path.display().to_string(),
            format!("owner UID {}, mode {mode:04o}", metadata.uid()),
        ));
        if mode & 0o022 != 0 {
            state.push(Finding::new(
                "High",
                "Logging",
                "Security log is writable by group or others",
                format!("{} mode {mode:04o}", path.display()),
                "Restrict write access to the logging service and trusted administrators.",
            ));
        }
    }

    let config_paths = [
        Path::new("/etc/systemd/journald.conf"),
        Path::new("/usr/lib/systemd/journald.conf"),
    ];
    for path in config_paths {
        let Ok(contents) = read_small(path) else {
            continue;
        };
        if let Some(storage) = parse_ini_value(&contents, "Journal", "Storage") {
            state.push_inventory(InventoryItem::new("Logging", "journald storage", &storage));
            if storage.eq_ignore_ascii_case("none") {
                state.push(Finding::new(
                    "Medium",
                    "Logging",
                    "Journald storage is disabled",
                    format!("{}: Storage=none", path.display()),
                    "Retain security-relevant logs locally or forward them to a protected remote collector.",
                ));
            } else if storage.eq_ignore_ascii_case("volatile") {
                state.push(Finding::new(
                    "Low",
                    "Logging",
                    "Journald logs are volatile",
                    format!("{}: Storage=volatile", path.display()),
                    "Use persistent or remote logging where incident-response requirements need reboot survival.",
                ));
            }
            break;
        }
    }
}

fn parse_ini_value(contents: &str, section: &str, key: &str) -> Option<String> {
    let mut current_section = "";
    let mut value = None;
    for raw_line in contents.lines().take(2048) {
        let line = raw_line.trim();
        if line.starts_with('#') || line.starts_with(';') || line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line.trim_matches(['[', ']']);
            continue;
        }
        if current_section == section
            && let Some((candidate, candidate_value)) = line.split_once('=')
            && candidate.trim() == key
        {
            value = Some(candidate_value.trim().to_string());
        }
    }
    value
}

fn check_firewall_policy_presence(state: &mut AuditState) {
    for (path, family) in [
        (Path::new("/proc/net/ip_tables_names"), "IPv4 iptables"),
        (Path::new("/proc/net/ip6_tables_names"), "IPv6 iptables"),
    ] {
        if let Ok(contents) = fs::read_to_string(path) {
            let tables = contents.split_whitespace().collect::<Vec<_>>();
            if !tables.is_empty() {
                state.push_inventory(InventoryItem::new(
                    "Firewall",
                    family,
                    format!("kernel tables: {}", tables.join(", ")),
                ));
            }
        }
    }
    for path in [
        Path::new("/etc/nftables.conf"),
        Path::new("/etc/ufw/ufw.conf"),
        Path::new("/etc/firewalld/firewalld.conf"),
    ] {
        if path.exists() {
            state.push_inventory(InventoryItem::new(
                "Firewall",
                "Policy file present",
                path.display().to_string(),
            ));
        }
    }
}

fn check_pam_lockout(state: &mut AuditState) {
    let candidates = [
        Path::new("/etc/pam.d/common-auth"),
        Path::new("/etc/pam.d/system-auth"),
        Path::new("/etc/pam.d/password-auth"),
    ];
    for path in candidates {
        let Ok(contents) = read_small(path) else {
            continue;
        };
        let has_lockout = contents
            .lines()
            .map(|line| line.split('#').next().unwrap_or_default())
            .any(|line| line.contains("pam_faillock.so") || line.contains("pam_tally2.so"));
        state.push_inventory(InventoryItem::new(
            "Authentication",
            "PAM account lockout",
            if has_lockout {
                format!("configured in {}", path.display())
            } else {
                format!("not detected in {}", path.display())
            },
        ));
        return;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privileged_group_parser_reports_members_without_password_fields() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("group");
        fs::write(&path, "docker:x:999:alice,bob\nsudo:x:27:alice\n").unwrap();
        let mut state = AuditState::new(25);

        check_privileged_groups(&path, &mut state);

        assert!(
            state
                .findings
                .iter()
                .any(|finding| finding.title.contains("docker"))
        );
        assert!(
            state
                .inventory
                .iter()
                .all(|item| !item.details.contains(":x:"))
        );
    }

    #[test]
    fn ini_parser_uses_last_value_in_requested_section() {
        let contents = "[Journal]\nStorage=auto\nStorage=persistent\n[Other]\nStorage=none\n";

        assert_eq!(
            parse_ini_value(contents, "Journal", "Storage").as_deref(),
            Some("persistent")
        );
    }
}
