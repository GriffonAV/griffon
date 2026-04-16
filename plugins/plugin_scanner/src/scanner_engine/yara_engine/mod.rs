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
            println!("Loading YARA rule file: {}", entry.path().display());
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

    pub fn scan_bytes(&self, path: &Path, data: &[u8]) -> ScanResult {
        let mut scanner = yara_x::Scanner::new(&self.rules);

        match scanner.scan(data) {
            Ok(scan_results) => {
                let threats = scan_results
                    .matching_rules()
                    .map(|r| Threat {
                        name: r.identifier().to_string(),
                        severity: Severity::High,
                        matched_rule: "yara-x".to_string(),
                    })
                    .collect();

                ScanResult {
                    path: path.to_path_buf(),
                    threats,
                    error: None,
                }
            }
            Err(e) => ScanResult {
                path: path.to_path_buf(),
                threats: Vec::new(),
                error: Some(format!("Scan Error: {}", e)),
            },
        }
    }

    #[allow(dead_code)]
    pub fn scan(&self, path: &Path) -> ScanResult {
        let data = match std::fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                return ScanResult {
                    path: path.to_path_buf(),
                    threats: Vec::new(),
                    error: Some(format!("IO Error: {}", e)),
                };
            }
        };

        let mut scanner = yara_x::Scanner::new(&self.rules);

        match scanner.scan(&data) {
            Ok(scan_results) => {
                let threats = scan_results
                    .matching_rules()
                    .map(|r| Threat {
                        name: r.identifier().to_string(),
                        severity: Severity::High,
                        matched_rule: "yara-x".to_string(),
                    })
                    .collect();

                ScanResult {
                    path: path.to_path_buf(),
                    threats,
                    error: None,
                }
            }
            Err(e) => ScanResult {
                path: path.to_path_buf(),
                threats: Vec::new(),
                error: Some(format!("Scan Error: {}", e)),
            },
        }
    }
}
