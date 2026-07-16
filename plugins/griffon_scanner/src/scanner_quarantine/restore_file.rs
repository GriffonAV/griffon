use std::path::PathBuf;

use crate::scanner_quarantine::{Quarantine, manifest::QuarantineManifest};

impl Quarantine {
    pub fn restore_file(&self, quarantined_name: &str) -> Result<PathBuf, String> {
        let dest_path = self.dir.join(quarantined_name);
        let manifest_path = self
            .dir
            .join(quarantined_name.replace(".quarantined", ".json"));

        let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|e| {
            format!(
                "Failed to read manifest [{}]: {}",
                manifest_path.display(),
                e
            )
        })?;
        let manifest: QuarantineManifest = serde_json::from_str(&manifest_json).map_err(|e| {
            format!(
                "Failed to parse manifest [{}]: {}",
                manifest_path.display(),
                e
            )
        })?;

        if let Some(parent) = manifest.original_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to recreate directory: {}", e))?;
        }

        std::fs::rename(&dest_path, &manifest.original_path)
            .or_else(|_| {
                std::fs::copy(&dest_path, &manifest.original_path)
                    .and_then(|_| std::fs::remove_file(&dest_path))
                    .map(|_| ())
            })
            .map_err(|e| format!("Failed to restore file: {}", e))?;

        std::fs::remove_file(&manifest_path).ok();

        log::info!("Restored: {}", manifest.original_path.display());
        Ok(manifest.original_path)
    }

    pub fn restore_files(&self, quarantined_names: &[String]) -> Result<Vec<PathBuf>, String> {
        if quarantined_names.is_empty() {
            return Err("No names provided for restore".into());
        }

        let mut restored = Vec::new();
        let mut errors = Vec::new();

        for name in quarantined_names {
            match self.restore_file(name) {
                Ok(path) => restored.push(path),
                Err(e) => {
                    log::warn!("Skipping {}: {}", name, e);
                    errors.push(format!("{}: {}", name, e));
                }
            }
        }

        if restored.is_empty() {
            return Err(format!(
                "Failed to restore all {} file(s): {}",
                quarantined_names.len(),
                errors.join("; ")
            ));
        }

        Ok(restored)
    }

    pub fn delete_quarantined_file(&self, quarantined_name: &str) -> Result<(), String> {
        let dest_path = self.dir.join(quarantined_name);
        let manifest_path = self
            .dir
            .join(quarantined_name.replace(".quarantined", ".json"));

        std::fs::remove_file(&dest_path)
            .map_err(|e| format!("Failed to delete quarantined file: {}", e))?;
        std::fs::remove_file(&manifest_path)
            .map_err(|e| format!("Failed to delete manifest file: {}", e))?;

        log::info!("Deleted: {}", dest_path.display());
        Ok(())
    }

    pub fn delete_quarantined_files(
        &self,
        quarantined_names: &[String],
    ) -> Result<Vec<String>, String> {
        if quarantined_names.is_empty() {
            return Err("No names provided for delete".into());
        }

        let mut deleted = Vec::new();
        let mut errors = Vec::new();

        for name in quarantined_names {
            match self.delete_quarantined_file(name) {
                Ok(()) => deleted.push(name.clone()),
                Err(e) => {
                    log::warn!("Skipping {}: {}", name, e);
                    errors.push(format!("{}: {}", name, e));
                }
            }
        }

        if deleted.is_empty() {
            return Err(format!(
                "Failed to delete all {} file(s): {}",
                quarantined_names.len(),
                errors.join("; ")
            ));
        }

        Ok(deleted)
    }
}
