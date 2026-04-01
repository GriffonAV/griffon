use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct PathStats {
    pub files_touched: u64,
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeStats {
    pub files_touched: u64,
    pub bytes_freed: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleReport {
    pub module_id: String,
    pub files_touched: u64,
    pub bytes_freed: u64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub permission_denied: u64,
    pub per_root_path: HashMap<String, PathStats>,
    pub per_file_type: HashMap<String, TypeStats>,

    pub candidate_files_count: u64,
    pub deleted_files_count: u64,
    pub skipped_files_count: u64,
    pub missing_paths_count: u64,
    pub existing_paths_count: u64,
    pub duration_ms: u128,
    pub warning_count: u64,
}

impl ModuleReport {
    pub fn empty(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            files_touched: 0,
            bytes_freed: 0,
            warnings: Vec::new(),
            errors: Vec::new(),
            permission_denied: 0,
            per_root_path: HashMap::new(),
            per_file_type: HashMap::new(),
            candidate_files_count: 0,
            deleted_files_count: 0,
            skipped_files_count: 0,
            missing_paths_count: 0,
            existing_paths_count: 0,
            duration_ms: 0,
            warning_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GlobalReport {
    pub dry_run: bool,
    pub per_module: HashMap<String, ModuleReport>,
    pub total_files_touched: u64,
    pub total_bytes_freed: u64,
    pub total_warnings: u64,
    pub total_errors: u64,
    pub total_permission_denied: u64,
    pub total_duration_ms: u128,
}
