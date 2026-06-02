use std::{collections::HashMap, path::Path};

use yara_x::{Compiler, Rules};

use crate::scanner_engine::{
    data_type::{Severity, Threat},
    yara_engine::threat_category::ThreatCategory,
};
use walkdir::WalkDir;

pub mod threat_category;

pub struct YaraEngine {
    rules: HashMap<threat_category::ThreatCategory, Rules>,
    rule_count: usize,
}

impl YaraEngine {
    pub fn load_from_dir(rules_dir: &str) -> Result<Self, yara_x::ScanError> {
        let mut compilers: HashMap<threat_category::ThreatCategory, Compiler> = HashMap::new();

        for cat in threat_category::ThreatCategory::all() {
            compilers.insert(cat.clone(), Compiler::new());
        }

        for entry in WalkDir::new(rules_dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if path.is_file() {
                let filename = path.file_stem().and_then(|s| s.to_str()).unwrap_or("other");

                let category = filename
                    .parse::<threat_category::ThreatCategory>()
                    .unwrap_or(threat_category::ThreatCategory::Other);

                if let Ok(source) = std::fs::read_to_string(path)
                    && let Err(e) = compilers
                        .get_mut(&category)
                        .unwrap()
                        .add_source(source.as_str())
                {
                    log::error!("Failed to compile rule file {}: {}", path.display(), e);
                }
            }
        }

        let mut rules: HashMap<threat_category::ThreatCategory, Rules> = HashMap::new();
        let mut total_rule_count = 0;

        for (cat, compiler) in compilers {
            let compiled = compiler.build();
            let count = compiled.iter().len();

            if count > 0 {
                total_rule_count += count;
                rules.insert(cat, compiled);
            }
        }

        Ok(YaraEngine {
            rules,
            rule_count: total_rule_count,
        })
    }

    // pub fn load_from_dir(rules_dir: &str) -> Result<Self, yara_x::ScanError> {
    //     let mut compiler = Compiler::new();
    //     let mut rules: HashMap<threat_category::ThreatCategory, Rules> = HashMap::new();

    //     let entries = WalkDir::new(rules_dir).into_iter().filter_map(|e| e.ok());
    //     for entry in entries {
    //         let path = entry.path();
    //         if path.is_file()
    //             && let Ok(source) = std::fs::read_to_string(path)
    //         {
    //             let _ = compiler.add_source(source.as_str());
    //         }
    //     }

    //     let compiled_rules = compiler.build();

    //     let rule_count = compiled_rules.iter().len();

    //     rules.insert(threat_category::ThreatCategory::Other, compiled_rules);

    //     Ok(YaraEngine { rules, rule_count })
    // }

    pub fn rule_count(&self) -> usize {
        self.rule_count
    }

    pub fn rule_count_by_category(&self, category: &ThreatCategory) -> usize {
        self.rules
            .get(category)
            .map(|r| r.iter().len())
            .unwrap_or(0)
    }

    pub fn scan_bytes(
        &self,
        path: &Path,
        data: &[u8],
        threat: ThreatCategory,
    ) -> Result<Vec<Threat>, yara_x::ScanError> {
        let rules = self.rules.get(&threat);
        if rules.is_none() {
            return Ok(Vec::new());
        }
        let mut scanner = yara_x::Scanner::new(rules.unwrap());

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

        let mut scanner = yara_x::Scanner::new(
            self.rules
                .get(&threat_category::ThreatCategory::Other)
                .unwrap(),
        );
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
