use std::path::Path;

use crate::scanner_engine::{
    ScanEngine,
    archive::{ArchiveKind, detect_archive, extract::extract_entries},
    data_type::FileResult,
};

const MAX_DEPTH: u32 = 5;
const MAX_ENTRY_SIZE: usize = 100 * 1024 * 1024; // 100MB per entry

impl ScanEngine {
    pub fn scan_file(&self, path: &Path) -> Vec<FileResult> {
        let bytes = match std::fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                return vec![FileResult {
                    path: path.to_path_buf(),
                    threats: vec![],
                    skipped: false,
                    error: Some(e.to_string()),
                }];
            }
        };

        self.scan_bytes(path, &bytes, 0)
    }

    pub fn scan_bytes(&self, path: &Path, bytes: &[u8], depth: u32) -> Vec<FileResult> {
        log::info!("Scanning file {}", path.display());
        let mut results = vec![self.scan_entry(path, bytes)];

        if depth >= MAX_DEPTH {
            return results;
        }

        let kind = detect_archive(bytes);

        if matches!(kind, ArchiveKind::Unknown) {
            return results;
        }
        // only zip is supported for now
        if kind != ArchiveKind::Zip {
            log::warn!(
                "Detected archive type {:?} is not supported for scanning",
                kind
            );
            results[0].skipped = true;
            return results;
        }

        let entries = extract_entries(bytes, &kind);
        for entry in entries {
            if entry.bytes.len() > MAX_ENTRY_SIZE {
                log::warn!(
                    "Entry {} in archive {} exceeds maximum size limit",
                    entry.name,
                    path.display()
                );
                continue;
            }
            let virtual_path = Path::new(path)
                .to_path_buf()
                .join(format!("!/{}", entry.name));

            let mut entry_results = self.scan_bytes(&virtual_path, &entry.bytes, depth + 1);

            results.append(&mut entry_results);
        }

        results
    }

    fn scan_entry(&self, path: &Path, bytes: &[u8]) -> FileResult {
        let mut result = FileResult::clean(path.to_path_buf());

        if !self.scan_args.yara_only
            && let Some(db) = &self.hash_db
        {
            log::info!(
                "Scanning with hash database containing {} signatures",
                db.count()
            );
        }

        if let Some(rules) = &self.yara_rules {
            log::info!("Scanning with {} YARA rules", rules.rule_count());
            match rules.scan_bytes(path, bytes) {
                Ok(mut t) => {
                    result.threats.append(&mut t);
                }
                Err(e) => {
                    result.error = Some(format!("YARA scan error: {}", e));
                }
            }
        }
        result
    }
}
