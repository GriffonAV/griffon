use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CleanerFilters {
    #[serde(default)]
    pub file_types: Vec<String>,

    #[serde(default)]
    pub dry_run: Option<bool>,
}
impl CleanerFilters {
    pub fn has_file_type_filter(&self) -> bool {
        !self.file_types.is_empty()
    }

    pub fn matches_file_type(&self, file_type: &str) -> bool {
        if self.file_types.is_empty() {
            return true;
        }

        let normalized_file_type = normalize_file_type(file_type);

        self.file_types
            .iter()
            .map(|item| normalize_file_type(item))
            .any(|selected| selected == normalized_file_type)
    }
}

pub fn normalize_file_type(file_type: &str) -> String {
    file_type.trim().trim_start_matches('.').to_lowercase()
}

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
    pub file_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCandidatesResponse {
    pub ok: bool,
    pub items: Vec<CleanerCandidate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeleteSelectedRequest {
    #[serde(default)]
    pub items: Vec<String>,

    #[serde(default)]
    pub paths: Vec<String>,

    #[serde(default)]
    pub dry_run: Option<bool>,

    #[serde(default)]
    pub file_types: Vec<String>,
}

impl DeleteSelectedRequest {
    pub fn selected_paths(&self) -> Vec<String> {
        if !self.paths.is_empty() {
            return self.paths.clone();
        }

        self.items.clone()
    }

    pub fn to_filters(&self) -> CleanerFilters {
        CleanerFilters {
            file_types: self.file_types.clone(),
            dry_run: self.dry_run,
        }
    }
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
