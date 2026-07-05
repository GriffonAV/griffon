use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::scanner_quarantine::{Quarantine, manifest::QuarantineManifest};

fn check_virtual_path(path: &Path) -> Option<PathBuf> {
    if path.to_string_lossy().contains('!') {
        let s = path.to_string_lossy().into_owned();
        let parts: Vec<String> = s.split('!').map(|p| p.to_string()).collect();
        let real_path = PathBuf::from(&parts[0]);

        if real_path.exists() {
            Some(real_path)
        } else {
            log::warn!(
                "Real path {} does not exist for virtual path {}",
                real_path.display(),
                path.display()
            );
            None
        }
    } else {
        Some(path.to_path_buf())
    }
}

impl Quarantine {
    pub fn quarantine_file(&self, path: &PathBuf) -> Result<PathBuf, String> {
        let real_path = check_virtual_path(path);
        if real_path.is_none() {
            log::error!(
                "Failed to quarantine {}: real path does not exist",
                path.display()
            );
            return Err(format!(
                "Failed to quarantine {}: real path does not exist",
                path.display()
            ));
        }
        let real_path = real_path.unwrap();

        log::info!(
            "Quarantining file {} (real path {})",
            path.display(),
            real_path.display()
        );

        let bytes = match std::fs::read(&real_path) {
            Ok(b) => b,
            Err(e) => {
                log::error!("Failed to read file {}: {}", real_path.display(), e);
                return Err(format!(
                    "Failed to read file {}: {}",
                    real_path.display(),
                    e
                ));
            }
        };

        let hash = hex::encode(Sha256::digest(&bytes));

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let dest_name = format!("{}_{}.quarantined", timestamp, &hash[..16]);
        let dest_path = self.dir.join(dest_name.clone());
        let manifest_path = self.dir.join(dest_name.replace(".quarantined", ".json"));

        let manifest = QuarantineManifest {
            original_path: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()),
            quarantined_at: chrono::Utc::now().to_rfc3339(),
            original_hash: hash,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        std::fs::write(&manifest_path, manifest_json)
            .map_err(|e| format!("Failed to write manifest: {}", e))?;

        std::fs::rename(path, &dest_path)
            .or_else(|_| {
                std::fs::copy(path, &dest_path)
                    .and_then(|_| std::fs::remove_file(path))
                    .map(|_| ())
            })
            .map_err(|e| format!("Failed to move file to quarantine: {}", e))?;

        log::info!("Quarantined: {} → {}", path.display(), dest_path.display());

        Ok(dest_path)
    }

    pub fn quarantine_files(&self, paths: &[PathBuf]) -> Result<(), String> {
        for path in paths {
            self.quarantine_file(path)?;
        }
        Ok(())
    }
}
