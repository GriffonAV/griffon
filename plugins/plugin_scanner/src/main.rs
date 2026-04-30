mod scanner_engine;

use clap::Parser;

use crate::scanner_engine::ScanArgs;

fn main() {
    let args = ScanArgs::parse();

    let mut scanner = scanner_engine::ScanEngine::new();
    scanner
        .load_hash_db(&args)
        .expect("Failed to load signature DB");
    scanner
        .load_yara_rules(&args)
        .expect("Failed to load YARA rules");

    let result = scanner.scan(&args.path, &args);

    if result.threats.is_empty() {
        println!("Clean");
    } else {
        println!("{} is a threat:", result.path.display());
        println!("Number of files scanned: {}", result.number_file_scanned);
        for threat in &result.threats {
            println!(
                "THREAT: {} ({}) in {}",
                threat.name,
                threat.matched_rule,
                threat.path.display()
            );
        }
    }
    if !result.threats.is_empty() {
        std::process::exit(1);
    }
}
