use super::scan_match::ScanMatch;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum ScanResult {
    Clean {
        file_path: String,
        file_type: String,
        scan_duration_ms: u64,
    },
    Infected {
        file_path: String,
        file_type: String,
        matches: Vec<ScanMatch>,
        scan_duration_ms: u64,
        threat_level: u32,
    },
    Error {
        file_path: String,
        error: String,
        scan_duration_ms: u64,
    },
    Skipped {
        file_path: String,
        reason: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub scan_path: String,
    pub total_files: usize,
    pub infected_files: usize,
    pub clean_files: usize,
    pub errors: usize,
    pub skipped_files: usize,
    pub scan_duration_secs: f64,
    pub results: Vec<ScanResult>,
}

impl ScanReport {
    pub fn new(scan_path: String) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            scan_path,
            total_files: 0,
            infected_files: 0,
            clean_files: 0,
            errors: 0,
            skipped_files: 0,
            scan_duration_secs: 0.0,
            results: Vec::new(),
        }
    }
}

impl fmt::Display for ScanReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Scan Report for {}", self.scan_path)?;
        writeln!(f, "Timestamp: {}", self.timestamp)?;
        writeln!(f, "Total Files Scanned: {}", self.total_files)?;
        writeln!(f, "Infected Files: {}", self.infected_files)?;
        writeln!(f, "Clean Files: {}", self.clean_files)?;
        writeln!(f, "Errors: {}", self.errors)?;
        writeln!(f, "Skipped Files: {}", self.skipped_files)?;
        writeln!(f, "Scan Duration (secs): {:.2}", self.scan_duration_secs)?;
        writeln!(f, "Detailed Results:")?;
        for result in &self.results {
            writeln!(f, "{:?}", result)?;
        }
        Ok(())
    }
}
