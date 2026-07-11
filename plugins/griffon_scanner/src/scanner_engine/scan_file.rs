use std::path::Path;

use crate::scanner_engine::{
    ScanEngine,
    archive::{ArchiveKind, detect_archive, extract::extract_entries},
    data_type::FileResult,
};

const MAX_DEPTH: u32 = 5;
const MAX_ENTRY_SIZE: usize = 100 * 1024 * 1024; // 100MB per entry

const MAX_FILE_SIZE: u64 = 50 * 1024 * 1024;

impl ScanEngine {
    pub fn scan_file(&self, path: &Path) -> Vec<FileResult> {
        if let Ok(metadata) = std::fs::metadata(path)
            && metadata.len() > MAX_FILE_SIZE
        {
            log::debug!("Skipped {} (Exceeds 50MB limit)", path.display());
            return vec![FileResult {
                path: path.to_path_buf(),
                threats: vec![],
                skipped: true,
                error: Some("File exceeds maximum scan size".to_string()),
            }];
        }

        if let Ok(Some(kind)) = infer::get_from_path(path) {
            let mime = kind.mime_type();
            if mime.starts_with("video/")
                || mime.starts_with("audio/")
                || mime.starts_with("image/")
            {
                log::debug!("Skipped {} (Safe media type: {})", path.display(), mime);
                return vec![FileResult {
                    path: path.to_path_buf(),
                    threats: vec![],
                    skipped: true,
                    error: Some(format!("Skipped safe media type: {}", mime)),
                }];
            }
        }

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
            let active_categories = self.scan_args.get_active_categories();

            for category in active_categories {
                log::info!(
                    "Scanning with {} YARA rules for category {:?}",
                    rules.rule_count_by_category(&category),
                    category
                );
                match rules.scan_bytes(path, bytes, category) {
                    Ok(mut t) => {
                        result.threats.append(&mut t);
                    }
                    Err(e) => {
                        result.error = Some(format!("YARA scan error: {}", e));
                    }
                }
            }

            // log::info!("Scanning with {} YARA rules", rules.rule_count());
            // match rules.scan_bytes(path, bytes, ThreatCategory::Other) {
            //     Ok(mut t) => {
            //         result.threats.append(&mut t);
            //     }
            //     Err(e) => {
            //         result.error = Some(format!("YARA scan error: {}", e));
            //     }
            // }
        }
        result
    }
}

// ######################################## tests

#[cfg(test)]
mod tests {
    use crate::scanner_engine::ScanEngine;
    use crate::scanner_engine::scanargs::{PrepArgs, ScanArgs};
    use std::path::{Path, PathBuf};
    use std::sync::Once;

    use crate::scanner_updater::ScannerUpdater;

    static INIT: Once = Once::new();

    fn setup_test_engine() -> ScanEngine {
        INIT.call_once(|| {
            println!("[+] Initializing YARA rules for test suite...");
            let updater = ScannerUpdater::default();

            // In debug mode, this saves to the local "rules/" directory
            if let Err(e) = updater.update() {
                panic!("CRITICAL: Failed to download YARA rules for testing: {}", e);
            }
        });

        let mut engine = ScanEngine::new();
        let prep = PrepArgs::default();
        engine.prepare(&prep).expect("Engine prep failed in test");
        engine
    }

    #[test]
    fn test_clean_file_returns_no_threats() {
        let mut engine = setup_test_engine();

        let args = ScanArgs {
            paths: vec![PathBuf::from(file!())],
            ..Default::default()
        };

        let report = engine.scan(&args);

        assert_eq!(report.total_threats, 0, "Clean file should have 0 threats");
    }

    #[test]
    fn test_eicar_file_is_detected() {
        let mut engine = setup_test_engine();

        let args = ScanArgs {
            paths: vec![PathBuf::from("tests/fixtures/eicar.com")],
            ..Default::default()
        };

        let report = engine.scan(&args);

        assert!(report.total_threats > 0, "EICAR file must be detected");
    }

    #[test]
    fn test_eicar_zip_is_detected() {
        let mut engine = setup_test_engine();

        let args = ScanArgs {
            paths: vec![PathBuf::from("tests/fixtures/eicar.zip")],
            archives: true,
            ..Default::default()
        };

        let report = engine.scan(&args);

        assert!(
            report.total_threats > 0,
            "Scanner failed to detect EICAR inside the ZIP archive"
        );
    }

    #[test]
    fn test_eicar_nested_zip_is_detected() {
        let mut engine = setup_test_engine();
        let args = ScanArgs {
            paths: vec![PathBuf::from("tests/fixtures/eicar_nested.zip")],
            archives: true,
            ..Default::default()
        };

        let report = engine.scan(&args);

        assert!(
            report.total_threats > 0,
            "Scanner failed to recurse and detect EICAR inside the nested ZIP archive"
        );
    }

    #[test]
    fn test_large_file_is_skipped() {
        let engine = setup_test_engine();

        let path = Path::new("tests/fixtures/dummy_fs/large_file.dat");
        let results = engine.scan_file(path);

        assert_eq!(results.len(), 1, "Should return exactly one FileResult");
        assert!(
            results[0].skipped,
            "File > 50MB should be marked as skipped"
        );

        let error_msg = results[0].error.as_deref().unwrap_or("");
        assert!(
            error_msg.contains("maximum scan size"),
            "Should contain the size limit error message"
        );
    }

    #[test]
    fn test_media_file_is_skipped() {
        let engine = setup_test_engine();

        let path = Path::new("tests/fixtures/dummy_fs/media/image.png");
        let results = engine.scan_file(path);

        assert_eq!(results.len(), 1, "Should return exactly one FileResult");
        assert!(results[0].skipped, "Media file should be marked as skipped");

        let error_msg = results[0].error.as_deref().unwrap_or("");
        assert!(
            error_msg.contains("safe media type"),
            "Should contain the media type error message"
        );
    }
}
