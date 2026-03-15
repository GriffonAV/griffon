use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use yara_x::{Compiler, Rules, Scanner};

use log::debug;

pub mod file_context;
pub mod rules_engine;
pub mod scanner_engine;

pub fn load_yara_rules<P: AsRef<Path>>(dir: P) -> Rules {
    let mut compiler = Compiler::new();
    let mut loaded_count = 0;
    let mut error_count = 0;

    let benchmark_rule = r#"
        rule Benchmark_Test {
            strings:
                $a = "RUST_AV_BENCHMARK_PAYLOAD_SIGNATURE"
            condition:
                $a
        }
    "#;
    compiler
        .add_source(benchmark_rule)
        .expect("Failed to add internal benchmark rule");

    debug!("Scanning directory for rules...");
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();

        if path.is_file()
            && let Some(ext) = path.extension()
            && (ext == "yar" || ext == "yara")
        {
            match fs::read_to_string(path) {
                Ok(contents) => {
                    if compiler.add_source(contents.as_str()).is_ok() {
                        loaded_count += 1;
                    } else {
                        error_count += 1;
                    }
                }
                Err(_) => error_count += 1,
            }
        }
    }

    debug!("✅ Compilation complete.");
    debug!("   -> Loaded files: {}", loaded_count);
    debug!("   -> Skipped/Failed: {}", error_count);

    compiler.build()
}

pub fn scan_bytes(rules: &Rules, input: &[u8]) -> usize {
    let mut scanner = Scanner::new(rules);
    match scanner.scan(input) {
        Ok(results) => results.matching_rules().len(),
        Err(_) => 0,
    }
}

pub fn scan_file<P: AsRef<Path>>(rules: &Rules, path: P) -> usize {
    match fs::read(path) {
        Ok(input) => scan_bytes(rules, &input),
        Err(_) => 0,
    }
}
