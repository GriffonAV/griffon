use std::path::Path;

use rayon::iter::{ParallelBridge, ParallelIterator};
use walkdir::WalkDir;

use crate::scanner_engine::{ScanEngine, data_type::FileResult};

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
                .par_bridge() // Actually enables Rayon parallelism
                .filter_map(Result::ok) // Silently skips permission errors/unreadable entries
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
