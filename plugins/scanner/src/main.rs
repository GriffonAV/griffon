use anyhow::Result;
use log::debug;
use scanner::{
    load_yara_rules, scan_file,
    scanner_engine::{self, MultiThreadScanner},
};
use std::time::Instant;
use walkdir::WalkDir;

use scanner::file_context::{FileType, ScanStage};
use scanner::rules_engine::RulesEngine;
use scanner::scanner_engine::ScanReport;

use std::sync::Arc;

fn main() {
    env_logger::init();

    debug!("Loading rules...");
    let load_start = Instant::now();

    let mut engine = RulesEngine::from_dir("./rules").unwrap();
    engine.select_rules(FileType::GenericBinary, ScanStage::Pre);
    let engine: Arc<RulesEngine> = Arc::new(engine);

    let scanner_engine: MultiThreadScanner = MultiThreadScanner::new(engine.clone()).unwrap();

    debug!("Rules loaded in {:.2?}", load_start.elapsed());

    debug!("Scanning samples...");

    let path: &str = "samples";
    let result: Result<ScanReport> = scanner_engine.scan_directory(path);
    match result {
        Ok(report) => {
            debug!("Scan completed: \n{}", report);
        }
        Err(e) => {
            debug!("Error during scan: {}", e);
        }
    }
}
