use std::path::Path;

use rayon::iter::{ParallelBridge, ParallelIterator};
use walkdir::{DirEntry, WalkDir};

use crate::scanner_engine::{ScanEngine, data_type::FileResult};

fn is_valid_entry(entry: &DirEntry) -> bool {
    let path = entry.path();
    let is_dir = entry.file_type().is_dir();

    if is_dir {
        if let Some(path_str) = path.to_str()
            && matches!(
                path_str,
                "/dev" | "/proc" | "/sys" | "/run" | "/mnt" | "/media" | "/lost+found"
            )
        {
            log::debug!("Skipping system directory: {}", path_str);
            return false;
        }

        let name = entry.file_name().to_string_lossy();
        if matches!(
            name.as_ref(),
            ".git"
                | ".svn"
                | ".hg"
                | "node_modules"
                | "target"
                | "vendor"
                | "__pycache__"
                | "build"
        ) {
            return false;
        }
    }

    true
}

impl ScanEngine {
    pub fn scan_dir(&self, root: &Path) -> Vec<FileResult> {
        if self.scan_args.threads != "off" {
            self.scan_dir_parallel(root)
        } else {
            self.scan_dir_sequential(root)
        }
    }

    pub fn scan_dir_parallel(&self, root: &Path) -> Vec<FileResult> {
        log::info!("Scanning directory {} in parallel", root.display());

        if self.scan_args.recursive {
            WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_entry(is_valid_entry)
                .par_bridge()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .flat_map(|e| self.scan_file(e.path()))
                .collect()
        } else {
            let Ok(entries) = std::fs::read_dir(root) else {
                log::error!("Failed to read directory: {}", root.display());
                return Vec::new();
            };

            entries
                .par_bridge()
                .filter_map(Result::ok)
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .flat_map(|e| self.scan_file(&e.path()))
                .collect()
        }
    }

    pub fn scan_dir_sequential(&self, root: &Path) -> Vec<FileResult> {
        log::info!("Scanning directory {} sequentially", root.display());

        if self.scan_args.recursive {
            WalkDir::new(root)
                .follow_links(false)
                .into_iter()
                .filter_entry(is_valid_entry)
                .filter_map(Result::ok)
                .filter(|e| e.file_type().is_file())
                .flat_map(|e| self.scan_file(e.path()))
                .collect()
        } else {
            let Ok(entries) = std::fs::read_dir(root) else {
                log::error!("Failed to read directory: {}", root.display());
                return Vec::new();
            };

            entries
                .filter_map(Result::ok)
                .filter(|e| e.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .flat_map(|e| self.scan_file(&e.path()))
                .collect()
        }
    }
}

// ################## unit tests ##################

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner_engine::ScanEngine;
    use crate::scanner_engine::scanargs::{PrepArgs, ScanArgs};

    fn setup_test_engine() -> ScanEngine {
        let mut engine = ScanEngine::new();
        let prep = PrepArgs::default();
        engine.prepare(&prep).expect("Engine prep failed in test");
        engine
    }

    #[test]
    fn test_directory_exclusion_ignores_node_modules() {
        let mut engine = setup_test_engine();

        engine.scan_args = ScanArgs {
            recursive: true,
            threads: "off".to_string(),
            ..Default::default()
        };

        let results = engine.scan_dir(Path::new("tests/fixtures/dummy_fs"));

        let scanned_node_modules = results
            .iter()
            .any(|r| r.path.to_string_lossy().contains("node_modules"));

        assert!(
            !scanned_node_modules,
            "Scanner must not traverse into node_modules directory"
        );
    }

    #[test]
    fn test_recursive_vs_non_recursive() {
        let mut engine = setup_test_engine();
        let target_dir = Path::new("tests/fixtures/dummy_fs");

        engine.scan_args = ScanArgs {
            recursive: false,
            threads: "off".to_string(),
            ..Default::default()
        };
        let results_flat = engine.scan_dir(target_dir);

        engine.scan_args.recursive = true;
        let results_deep = engine.scan_dir(target_dir);

        assert!(
            results_deep.len() > results_flat.len(),
            "Recursive scan should discover more files than non-recursive scan"
        );
    }
}
