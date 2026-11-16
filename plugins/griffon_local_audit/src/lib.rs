mod checks;

use std::collections::{HashMap, HashSet};
use std::ffi::CString;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::time::Instant;

use plugin_sdk::{PluginRouter, export_plugin};
use serde::{Deserialize, Serialize};

const PLUGIN_UUID: &str = "6f7b1e8a-2fd5-4cab-9c72-69e0f0b43521";
const MIN_FINDINGS: usize = 25;
const MAX_FINDINGS: usize = 500;
const MAX_DIAGNOSTICS: usize = 50;
const MAX_FILESYSTEM_ENTRIES: usize = 100_000;
const MAX_SCHEDULED_ENTRIES: usize = 4096;

#[derive(Deserialize)]
struct AuditRequest {
    #[serde(default = "default_include_suid")]
    include_suid: bool,
    #[serde(default = "default_max_findings")]
    max_findings: usize,
}

fn default_include_suid() -> bool {
    true
}

fn default_max_findings() -> usize {
    250
}

#[derive(Serialize)]
struct Finding {
    severity: &'static str,
    category: &'static str,
    title: String,
    evidence: String,
    recommendation: String,
}

#[derive(Serialize)]
struct InventoryItem {
    category: &'static str,
    item: String,
    details: String,
}

impl InventoryItem {
    fn new(category: &'static str, item: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            category,
            item: item.into(),
            details: details.into(),
        }
    }
}

impl Finding {
    fn new(
        severity: &'static str,
        category: &'static str,
        title: impl Into<String>,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            category,
            title: title.into(),
            evidence: evidence.into(),
            recommendation: recommendation.into(),
        }
    }
}

#[derive(Serialize)]
struct AuditResponse {
    ok: bool,
    message: String,
    duration_ms: u128,
    critical_count: usize,
    high_count: usize,
    medium_count: usize,
    low_count: usize,
    info_count: usize,
    truncated: bool,
    diagnostics_summary: String,
    findings: Vec<Finding>,
    inventory: Vec<InventoryItem>,
    diagnostics: Vec<String>,
}

struct AuditState {
    findings: Vec<Finding>,
    inventory: Vec<InventoryItem>,
    diagnostics: Vec<String>,
    max_findings: usize,
    max_inventory: usize,
    truncated: bool,
}

impl AuditState {
    fn new(max_findings: usize) -> Self {
        Self {
            findings: Vec::with_capacity(max_findings.min(128)),
            inventory: Vec::with_capacity(max_findings.min(128)),
            diagnostics: Vec::new(),
            max_findings,
            max_inventory: max_findings,
            truncated: false,
        }
    }

    fn push(&mut self, finding: Finding) {
        if self.findings.len() < self.max_findings {
            self.findings.push(finding);
        } else {
            self.truncated = true;
        }
    }

    fn diagnostic(&mut self, message: impl Into<String>) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics.push(message.into());
        }
    }

    fn push_inventory(&mut self, item: InventoryItem) {
        if self.inventory.len() < self.max_inventory {
            self.inventory.push(item);
        } else {
            self.truncated = true;
        }
    }

    fn is_full(&self) -> bool {
        self.findings.len() >= self.max_findings
    }

    fn inventory_is_full(&self) -> bool {
        self.inventory.len() >= self.max_inventory
    }
}

fn build_router() -> PluginRouter {
    PluginRouter::new().method("audit", audit)
}

export_plugin! {
    meta: PluginMeta {
        name: "Local Security Audit",
        uuid: PLUGIN_UUID,
        author: "Griffon",
        description: "Read-only Linux security posture audit with bounded local checks.",
    },
    router: build_router,
}

fn audit(request: AuditRequest) -> Result<AuditResponse, String> {
    let started_at = Instant::now();
    let mut state = AuditState::new(request.max_findings.clamp(MIN_FINDINGS, MAX_FINDINGS));

    check_accounts(Path::new("/etc/passwd"), &mut state);
    check_sensitive_paths(&mut state);
    check_kernel_hardening(&mut state);
    check_sshd_config(Path::new("/etc/ssh/sshd_config"), &mut state);
    check_scheduled_tasks(&mut state);
    checks::run_all(&mut state);

    if request.include_suid {
        let roots = [
            Path::new("/usr/bin"),
            Path::new("/usr/sbin"),
            Path::new("/bin"),
            Path::new("/sbin"),
        ];
        check_privileged_executables(&roots, &mut state);
    }
    check_listening_tcp_ports(&mut state);
    check_bound_udp_ports(&mut state);

    deduplicate_findings(&mut state.findings);
    state
        .findings
        .sort_by_key(|finding| severity_rank(finding.severity));
    state.inventory.sort_by(|left, right| {
        left.category
            .cmp(right.category)
            .then_with(|| left.item.cmp(&right.item))
    });

    let critical_count = count_severity(&state.findings, "Critical");
    let high_count = count_severity(&state.findings, "High");
    let medium_count = count_severity(&state.findings, "Medium");
    let low_count = count_severity(&state.findings, "Low");
    let info_count = state.inventory.len();
    let actionable_count = critical_count + high_count + medium_count + low_count;
    let diagnostic_count = state.diagnostics.len();
    let diagnostics_summary = if state.diagnostics.is_empty() {
        "All available checks completed without read errors.".to_string()
    } else {
        state.diagnostics.join(" · ")
    };

    let mut message = format!(
        "Audit completed: {actionable_count} actionable finding(s), {info_count} informational item(s)."
    );
    if diagnostic_count > 0 {
        message.push_str(&format!(
            " {diagnostic_count} check(s) could not be completed; see the diagnostics below."
        ));
    }
    if state.truncated {
        message.push_str(" The report reached its configured limit and was truncated.");
    }

    Ok(AuditResponse {
        ok: true,
        message,
        duration_ms: started_at.elapsed().as_millis(),
        critical_count,
        high_count,
        medium_count,
        low_count,
        info_count,
        truncated: state.truncated,
        diagnostics_summary,
        findings: state.findings,
        inventory: state.inventory,
        diagnostics: state.diagnostics,
    })
}

fn deduplicate_findings(findings: &mut Vec<Finding>) {
    let mut seen = HashSet::new();
    findings.retain(|finding| {
        seen.insert((
            finding.severity,
            finding.category,
            finding.title.clone(),
            finding.evidence.clone(),
        ))
    });
}

fn severity_rank(severity: &str) -> u8 {
    match severity {
        "Critical" => 0,
        "High" => 1,
        "Medium" => 2,
        "Low" => 3,
        _ => 4,
    }
}

fn count_severity(findings: &[Finding], severity: &str) -> usize {
    findings
        .iter()
        .filter(|finding| finding.severity == severity)
        .count()
}

fn check_accounts(path: &Path, state: &mut AuditState) {
    match checks::read_limited(path, 64 * 1024) {
        Ok(contents) => check_accounts_from(&contents, state),
        Err(error) => state.diagnostic(format!("Could not read {}: {error}", path.display())),
    }
}

fn check_accounts_from(contents: &str, state: &mut AuditState) {
    for line in contents.lines().filter(|line| !line.trim().is_empty()) {
        let mut fields = line.splitn(7, ':');
        let (Some(username), Some(password_marker), Some(uid_raw)) =
            (fields.next(), fields.next(), fields.next())
        else {
            state.diagnostic("Ignored a malformed entry in /etc/passwd.");
            continue;
        };
        if fields.count() != 4 {
            state.diagnostic("Ignored a malformed entry in /etc/passwd.");
            continue;
        }
        let Ok(uid) = uid_raw.parse::<u32>() else {
            state.diagnostic(format!("Ignored invalid UID for account {username}."));
            continue;
        };

        if uid == 0 && username != "root" {
            state.push(Finding::new(
                "Critical",
                "Accounts",
                format!("Account {username} has UID 0"),
                format!("/etc/passwd assigns UID 0 to {username}"),
                "Confirm that the account is required and remove unintended root-equivalent identities.",
            ));
        }

        if password_marker.is_empty() {
            state.push(Finding::new(
                "High",
                "Accounts",
                format!("Account {username} has an empty password field"),
                format!("/etc/passwd contains an empty password marker for {username}"),
                "Lock the account or configure authentication through the system password database.",
            ));
        }
    }
}

fn check_sensitive_paths(state: &mut AuditState) {
    let paths = [
        (Path::new("/etc/passwd"), true),
        (Path::new("/etc/shadow"), true),
        (Path::new("/etc/group"), true),
        (Path::new("/etc/gshadow"), false),
        (Path::new("/etc/sudoers"), false),
        (Path::new("/etc/doas.conf"), false),
        (Path::new("/etc/ld.so.conf"), false),
        (Path::new("/etc/ld.so.preload"), false),
        (Path::new("/etc/ssh/sshd_config"), false),
    ];

    for (path, required) in paths {
        inspect_sensitive_path(path, required, state);
    }
}

fn inspect_sensitive_path(path: &Path, required: bool, state: &mut AuditState) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            if required {
                state.diagnostic(format!("{} is not present.", path.display()));
            }
            return;
        }
        Err(error) => {
            state.diagnostic(format!("Could not inspect {}: {error}", path.display()));
            return;
        }
    };

    let mode = metadata.mode() & 0o7777;
    if metadata.uid() != 0 {
        state.push(Finding::new(
            "High",
            "Permissions",
            format!("{} is not owned by root", path.display()),
            format!("owner UID: {}, mode: {mode:04o}", metadata.uid()),
            "Restore the expected root ownership after confirming the file is legitimate.",
        ));
    }

    if mode & 0o022 != 0 {
        state.push(Finding::new(
            "High",
            "Permissions",
            format!("{} is writable by group or others", path.display()),
            format!("mode: {mode:04o}"),
            "Remove unnecessary group/other write permissions and verify file ownership.",
        ));
    }

    if matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some("shadow" | "gshadow")
    ) && mode & 0o004 != 0
    {
        state.push(Finding::new(
            "High",
            "Permissions",
            format!("{} is readable by every user", path.display()),
            format!("mode: {mode:04o}"),
            "Restrict access to the system password database.",
        ));
    }
}

fn check_kernel_hardening(state: &mut AuditState) {
    let checks = [
        (
            "/proc/sys/kernel/randomize_va_space",
            "2",
            "Medium",
            "Full address-space randomization is disabled",
            "Enable full ASLR unless a documented compatibility requirement prevents it.",
        ),
        (
            "/proc/sys/kernel/yama/ptrace_scope",
            "1",
            "Medium",
            "Process tracing restrictions are weak",
            "Set kernel.yama.ptrace_scope to at least 1 where Yama is available.",
        ),
        (
            "/proc/sys/kernel/dmesg_restrict",
            "1",
            "Low",
            "Kernel messages are available to unprivileged users",
            "Set kernel.dmesg_restrict=1 unless unprivileged access is required.",
        ),
        (
            "/proc/sys/kernel/kptr_restrict",
            "1",
            "Low",
            "Kernel pointer exposure restrictions are weak",
            "Set kernel.kptr_restrict to at least 1.",
        ),
    ];

    for (path, minimum, severity, title, recommendation) in checks {
        match fs::read_to_string(path) {
            Ok(value) => {
                let observed = value.trim().parse::<u32>();
                let expected = minimum.parse::<u32>().expect("static minimum is valid");
                match observed {
                    Ok(observed) if observed < expected => state.push(Finding::new(
                        severity,
                        "Kernel",
                        title,
                        format!("{path} = {observed}; expected at least {expected}"),
                        recommendation,
                    )),
                    Ok(_) => {}
                    Err(_) => state.diagnostic(format!("Unexpected value in {path}.")),
                }
            }
            Err(error) => state.diagnostic(format!("Could not read {path}: {error}")),
        }
    }
}

fn check_sshd_config(path: &Path, state: &mut AuditState) {
    let contents = match checks::read_limited(path, 256 * 1024) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return;
        }
        Err(error) => {
            state.diagnostic(format!("Could not read {}: {error}", path.display()));
            return;
        }
    };

    let directives = parse_global_sshd_directives(&contents);
    if directives
        .get("permitrootlogin")
        .is_some_and(|value| value == "yes")
    {
        state.push(Finding::new(
            "High",
            "SSH",
            "SSH permits direct root login",
            format!("{}: PermitRootLogin yes", path.display()),
            "Prefer individual accounts with audited privilege elevation.",
        ));
    }
    if directives
        .get("permitemptypasswords")
        .is_some_and(|value| value == "yes")
    {
        state.push(Finding::new(
            "Critical",
            "SSH",
            "SSH permits empty passwords",
            format!("{}: PermitEmptyPasswords yes", path.display()),
            "Disable empty-password authentication immediately.",
        ));
    }
    if directives
        .get("passwordauthentication")
        .is_some_and(|value| value == "yes")
    {
        state.push(Finding::new(
            "Medium",
            "SSH",
            "SSH password authentication is enabled",
            format!("{}: PasswordAuthentication yes", path.display()),
            "Prefer key-based authentication where operationally appropriate.",
        ));
    }
}

fn parse_global_sshd_directives(contents: &str) -> HashMap<String, String> {
    let mut directives = HashMap::new();
    for line in contents.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split_whitespace();
        let Some(key) = fields.next() else {
            continue;
        };
        if key.eq_ignore_ascii_case("match") {
            break;
        }
        let Some(value) = fields.next() else {
            continue;
        };
        directives
            .entry(key.to_ascii_lowercase())
            .or_insert_with(|| value.to_ascii_lowercase());
    }
    directives
}

fn check_scheduled_tasks(state: &mut AuditState) {
    let files = [Path::new("/etc/crontab"), Path::new("/etc/anacrontab")];
    for path in files {
        inspect_scheduled_path(path, state);
    }

    let directories = [
        Path::new("/etc/cron.d"),
        Path::new("/etc/cron.daily"),
        Path::new("/etc/cron.hourly"),
        Path::new("/etc/cron.weekly"),
        Path::new("/etc/cron.monthly"),
        Path::new("/var/spool/cron"),
        Path::new("/etc/systemd/system"),
    ];

    let mut inspected = 0usize;
    for directory in directories {
        inspect_scheduled_path(directory, state);
        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                state.diagnostic(format!("Could not list {}: {error}", directory.display()));
                continue;
            }
        };

        for entry in entries {
            inspected += 1;
            if inspected > MAX_SCHEDULED_ENTRIES {
                state.truncated = true;
                state.diagnostic(format!(
                    "Scheduled-task review stopped after {MAX_SCHEDULED_ENTRIES} entries."
                ));
                return;
            }
            match entry {
                Ok(entry) => inspect_scheduled_path(&entry.path(), state),
                Err(error) => state.diagnostic(format!(
                    "Could not inspect an entry in {}: {error}",
                    directory.display()
                )),
            }
            if state.is_full() {
                return;
            }
        }
    }
}

fn inspect_scheduled_path(path: &Path, state: &mut AuditState) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            state.diagnostic(format!("Could not inspect {}: {error}", path.display()));
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        return;
    }

    let mode = metadata.mode() & 0o7777;
    if is_cron_spool_path(path) {
        if mode & 0o002 != 0 {
            state.push(Finding::new(
                "High",
                "Scheduled tasks",
                format!("World-writable cron spool path: {}", path.display()),
                format!("owner UID: {}, mode: {mode:04o}", metadata.uid()),
                "Remove world-write access while preserving the distribution's cron group policy.",
            ));
        }
        return;
    }
    if metadata.uid() != 0 || mode & 0o022 != 0 {
        state.push(Finding::new(
            "High",
            "Scheduled tasks",
            format!("Unsafe ownership or permissions on {}", path.display()),
            format!("owner UID: {}, mode: {mode:04o}", metadata.uid()),
            "Require root ownership and remove group/other write permissions after validating the entry.",
        ));
    }
}

fn is_cron_spool_path(path: &Path) -> bool {
    path.starts_with("/var/spool/cron")
}

fn check_listening_tcp_ports(state: &mut AuditState) {
    let mut ports = HashSet::new();
    for path in [Path::new("/proc/net/tcp"), Path::new("/proc/net/tcp6")] {
        match checks::read_limited(path, 2 * 1024 * 1024) {
            Ok(contents) => parse_listening_tcp_ports(&contents, &mut ports),
            Err(error) => state.diagnostic(format!("Could not read {}: {error}", path.display())),
        }
    }

    let mut ports: Vec<_> = ports.into_iter().collect();
    ports.sort_unstable();
    for port in ports {
        state.push_inventory(InventoryItem::new(
            "Network",
            format!("TCP port {port}"),
            "Listening in the runner process network namespace",
        ));
        if state.inventory_is_full() {
            break;
        }
    }
}

fn check_bound_udp_ports(state: &mut AuditState) {
    let mut ports = HashSet::new();
    for path in [Path::new("/proc/net/udp"), Path::new("/proc/net/udp6")] {
        match checks::read_limited(path, 2 * 1024 * 1024) {
            Ok(contents) => parse_bound_udp_ports(&contents, &mut ports),
            Err(error) => state.diagnostic(format!("Could not read {}: {error}", path.display())),
        }
    }

    let mut ports: Vec<_> = ports.into_iter().collect();
    ports.sort_unstable();
    for port in ports {
        state.push_inventory(InventoryItem::new(
            "Network",
            format!("UDP port {port}"),
            "Bound in the runner process network namespace",
        ));
        if state.inventory_is_full() {
            break;
        }
    }
}

fn parse_listening_tcp_ports(contents: &str, ports: &mut HashSet<u16>) {
    for line in contents.lines().skip(1) {
        let mut fields = line.split_whitespace();
        let Some(local_address) = fields.nth(1) else {
            continue;
        };
        let Some(state) = fields.nth(1) else {
            continue;
        };
        if state != "0A" {
            continue;
        }
        let Some(port_hex) = local_address.rsplit(':').next() else {
            continue;
        };
        if let Ok(port) = u16::from_str_radix(port_hex, 16) {
            ports.insert(port);
        }
    }
}

fn parse_bound_udp_ports(contents: &str, ports: &mut HashSet<u16>) {
    for line in contents.lines().skip(1) {
        let Some(local_address) = line.split_whitespace().nth(1) else {
            continue;
        };
        let Some(port_hex) = local_address.rsplit(':').next() else {
            continue;
        };
        if let Ok(port) = u16::from_str_radix(port_hex, 16)
            && port != 0
        {
            ports.insert(port);
        }
    }
}

fn check_privileged_executables(roots: &[&Path], state: &mut AuditState) {
    let mut canonical_roots = HashSet::new();
    let mut stack = Vec::new();
    for root in roots {
        match root.canonicalize() {
            Ok(path) if canonical_roots.insert(path.clone()) => stack.push(path),
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                state.diagnostic(format!("Could not resolve {}: {error}", root.display()))
            }
        }
    }

    let mut visited = 0usize;
    while let Some(directory) = stack.pop() {
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) => {
                state.diagnostic(format!("Could not list {}: {error}", directory.display()));
                continue;
            }
        };

        for entry in entries {
            visited += 1;
            if visited > MAX_FILESYSTEM_ENTRIES {
                state.truncated = true;
                state.diagnostic(format!(
                    "Privileged executable scan stopped after {MAX_FILESYSTEM_ENTRIES} entries."
                ));
                return;
            }

            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    state.diagnostic(format!(
                        "Could not inspect an entry in {}: {error}",
                        directory.display()
                    ));
                    continue;
                }
            };
            let path = entry.path();
            let metadata = match fs::symlink_metadata(&path) {
                Ok(metadata) => metadata,
                Err(error) => {
                    state.diagnostic(format!("Could not inspect {}: {error}", path.display()));
                    continue;
                }
            };

            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                stack.push(path);
                continue;
            }
            if !metadata.is_file() {
                continue;
            }

            if metadata.mode() & 0o111 != 0 {
                inspect_file_capabilities(&path, &metadata, state);
            }

            if metadata.mode() & 0o6000 == 0 {
                continue;
            }

            let mode = metadata.mode() & 0o7777;
            if mode & 0o002 != 0 {
                state.push(Finding::new(
                    "Critical",
                    "Privileged files",
                    format!("World-writable privileged executable: {}", path.display()),
                    format!("owner UID: {}, mode: {mode:04o}", metadata.uid()),
                    "Remove world-write access immediately and verify the file against its package source.",
                ));
            } else {
                state.push_inventory(InventoryItem::new(
                    "Privileged file",
                    path.display().to_string(),
                    format!("owner UID: {}, mode: {mode:04o}", metadata.uid()),
                ));
            }
            if state.is_full() && state.inventory_is_full() {
                return;
            }
        }
    }
}

fn inspect_file_capabilities(path: &Path, metadata: &fs::Metadata, state: &mut AuditState) {
    let Ok(Some(bytes)) = read_file_capability(path) else {
        return;
    };
    let Some(mask) = parse_file_capability_mask(&bytes) else {
        state.diagnostic(format!(
            "Invalid security.capability metadata on {}.",
            path.display()
        ));
        return;
    };
    let names = capability_names(mask);
    state.push_inventory(InventoryItem::new(
        "File capabilities",
        path.display().to_string(),
        names.join(", "),
    ));

    let dangerous = [1u8, 2, 6, 7, 16, 17, 19, 21, 22, 25, 30, 32, 33, 39];
    if dangerous.iter().any(|bit| mask & (1u64 << bit) != 0) {
        state.push(Finding::new(
            "High",
            "File capabilities",
            format!("Executable has high-impact Linux capabilities: {}", path.display()),
            format!(
                "owner UID {}, mode {:04o}, capabilities {}",
                metadata.uid(),
                metadata.mode() & 0o7777,
                names.join(", ")
            ),
            "Confirm that each capability is required and prefer a narrowly scoped service sandbox.",
        ));
    }
}

fn read_file_capability(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    let path = CString::new(path.as_os_str().as_bytes())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "path contains NUL"))?;
    let name = c"security.capability";
    // SAFETY: both C strings are NUL-terminated and valid for the duration of
    // the calls; a null buffer with length zero is the documented size query.
    let size = unsafe { libc::getxattr(path.as_ptr(), name.as_ptr(), std::ptr::null_mut(), 0) };
    if size < 0 {
        let error = std::io::Error::last_os_error();
        if matches!(error.raw_os_error(), Some(code) if code == libc::ENODATA || code == libc::ENOTSUP || code == libc::EOPNOTSUPP)
        {
            return Ok(None);
        }
        return Err(error);
    }
    if size == 0 {
        return Ok(None);
    }
    if size > 64 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "security.capability is unexpectedly large",
        ));
    }
    let mut bytes = vec![0u8; size as usize];
    // SAFETY: the vector has `size` writable bytes and the C strings remain valid.
    let read = unsafe {
        libc::getxattr(
            path.as_ptr(),
            name.as_ptr(),
            bytes.as_mut_ptr().cast(),
            bytes.len(),
        )
    };
    if read < 0 {
        return Err(std::io::Error::last_os_error());
    }
    bytes.truncate(read as usize);
    Ok(Some(bytes))
}

fn parse_file_capability_mask(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 12 {
        return None;
    }
    let lower_permitted = u32::from_le_bytes(bytes.get(4..8)?.try_into().ok()?);
    let lower_inheritable = u32::from_le_bytes(bytes.get(8..12)?.try_into().ok()?);
    let mut mask = u64::from(lower_permitted | lower_inheritable);
    if bytes.len() >= 20 {
        let upper_permitted = u32::from_le_bytes(bytes.get(12..16)?.try_into().ok()?);
        let upper_inheritable = u32::from_le_bytes(bytes.get(16..20)?.try_into().ok()?);
        mask |= u64::from(upper_permitted | upper_inheritable) << 32;
    }
    Some(mask)
}

fn capability_names(mask: u64) -> Vec<&'static str> {
    const NAMES: [&str; 41] = [
        "CAP_CHOWN",
        "CAP_DAC_OVERRIDE",
        "CAP_DAC_READ_SEARCH",
        "CAP_FOWNER",
        "CAP_FSETID",
        "CAP_KILL",
        "CAP_SETGID",
        "CAP_SETUID",
        "CAP_SETPCAP",
        "CAP_LINUX_IMMUTABLE",
        "CAP_NET_BIND_SERVICE",
        "CAP_NET_BROADCAST",
        "CAP_NET_ADMIN",
        "CAP_NET_RAW",
        "CAP_IPC_LOCK",
        "CAP_IPC_OWNER",
        "CAP_SYS_MODULE",
        "CAP_SYS_RAWIO",
        "CAP_SYS_CHROOT",
        "CAP_SYS_PTRACE",
        "CAP_SYS_PACCT",
        "CAP_SYS_ADMIN",
        "CAP_SYS_BOOT",
        "CAP_SYS_NICE",
        "CAP_SYS_RESOURCE",
        "CAP_SYS_TIME",
        "CAP_SYS_TTY_CONFIG",
        "CAP_MKNOD",
        "CAP_LEASE",
        "CAP_AUDIT_WRITE",
        "CAP_AUDIT_CONTROL",
        "CAP_SETFCAP",
        "CAP_MAC_OVERRIDE",
        "CAP_MAC_ADMIN",
        "CAP_SYSLOG",
        "CAP_WAKE_ALARM",
        "CAP_BLOCK_SUSPEND",
        "CAP_AUDIT_READ",
        "CAP_PERFMON",
        "CAP_BPF",
        "CAP_CHECKPOINT_RESTORE",
    ];
    NAMES
        .iter()
        .enumerate()
        .filter_map(|(bit, name)| (mask & (1u64 << bit) != 0).then_some(*name))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn account_check_detects_root_equivalent_and_empty_password() {
        let mut state = AuditState::new(25);
        check_accounts_from(
            "root:x:0:0:root:/root:/bin/bash\nbackup::0:0:backup:/tmp:/bin/sh\n",
            &mut state,
        );

        assert_eq!(count_severity(&state.findings, "Critical"), 1);
        assert_eq!(count_severity(&state.findings, "High"), 1);
        assert!(
            state
                .findings
                .iter()
                .any(|finding| finding.title.contains("backup"))
        );
    }

    #[test]
    fn ssh_parser_uses_global_first_value_and_ignores_match_blocks() {
        let directives = parse_global_sshd_directives(
            "PasswordAuthentication no\nPasswordAuthentication yes\nMatch User test\nPermitRootLogin yes\n",
        );

        assert_eq!(
            directives.get("passwordauthentication").map(String::as_str),
            Some("no")
        );
        assert!(!directives.contains_key("permitrootlogin"));
    }

    #[test]
    fn tcp_parser_returns_only_listening_ports() {
        let input = "  sl  local_address rem_address   st\n   0: 0100007F:0016 00000000:0000 0A\n   1: 0100007F:01BB 00000000:0000 01\n";
        let mut ports = HashSet::new();

        parse_listening_tcp_ports(input, &mut ports);

        assert_eq!(ports, HashSet::from([22]));
    }

    #[test]
    fn udp_parser_returns_bound_nonzero_ports() {
        let mut ports = HashSet::new();
        parse_bound_udp_ports(
            "  sl  local_address rem_address   st\n   0: 00000000:0035 00000000:0000 07\n   1: 00000000:0000 00000000:0000 07\n",
            &mut ports,
        );

        assert_eq!(ports, HashSet::from([53]));
    }

    #[test]
    fn duplicate_findings_are_collapsed() {
        let mut findings = vec![
            Finding::new("High", "Test", "Same", "Evidence", "Recommendation"),
            Finding::new("High", "Test", "Same", "Evidence", "Recommendation"),
        ];

        deduplicate_findings(&mut findings);

        assert_eq!(findings.len(), 1);
    }

    #[test]
    fn cron_spool_matching_uses_path_components() {
        assert!(is_cron_spool_path(Path::new(
            "/var/spool/cron/crontabs/alice"
        )));
        assert!(!is_cron_spool_path(Path::new("/var/spool/crondanger")));
    }

    #[test]
    fn sensitive_file_check_reports_unsafe_mode() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("passwd");
        fs::write(&path, "root:x:0:0:root:/root:/bin/sh\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o666)).unwrap();
        let mut state = AuditState::new(25);

        inspect_sensitive_path(&path, true, &mut state);

        assert!(state.findings.iter().any(|finding| {
            finding.category == "Permissions" && finding.title.contains("writable")
        }));
    }

    #[test]
    fn privileged_file_scan_does_not_follow_symlinks() {
        let directory = tempfile::tempdir().unwrap();
        let privileged = directory.path().join("tool");
        let unsafe_privileged = directory.path().join("unsafe-tool");
        fs::write(&privileged, "fixture").unwrap();
        fs::write(&unsafe_privileged, "fixture").unwrap();
        fs::set_permissions(&privileged, fs::Permissions::from_mode(0o4755)).unwrap();
        fs::set_permissions(&unsafe_privileged, fs::Permissions::from_mode(0o4777)).unwrap();
        std::os::unix::fs::symlink(&privileged, directory.path().join("tool-link")).unwrap();
        let mut state = AuditState::new(25);

        check_privileged_executables(&[directory.path()], &mut state);

        assert_eq!(state.findings.len(), 1);
        assert_eq!(state.inventory.len(), 1);
        assert!(state.inventory[0].item.contains("tool"));
        assert!(state.findings[0].title.contains("unsafe-tool"));
    }

    #[test]
    fn missing_optional_sensitive_file_is_not_an_error() {
        let directory = tempfile::tempdir().unwrap();
        let mut state = AuditState::new(25);

        inspect_sensitive_path(&directory.path().join("not-installed"), false, &mut state);

        assert!(state.diagnostics.is_empty());
    }

    #[test]
    fn capability_parser_decodes_lower_and_upper_sets() {
        let mut bytes = vec![0u8; 20];
        bytes[4..8].copy_from_slice(&(1u32 << 7).to_le_bytes());
        bytes[12..16].copy_from_slice(&(1u32 << (39 - 32)).to_le_bytes());

        let mask = parse_file_capability_mask(&bytes).unwrap();
        let names = capability_names(mask);

        assert!(names.contains(&"CAP_SETUID"));
        assert!(names.contains(&"CAP_BPF"));
    }

    #[test]
    fn complete_read_only_audit_respects_response_limits() {
        let response = audit(AuditRequest {
            include_suid: false,
            max_findings: 25,
        })
        .unwrap();

        assert!(response.findings.len() <= 25);
        assert!(response.inventory.len() <= 25);
    }
}
