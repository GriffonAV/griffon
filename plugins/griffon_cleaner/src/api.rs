use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerCandidate {
    pub id: String,
    pub path: String,
    pub kind: CandidateKind,
    pub module: String,
    pub category: String,
    pub size: u64,
    pub selected_by_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCandidatesResponse {
    pub ok: bool,
    pub items: Vec<CleanerCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSelectedRequest {
    pub items: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFailure {
    pub path: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSelectedResponse {
    pub ok: bool,
    pub deleted_count: u64,
    pub deleted_bytes: u64,
    pub failed: Vec<DeleteFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerPluginRequest {
    pub function: String,
    pub payload: Option<serde_json::Value>,
}
