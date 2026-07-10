use clap::Parser;
use griffon_scanner::scanner_engine::ScanEngine;
use griffon_scanner::scanner_engine::scanargs::{PrepArgs, ScanArgs};
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct BenchArgs {
    #[arg(long, required = true)]
    target: PathBuf,

    #[arg(long, default_value = "auto")]
    threads: String,
}

fn main() {
    let bench_args = BenchArgs::parse();

    let mut engine = ScanEngine::new();
    let prep = PrepArgs {
        yara_only: true,
        ..PrepArgs::default()
    };
    engine.prepare(&prep).expect("Engine prepare failed");

    let scan_args = ScanArgs {
        paths: vec![bench_args.target],
        threads: bench_args.threads,
        ..ScanArgs::default()
    };

    let report = engine.scan(&scan_args);

    println!(
        "{{\"time_taken\": {}, \"scanned_files\": {}}}",
        report.time_taken, report.total_scanned
    );
}
