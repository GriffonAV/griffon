#[allow(dead_code)]
pub mod manifest;

use crate::scanner_engine::data_type::{FileResult, Threat};
use manifest::QuarantineManifest;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

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
        /// Default location: ~/.local/share/griffon_scanner/quarantine
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("griffon_scanner")
            .join("quarantine")
    }
}
