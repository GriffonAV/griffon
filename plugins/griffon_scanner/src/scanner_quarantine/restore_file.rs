use std::path::PathBuf;

use crate::scanner_quarantine::{Quarantine, manifest::QuarantineManifest};

impl Quarantine {
    pub fn restore_file(&self, quarantined_name: &str) -> Result<PathBuf, String> {
        let dest_path = self.dir.join(quarantined_name);
        let manifest_path = self
            .dir
            .join(quarantined_name.replace(".quarantined", ".json"));

        let manifest_json = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read manifest: {}", e))?;
        let manifest: QuarantineManifest = serde_json::from_str(&manifest_json)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

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
}
