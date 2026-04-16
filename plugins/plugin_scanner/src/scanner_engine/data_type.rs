use std::path::PathBuf;

#[allow(dead_code)]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

pub struct Threat {
    pub name: String,

    #[allow(dead_code)]
    pub severity: Severity,
    pub matched_rule: String, // YARA rule name or "hash-db"
}

pub struct ScanResult {
    #[allow(dead_code)]
    pub path: PathBuf,
    pub threats: Vec<Threat>,
    #[allow(dead_code)]
    pub error: Option<String>,
}
