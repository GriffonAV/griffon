use clap::Parser;
use griffon_scanner::{
    scanner_engine::{
        ScanEngine,
        scanargs::{PrepArgs, ScanArgs},
    },
    scanner_quarantine::Quarantine,
    scanner_updater::ScannerUpdater,
};

fn main() {
    let mut scanner = ScanEngine::new();
    let quarantine =
        Quarantine::new(&Quarantine::default_dir()).expect("Failed to initialize quarantine");

    // shell

    while let Some(line) = std::io::stdin().lines().next() {
        let line = line.expect("Failed to read line");
        let mut parts: std::str::SplitWhitespace<'_> = line.split_whitespace();
        let command = parts.next().unwrap_or("");

        // let args = ScanArgs::parse();

        match command {
            "prepare" => {
                let args = PrepArgs::parse_from(std::iter::once(command).chain(parts));
                prep(&mut scanner, &args)
            }
            "scan" => {
                let args = ScanArgs::parse_from(std::iter::once(command).chain(parts));
                println!("Starting scan with args: {:?}", args);
                scan(&mut scanner, &args)
            }
            "db_update" => update_signatures(&mut scanner),
            "quarantine" => {
                let path_str = parts.next();
                if let Some(path) = path_str {
                    let pathbuf = std::path::Path::new(path).to_path_buf();
                    match quarantine.quarantine_file(&pathbuf) {
                        Ok(qpath) => println!("Quarantined to: {}", qpath.display()),
                        Err(e) => println!("Failed to quarantine: {}", e),
                    }
                } else {
                    println!("Usage: quarantine <file_path>");
                }
            }
            "q_list" => {
                let items = quarantine.list_sorted();
                if items.is_empty() {
                    println!("No quarantined items found.");
                } else {
                    for item in items {
                        println!(
                            "Quarantine Name: {}, Original Path: {}, Quarantined At: {}",
                            item.quarantine_name,
                            item.original_path.display(),
                            item.quarantined_at
                        );
                    }
                }
            }
            "q_restore" => {
                let file_name = parts.next();
                if let Some(name) = file_name {
                    match quarantine.restore_file(name) {
                        Ok(path) => println!("Restored to: {}", path.display()),
                        Err(e) => println!("Failed to restore: {}", e),
                    }
                } else {
                    println!("Usage: restore <quarantined_file_name>");
                }
            }
            "exit" | "quit" => break,
            _ => println!("Unknown command: {}", command),
        }
        print!("> ");
    }
}

fn prep(scanner: &mut ScanEngine, args: &PrepArgs) {
    if let Ok((hash_count, yara_count)) = scanner.prepare(args) {
        println!(
            "Scan engine prepared with {} hash signatures and {} YARA rules",
            hash_count, yara_count
        );
    } else {
        eprintln!("Failed to prepare scan engine");
    }
}

fn scan(scanner: &mut ScanEngine, args: &ScanArgs) {
    let report = scanner.scan(args);

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

fn update_signatures(scanner: &mut ScanEngine) {
    if let Err(e) = ScannerUpdater::default().update() {
        eprintln!("Failed to update signatures: {}", e);
    } else {
        log::info!("Signatures updated successfully");
    }

    let _ = scanner.prepare(&PrepArgs::default());
}
