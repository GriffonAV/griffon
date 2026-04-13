use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateKind {
    File,
    Directory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerCandidate {
    pub path: String,
    pub name: String,
    pub category: String,
    pub kind: CandidateKind,
    pub size: u64,
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
    pub dry_run: bool,
    pub deleted_count: u64,
    pub deleted_bytes: u64,
    pub failed: Vec<DeleteFailure>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanerPluginRequest {
    pub function: String,
    pub payload: Option<serde_json::Value>,
}
