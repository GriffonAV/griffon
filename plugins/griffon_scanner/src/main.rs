mod scanner_engine;

use clap::Parser;

use crate::scanner_engine::ScanArgs;

fn main() {
    let args = ScanArgs::parse();

    let mut scanner = scanner_engine::ScanEngine::new();
    scanner
        .prepare(&args)
        .expect("Failed to prepare the scan engine");

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
