use core::fmt;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    Low,
    #[default]
    Medium,
    High,
    #[allow(dead_code)]
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        };
        write!(f, "{}", s)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Threat {
    pub path: PathBuf,

    pub name: String,
    pub severity: Severity,
    pub matched_rule: String, // YARA rule name or "hash-db"
}

impl Default for Threat {
    fn default() -> Self {
        Threat {
            path: PathBuf::new(),
            name: String::new(),
            severity: Severity::Low,
            matched_rule: String::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResult {
    pub path: PathBuf,
    pub threats: Vec<Threat>,
    pub skipped: bool,
    pub error: Option<String>,
}

impl FileResult {
    pub fn clean(path: PathBuf) -> Self {
        Self {
            path,
            threats: vec![],
            skipped: false,
            error: None,
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScanReport {
    pub results: Vec<FileResult>,
    pub total_scanned: u64,
    pub total_skipped: u64,
    pub total_threats: u64,
    pub total_errors: u64,
}

impl ScanReport {
    pub fn add(&mut self, results: Vec<FileResult>) {
        for result in results {
            self.total_scanned += 1;
            if !result.threats.is_empty() {
                self.total_threats += 1;
            }
            if result.skipped {
                self.total_skipped += 1;
            }
            if result.error.is_some() {
                self.total_errors += 1;
            }
            self.results.push(result);
        }
    }

    #[allow(dead_code)]
    pub fn all_threats(&self) -> Vec<&Threat> {
        let mut threats: Vec<&Threat> =
            self.results.iter().flat_map(|r| r.threats.iter()).collect();
        threats.sort_by(|a, b| b.severity.cmp(&a.severity));
        threats
    }

    #[allow(dead_code)]
    pub fn errors(&self) -> Vec<&FileResult> {
        self.results.iter().filter(|r| r.error.is_some()).collect()
    }

    pub fn is_clean(&self) -> bool {
        self.total_threats == 0 && self.total_errors == 0
    }

    pub fn summary(&self) -> String {
        format!(
            "Scan complete: {} files scanned, {} threats found, {} errors, {} skipped",
            self.total_scanned, self.total_threats, self.total_errors, self.total_skipped
        )
    }
}
