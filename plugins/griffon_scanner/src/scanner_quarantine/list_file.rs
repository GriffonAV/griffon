use crate::scanner_quarantine::{Quarantine, manifest::QuarantineManifest};

impl Quarantine {
    #[allow(dead_code)]
    pub fn list(&self) -> Vec<QuarantineManifest> {
        let Ok(entries) = std::fs::read_dir(&self.dir) else {
            return vec![];
        };

        entries
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .filter_map(|e| {
                let json = std::fs::read_to_string(e.path()).ok()?;
                serde_json::from_str(&json).ok()
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn list_sorted(&self) -> Vec<QuarantineManifest> {
        let mut manifests = self.list();
        manifests.sort_by(|a, b| b.quarantined_at.cmp(&a.quarantined_at)); // newest first
        manifests
    }
}
