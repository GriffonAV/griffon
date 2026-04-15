use std::path::PathBuf;

pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct Threat {
    pub name: String,
    pub severity: Severity,
    pub matched_rule: String, // YARA rule name or "hash-db"
}

pub struct ScanResult {
    pub path: PathBuf,
    pub threats: Vec<Threat>,
    pub error: Option<String>,
}
