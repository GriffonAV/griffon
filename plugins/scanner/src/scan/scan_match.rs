use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanMatch {
    pub rule_name: String,
    pub tags: Vec<String>,
    pub meta: serde_json::Value,
}
