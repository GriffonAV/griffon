use std::path::Path;

use yara_x::{Compiler, Rules};

use crate::scanner_engine::data_type::{ScanResult, Severity, Threat};
use walkdir::WalkDir;

pub struct YaraEngine {
    rules: Rules,
    rule_count: usize,
}

impl YaraEngine {
    pub fn load_from_dir(rules_dir: &str) -> Result<Self, yara_x::ScanError> {
        let mut compiler = Compiler::new();
        let mut rule_count = 0;

        let entries = WalkDir::new(rules_dir).into_iter().filter_map(|e| e.ok());
        for entry in entries {
            log::debug!("Loading YARA rule file: {}", entry.path().display());
            let path = entry.path();
            if path.is_file()
                && let Ok(source) = std::fs::read_to_string(path)
                && let Ok(_) = compiler.add_source(source.as_str())
            {
                rule_count += 1;
            }
        }
        let rules = compiler.build();
        Ok(YaraEngine { rules, rule_count })
    }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }

    pub fn scan_bytes(&self, path: &Path, data: &[u8]) -> Result<Vec<Threat>, yara_x::ScanError> {
        let mut scanner = yara_x::Scanner::new(&self.rules);

        match scanner.scan(data) {
            Ok(scan_results) => {
                log::info!(
                    "YARA scan found {} matching rules in file {}",
                    scan_results.matching_rules().count(),
                    path.display()
                );
                let threats: Vec<Threat> = scan_results
                    .matching_rules()
                    .map(|r| Threat {
                        path: path.to_path_buf(),
                        name: r.identifier().to_string(),
                        severity: Severity::High,
                        matched_rule: "yara_x".to_string(),
                    })
                    .collect();

                Ok(threats)
            }
            Err(e) => {
                log::error!("YARA scan error for file {}: {}", path.display(), e);
                Err(e)
            }
        }
    }

    #[allow(dead_code)]
    pub fn scan(&self, path: &Path) -> Result<Vec<Threat>, yara_x::ScanError> {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                log::error!("Failed to read file {}: {}", path.display(), e);
                return Err(yara_x::ScanError::OpenError {
                    path: path.to_path_buf(),
                    err: e,
                });
            }
        };

        let mut scanner = yara_x::Scanner::new(&self.rules);
        match scanner.scan(&data) {
            Ok(scan_results) => {
                log::info!(
                    "YARA scan found {} matching rules in file {}",
                    scan_results.matching_rules().count(),
                    path.display()
                );
                let threats = scan_results
                    .matching_rules()
                    .map(|r| Threat {
                        path: path.to_path_buf(),
                        name: r.identifier().to_string(),
                        severity: Severity::High,
                        matched_rule: r.identifier().to_string(),
                    })
                    .collect();

                Ok(threats)
            }
            Err(e) => {
                log::error!("YARA scan error for file {}: {}", path.display(), e);
                Err(e)
            }
        }
    }
}
