mod scanner_engine;
use scanner_engine::database::RulesEngine;
use scanner_engine::file_context::{FileType, ScanStage};
use scanner_engine::hash_scanner::SignatureDb;
use scanner_engine::scan::{MultiThreadScanner, ScanReport};

use anyhow::Result;
use log::debug;
use std::path::Path;
use std::time::Instant;

use std::sync::Arc;

use crate::scanner_engine::hash_scanner::hash::scan_file;

fn main() {
    // env_logger::init();

    // debug!("Loading rules...");
    // let load_start = Instant::now();

    // let engine = RulesEngine::from_dir("./rules").unwrap();
    // engine.select_rules(FileType::GenericBinary, ScanStage::Pre);
    // let engine: Arc<RulesEngine> = Arc::new(engine);

    // let scanner_engine: MultiThreadScanner = MultiThreadScanner::new(engine.clone()).unwrap();

    // debug!("Rules loaded in {:.2?}", load_start.elapsed());

    // debug!("Scanning samples...");

    // let path: &str = "samples";
    // let result: Result<ScanReport> = scanner_engine.scan_directory(path);
    // match result {
    //     Ok(report) => {
    //         println!("Scan completed: \n{}", report);
    //     }
    //     Err(e) => {
    //         println!("Error during scan: {}", e);
    //     }
    // }

    let db =
        SignatureDb::load(Path::new("signatures/hashes.txt")).expect("Failed to load signature DB");

    println!("Loaded {} hashes", db.len());

    let result = scan_file(Path::new("eicar_files/eicar.com.txt"), &db);

    if result.threats.is_empty() {
        println!("Clean");
    } else {
        for threat in &result.threats {
            println!("THREAT: {} ({})", threat.name, threat.matched_rule);
        }
    }
}
