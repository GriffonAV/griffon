//! Default on-disk locations used by the scanner.
//!
//! Installed builds run inside the daemon's system service (as root), so
//! per-user XDG locations would resolve to root's home directory. Release
//! builds therefore use fixed system paths instead.

use std::path::PathBuf;

/// Root directory for scanner state in installed (release) builds.
pub const SYSTEM_DATA_DIR: &str = "/var/lib/griffon/scanner";

/// Directory the YARA rules are loaded from and downloaded to.
pub fn default_rules_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("rules/")
    } else {
        PathBuf::from(SYSTEM_DATA_DIR).join("rules")
    }
}

/// Directory holding quarantined files and their manifests.
pub fn default_quarantine_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("quarantine")
    } else {
        PathBuf::from(SYSTEM_DATA_DIR).join("quarantine")
    }
}
