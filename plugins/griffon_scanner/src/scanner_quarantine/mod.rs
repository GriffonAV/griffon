pub mod list_file;
pub mod manifest;
pub mod restore_file;

use std::path::{Path, PathBuf};

pub mod quarantine_file;

pub struct Quarantine {
    pub dir: PathBuf,
}

impl Quarantine {
    pub fn new(dir: &Path) -> std::io::Result<Self> {
        use std::os::unix::fs::PermissionsExt;

        std::fs::create_dir_all(dir)?;
        // Quarantined files are potentially malicious: keep them owner-only.
        std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))?;
        log::info!("Quarantine directory: {}", dir.display());
        Ok(Self {
            dir: dir.to_path_buf(),
        })
    }

    pub fn default_dir() -> PathBuf {
        crate::paths::default_quarantine_dir()
    }
}
