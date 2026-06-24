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
        std::fs::create_dir_all(dir)?;
        log::info!("Quarantine directory: {}", dir.display());
        Ok(Self {
            dir: dir.to_path_buf(),
        })
    }

    pub fn default_dir() -> PathBuf {
        if cfg!(debug_assertions) {
            PathBuf::from("quarantine")
        } else {
            // is equal to ~/.local/share/griffon_scanner/quarantine
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("griffon_scanner")
                .join("quarantine")
        }
    }
}
