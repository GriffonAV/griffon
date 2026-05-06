mod quanrantine;
mod scanner_engine;

use clap::Parser;

use crate::{quanrantine::Quarantine, scanner_engine::scanargs::ScanArgs};

fn main() {
    env_logger::init();
    let quarantine =
        Quarantine::new(&Quarantine::default_dir()).expect("Failed to initialize quarantine");
    // test "samples/sample_00000.bin" for quarantine
    // let path = quarantine.quarantine_file(&"samples/test.rs".into());
    log::info!("Quarantine initialized at {}", quarantine.dir.display());
    let result = quarantine.restore_file("20260506_074736_e3b0c44298fc1c14.quarantined");
    if let Err(e) = result {
        log::error!("Failed to restore file: {}", e);
    } else {
        log::info!("File restored successfully");
    }

    return;
    let args = ScanArgs::parse();

    let mut scanner = scanner_engine::ScanEngine::new();
    let (hash_count, yara_count) = scanner
        .prepare(&args)
        .expect("Failed to prepare the scan engine");
    log::info!(
        "Scan engine prepared with {} hash signatures and {} YARA rules",
        hash_count,
        yara_count
    );

    let report = scanner.scan(&args.path, &args);

    if report.is_clean() {
        println!("Clean");
    } else {
        println!("{}", report.summary());
        for file_result in &report.results {
            if !file_result.threats.is_empty() {
                println!("{} is a threat:", file_result.path.display());
                for threat in &file_result.threats {
                    println!(
                        "THREAT: {} ({}) in {}",
                        threat.name,
                        threat.matched_rule,
                        threat.path.display()
                    );
                }
            }
        }
    }

    if !report.is_clean() {
        std::process::exit(1);
    }
}
