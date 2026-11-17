use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use crate::{AuditState, Finding, InventoryItem};

use super::{mode_string, read_small};

const MAX_ACCOUNTS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Account {
    name: String,
    uid: u32,
    gid: u32,
    home: PathBuf,
    shell: String,
}

pub(super) fn run(state: &mut AuditState) {
    let passwd = match read_small(Path::new("/etc/passwd")) {
        Ok(passwd) => passwd,
        Err(error) => {
            state.diagnostic(format!("Could not read /etc/passwd: {error}"));
            return;
        }
    };
    let accounts = parse_passwd(&passwd);
    check_duplicate_uids(&accounts, state);
    check_interactive_accounts(&accounts, state);
    check_shadow_policy(Path::new("/etc/shadow"), &accounts, state);
    check_login_defs(Path::new("/etc/login.defs"), state);
}

fn parse_passwd(contents: &str) -> Vec<Account> {
    contents
        .lines()
        .take(MAX_ACCOUNTS)
        .filter_map(|line| {
            let fields: Vec<_> = line.splitn(7, ':').collect();
            if fields.len() != 7 {
                return None;
            }
            Some(Account {
                name: fields[0].to_string(),
                uid: fields[2].parse().ok()?,
                gid: fields[3].parse().ok()?,
                home: PathBuf::from(fields[5]),
                shell: fields[6].to_string(),
            })
        })
        .collect()
}

fn is_interactive_shell(shell: &str) -> bool {
    !shell.is_empty()
        && !matches!(
            shell.rsplit('/').next().unwrap_or(shell),
            "false" | "nologin" | "sync" | "shutdown" | "halt"
        )
}

fn check_duplicate_uids(accounts: &[Account], state: &mut AuditState) {
    let mut by_uid: HashMap<u32, Vec<&str>> = HashMap::new();
    for account in accounts {
        by_uid.entry(account.uid).or_default().push(&account.name);
    }
    for (uid, names) in by_uid {
        if names.len() < 2 || uid == 65_534 {
            continue;
        }
        state.push(Finding::new(
            if uid == 0 { "Critical" } else { "Medium" },
            "Accounts",
            format!("Multiple accounts share UID {uid}"),
            format!("accounts: {}", names.join(", ")),
            "Assign unique UIDs unless the shared identity is explicitly documented and monitored.",
        ));
    }
}

fn check_interactive_accounts(accounts: &[Account], state: &mut AuditState) {
    for account in accounts
        .iter()
        .filter(|account| is_interactive_shell(&account.shell))
    {
        state.push_inventory(InventoryItem::new(
            "Accounts",
            &account.name,
            format!(
                "uid={}, gid={}, home={}, shell={}",
                account.uid,
                account.gid,
                account.home.display(),
                account.shell
            ),
        ));
        inspect_home(account, state);
    }
}

fn inspect_home(account: &Account, state: &mut AuditState) {
    if !account.home.is_absolute() || account.home == Path::new("/") {
        return;
    }
    let metadata = match fs::symlink_metadata(&account.home) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            state.push(Finding::new(
                "Low",
                "Accounts",
                format!("Interactive account {} has no home directory", account.name),
                account.home.display().to_string(),
                "Create the intended home directory or assign a non-interactive shell if login is not required.",
            ));
            return;
        }
        Err(error) => {
            state.diagnostic(format!(
                "Could not inspect home for {}: {error}",
                account.name
            ));
            return;
        }
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        state.push(Finding::new(
            "Medium",
            "Accounts",
            format!("Home path for {} is not a regular directory", account.name),
            account.home.display().to_string(),
            "Verify the home path and avoid unexpected symbolic links for interactive accounts.",
        ));
        return;
    }
    if metadata.uid() != account.uid && account.uid != 0 {
        state.push(Finding::new(
            "Medium",
            "Accounts",
            format!("Home directory ownership does not match {}", account.name),
            format!(
                "{} owner UID {}, expected {}",
                account.home.display(),
                metadata.uid(),
                account.uid
            ),
            "Restore ownership after confirming that the account and path are legitimate.",
        ));
    }
    if metadata.mode() & 0o022 != 0 {
        state.push(Finding::new(
            "Medium",
            "Accounts",
            format!(
                "Home directory for {} is writable by group or others",
                account.name
            ),
            format!("{} mode {}", account.home.display(), mode_string(&metadata)),
            "Remove unnecessary group/other write permissions from the home directory.",
        ));
    }

    inspect_ssh_path(account, &account.home.join(".ssh"), true, state);
    inspect_ssh_path(
        account,
        &account.home.join(".ssh/authorized_keys"),
        false,
        state,
    );
}

fn inspect_ssh_path(
    account: &Account,
    path: &Path,
    expect_directory: bool,
    state: &mut AuditState,
) {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return,
        Err(error) => {
            state.diagnostic(format!("Could not inspect {}: {error}", path.display()));
            return;
        }
    };
    if metadata.file_type().is_symlink()
        || (expect_directory && !metadata.is_dir())
        || (!expect_directory && !metadata.is_file())
    {
        state.push(Finding::new(
            "Medium",
            "SSH",
            format!("Unexpected SSH path type for {}", account.name),
            path.display().to_string(),
            "Replace unexpected links or path types with account-owned SSH files and directories.",
        ));
        return;
    }
    if metadata.uid() != account.uid || metadata.mode() & 0o022 != 0 {
        state.push(Finding::new(
            "High",
            "SSH",
            format!("Unsafe SSH path permissions for {}", account.name),
            format!(
                "{} owner UID {}, mode {}",
                path.display(),
                metadata.uid(),
                mode_string(&metadata)
            ),
            "Require account ownership and remove group/other write permissions.",
        ));
    }
}

fn check_shadow_policy(path: &Path, accounts: &[Account], state: &mut AuditState) {
    let contents = match read_small(path) {
        Ok(contents) => contents,
        Err(error) => {
            state.diagnostic(format!("Could not read {}: {error}", path.display()));
            return;
        }
    };
    let interactive: HashSet<_> = accounts
        .iter()
        .filter(|account| is_interactive_shell(&account.shell))
        .map(|account| account.name.as_str())
        .collect();

    for line in contents.lines().take(MAX_ACCOUNTS) {
        let fields: Vec<_> = line.split(':').collect();
        if fields.len() < 5 || !interactive.contains(fields[0]) {
            continue;
        }
        let username = fields[0];
        let password = fields[1];
        if password.is_empty() {
            state.push(Finding::new(
                "Critical",
                "Accounts",
                format!("Interactive account {username} has no password hash"),
                format!("{username} has an empty password field in /etc/shadow"),
                "Lock the account or set a strong authentication credential immediately.",
            ));
            continue;
        }
        if password.starts_with('!') || password.starts_with('*') {
            continue;
        }
        if let Ok(max_days) = fields[4].parse::<u32>()
            && max_days > 365
        {
            state.push(Finding::new(
                "Low",
                "Password policy",
                format!("Password expiry is very long for {username}"),
                format!("maximum password age: {max_days} days"),
                "Set an expiry policy that matches organizational risk and authentication controls.",
            ));
        }
    }
}

fn check_login_defs(path: &Path, state: &mut AuditState) {
    let Ok(contents) = read_small(path) else {
        return;
    };
    let values = parse_login_defs(&contents);
    let summary = [
        "PASS_MAX_DAYS",
        "PASS_MIN_DAYS",
        "PASS_WARN_AGE",
        "UMASK",
        "ENCRYPT_METHOD",
    ]
    .into_iter()
    .filter_map(|key| values.get(key).map(|value| format!("{key}={value}")))
    .collect::<Vec<_>>()
    .join(", ");
    if !summary.is_empty() {
        state.push_inventory(InventoryItem::new(
            "Password policy",
            "login.defs defaults",
            summary,
        ));
    }

    if let Some(value) = values.get("PASS_MAX_DAYS")
        && value
            .parse::<i64>()
            .is_ok_and(|days| !(0..=365).contains(&days))
    {
        state.push(Finding::new(
            "Low",
            "Password policy",
            "Default maximum password age is weak",
            format!("PASS_MAX_DAYS={value}"),
            "Choose an account lifecycle policy appropriate to the authentication model.",
        ));
    }
    if let Some(value) = values.get("ENCRYPT_METHOD")
        && matches!(value.to_ascii_uppercase().as_str(), "DES" | "MD5")
    {
        state.push(Finding::new(
            "High",
            "Password policy",
            "Legacy password hashing is configured",
            format!("ENCRYPT_METHOD={value}"),
            "Use a modern password hashing method supported by the distribution.",
        ));
    }
    if let Some(value) = values.get("UMASK")
        && let Ok(mask) = u32::from_str_radix(value.trim_start_matches('0'), 8)
        && mask & 0o022 != 0o022
    {
        state.push(Finding::new(
            "Low",
            "Accounts",
            "Default login umask permits group or other writes",
            format!("UMASK={value}"),
            "Use at least 022, or a stricter mask such as 027, where compatible.",
        ));
    }
}

fn parse_login_defs(contents: &str) -> HashMap<String, String> {
    contents
        .lines()
        .filter_map(|line| {
            let line = line.split('#').next().unwrap_or_default().trim();
            let mut fields = line.split_whitespace();
            let key = fields.next()?;
            let value = fields.next()?;
            Some((key.to_string(), value.to_string()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwd_parser_and_duplicate_uid_check_are_bounded_and_defensive() {
        let accounts = parse_passwd(
            "root:x:0:0:root:/root:/bin/bash\nadmin:x:0:0:admin:/home/admin:/bin/sh\nsvc:x:10:10:svc:/srv:/usr/sbin/nologin\n",
        );
        let mut state = AuditState::new(25);

        check_duplicate_uids(&accounts, &mut state);

        assert_eq!(accounts.len(), 3);
        assert!(
            state
                .findings
                .iter()
                .any(|finding| finding.severity == "Critical")
        );
    }

    #[test]
    fn login_defs_parser_ignores_comments() {
        let values =
            parse_login_defs("# PASS_MAX_DAYS 1\nPASS_MAX_DAYS 99999 # default\nUMASK 022\n");

        assert_eq!(
            values.get("PASS_MAX_DAYS").map(String::as_str),
            Some("99999")
        );
        assert_eq!(values.get("UMASK").map(String::as_str), Some("022"));
    }
}
