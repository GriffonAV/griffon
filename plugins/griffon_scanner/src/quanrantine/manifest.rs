use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuarantineManifest {
    pub original_path: PathBuf,
    pub quarantined_at: String,
    // pub detections: Vec<String>,
    // pub severity: String,
    pub original_hash: String,
    // pub ruleset: String,
}
