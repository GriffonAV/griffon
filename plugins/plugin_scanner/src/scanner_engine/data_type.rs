use std::path::PathBuf;

#[derive(Default)]
pub enum Severity {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

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

pub struct ScanResult {
    pub number_file_scanned: u64,
    pub path: PathBuf,
    pub threats: Vec<Threat>,
    pub error: Option<String>,
}

impl Default for ScanResult {
    fn default() -> Self {
        ScanResult {
            number_file_scanned: 0,
            path: PathBuf::new(),
            threats: Vec::new(),
            error: None,
        }
    }
}
