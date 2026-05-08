/// bench_runner.rs — standalone binary for timed benchmarking of ScanEngine
///
/// Placed in src/bin/bench_runner.rs so it builds with your existing crate.
/// It times engine.prepare() and engine.scan() separately, then emits a
/// single JSON object to stdout that the Python harness can parse and enrich
/// with resource data.
///
/// Build:
///     cargo build --release --bin bench_runner
///
/// Run (Python will call this, but you can also run it manually):
///     ./target/release/bench_runner \
///         --target tests/fixtures/dirs/medium \
///         --threads auto \
///         --iterations 5
use std::env;
use std::path::Path;
use std::time::{Duration, Instant};

use serde::Serialize;

// Pull from your crate
use griffon_scanner::scanner_engine::ScanEngine;
use griffon_scanner::scanner_engine::scanargs::ScanArgs;

// ---------------------------------------------------------------------------
// Output schema — everything Python needs in one JSON blob
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct Timing {
    mean_ms: f64,
    min_ms: f64,
    max_ms: f64,
    stddev_ms: f64,
    iterations: usize,
    raw_ms: Vec<f64>,
}

#[derive(Serialize)]
struct PrepareStats {
    timing: Timing,
    hash_rules_loaded: i32,
    yara_rules_loaded: i32,
}

#[derive(Serialize)]
struct ScanStats {
    timing: Timing,
    target: String,
    /// Number of files scanned (extracted from last ScanReport)
    files_scanned: usize,
    threats_found: usize,
}

#[derive(Serialize)]
struct BenchOutput {
    target: String,
    threads: String,
    yara_only: bool,
    prepare: PrepareStats,
    scan: ScanStats,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn compute_timing(samples: &[f64]) -> Timing {
    let n = samples.len();
    let mean = samples.iter().sum::<f64>() / n as f64;
    let min = samples.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = samples.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let variance = samples.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / n as f64;
    Timing {
        mean_ms: round2(mean),
        min_ms: round2(min),
        max_ms: round2(max),
        stddev_ms: round2(variance.sqrt()),
        iterations: n,
        raw_ms: samples.iter().map(|x| round2(*x)).collect(),
    }
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

fn duration_ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2).find(|w| w[0] == flag).map(|w| w[1].clone())
}

fn flag_present(args: &[String], flag: &str) -> bool {
    args.iter().any(|a| a == flag)
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    let args: Vec<String> = env::args().collect();

    let target = arg_value(&args, "--target").unwrap_or_else(|| "/usr/bin".into());
    let threads = arg_value(&args, "--threads").unwrap_or_else(|| "auto".into());
    let iters: usize = arg_value(&args, "--iterations")
        .and_then(|s| s.parse().ok())
        .unwrap_or(3);
    let yara_only = flag_present(&args, "--yara-only");
    let rules_dir = arg_value(&args, "--rules-dir").unwrap_or_else(|| "rules/".into());

    // Build ScanArgs from CLI flags
    let scan_args = ScanArgs {
        threads: threads.clone(),
        yara_only,
        yara_rules: Some(rules_dir.into()),
        path: target.clone().into(),

        ..Default::default()
    };

    let target_path = Path::new(&target);
    if !target_path.exists() {
        eprintln!("ERR: target path does not exist: {}", target);
        std::process::exit(1);
    }

    // -----------------------------------------------------------------------
    // 1. Benchmark prepare() — run `iters` times, each with a fresh engine
    //    NOTE: because rayon's global pool uses OnceLock, thread pool config
    //    is set on the first call and reused. This is the real-world behaviour.
    // -----------------------------------------------------------------------
    let mut prepare_samples: Vec<f64> = Vec::with_capacity(iters);
    let mut hash_rules = 0i32;
    let mut yara_rules = 0i32;

    for i in 0..iters {
        let mut engine = ScanEngine::new();
        let t = Instant::now();
        match engine.prepare(&scan_args) {
            Ok((h, y)) => {
                prepare_samples.push(duration_ms(t.elapsed()));
                // Capture rule counts from last iteration
                if i == iters - 1 {
                    hash_rules = h;
                    yara_rules = y;
                }
            }
            Err(e) => {
                eprintln!("ERR: prepare() failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // -----------------------------------------------------------------------
    // 2. Benchmark scan() — engine is prepared once, scan is called `iters` times
    //    This isolates scan cost from engine startup cost.
    // -----------------------------------------------------------------------
    let mut scan_engine = ScanEngine::new();
    if let Err(e) = scan_engine.prepare(&scan_args) {
        eprintln!("ERR: prepare() for scan bench failed: {}", e);
        std::process::exit(1);
    }

    let mut scan_samples: Vec<f64> = Vec::with_capacity(iters);
    let mut files_scanned = 0usize;
    let mut threats_found = 0usize;

    for i in 0..iters {
        let t = Instant::now();
        let report = scan_engine.scan(target_path, &scan_args);
        scan_samples.push(duration_ms(t.elapsed()));

        // Capture counts from last iteration
        if i == iters - 1 {
            files_scanned = report.total_scanned as usize;
            threats_found = report.total_threats as usize;
        }
    }

    // -----------------------------------------------------------------------
    // 3. Emit JSON to stdout — Python harness reads this
    // -----------------------------------------------------------------------
    let output = BenchOutput {
        target: target.clone(),
        threads: threads.clone(),
        yara_only,
        prepare: PrepareStats {
            timing: compute_timing(&prepare_samples),
            hash_rules_loaded: hash_rules,
            yara_rules_loaded: yara_rules,
        },
        scan: ScanStats {
            timing: compute_timing(&scan_samples),
            target,
            files_scanned,
            threats_found,
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}
