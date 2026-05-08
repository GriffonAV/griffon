mod scanner_engine;
mod scanner_quarantine;
mod scanner_updater;

use clap::Parser;
use logger::Logger;

use crate::scanner_engine::scanargs::ScanArgs;

static SCANNER_LOGGER: Logger = Logger::new(
    "GRIFFON_SCANNER",
    logger::LogLevel::Debug,
    Some("~/.local/share/griffon_scanner/scanner.log"),
);

fn main() {
    let args = ScanArgs::parse();

    SCANNER_LOGGER.debug(format!("Starting engine with args: {:?}", args));

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
