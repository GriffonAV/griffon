use anyhow::Result;
use log::debug;
use std::path::Path;
use std::time::Instant;
use walkdir::WalkDir;

use std::sync::Arc;

use std::fs;
use yara_x::{Rules, Scanner};

use crate::scanner_engine::{database::RulesEngine, scan::scan_report::ScanReport};
// use crate::scan::ScanReport;

pub struct MultiThreadScanner {
    engine: Arc<RulesEngine>,
    _num_threads: usize,
}

impl MultiThreadScanner {
    pub fn new(engine: Arc<RulesEngine>) -> Result<Self> {
        Self::with_threads(engine, 0)
    }

    pub fn with_threads(engine: Arc<RulesEngine>, num_threads: usize) -> Result<Self> {
        let num_threads = if num_threads == 0 {
            num_cpus::get()
        } else {
            num_threads
        };

        Ok(Self {
            engine,
            _num_threads: num_threads,
        })
    }

    pub fn scan_bytes(&self, rules: &Rules, input: &[u8]) -> usize {
        let mut scanner = Scanner::new(rules);
        match scanner.scan(input) {
            Ok(results) => results.matching_rules().len(),
            Err(_) => 0,
        }
    }

    pub fn scan_file<P: AsRef<Path>>(&self, rules: &Rules, path: P) -> usize {
        match fs::read(path) {
            Ok(input) => self.scan_bytes(rules, &input),
            Err(_) => 0,
        }
    }

    pub fn scan_directory<P: AsRef<Path>>(&self, path: P) -> Result<ScanReport> {
        let scan_start = Instant::now();

        let mut scan_report = ScanReport::new("test".to_string());
        scan_report.scan_path = path.as_ref().to_string_lossy().to_string();
        scan_report.timestamp = chrono::Utc::now();
        let rules = match self.engine.select_rules(
            crate::scanner_engine::file_context::FileType::GenericBinary,
            crate::scanner_engine::file_context::ScanStage::Pre,
        ) {
            Some(r) => r,
            None => {
                return Err(anyhow::anyhow!(
                    "No rules found for the specified file type and scan stage"
                ));
            }
        };

        for entry in WalkDir::new("samples").into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let hits = self.scan_file(rules, entry.path());
                if hits > 0 {
                    debug!("[ALERT] {:?} matched {} rules", entry.path(), hits);
                    scan_report.infected_files += 1;
                } else {
                    scan_report.clean_files += 1;
                }
                scan_report.total_files += 1;
            }
        }

        scan_report.scan_duration_secs = scan_start.elapsed().as_secs_f64();
        Ok(scan_report)
    }
}
