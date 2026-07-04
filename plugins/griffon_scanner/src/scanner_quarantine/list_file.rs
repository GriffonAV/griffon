use crate::scanner_quarantine::{Quarantine, manifest::QuarantineManifest};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
pub struct QuarantineItem {
    pub quarantine_name: String,
    pub original_path: PathBuf,
    pub quarantined_at: String,
}

impl Quarantine {
    fn list(&self) -> Vec<QuarantineItem> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return vec![];
        };

        entries
            .filter_map(Result::ok)
            // Look for manifest json files
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .filter_map(|e| {
                let path = e.path();

                // Extract the file name stem (e.g., "20260704_214704_e3b0c44298fc1c14")
                let file_stem = path.file_stem()?.to_str()?;
                let quarantine_name = format!("{}.quarantined", file_stem);

                // Read and deserialize the manifest contents
                let json = std::fs::read_to_string(&path).ok()?;
                let manifest: QuarantineManifest = serde_json::from_str(&json).ok()?;

                // Combine both into our beautiful UI-friendly structure
                Some(QuarantineItem {
                    quarantine_name,
                    original_path: manifest.original_path,
                    quarantined_at: manifest.quarantined_at,
                })
            })
            .collect()
    }

    pub fn list_sorted(&self) -> Vec<QuarantineItem> {
        let mut manifests = self.list();
        manifests.sort_by(|a, b| b.quarantined_at.cmp(&a.quarantined_at)); // newest first
        manifests
    }
}
