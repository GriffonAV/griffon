use std::collections::HashMap;
use serde::Serialize;

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
        }
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct GlobalReport {
    pub dry_run: bool,
    pub total_files_touched: u64,
    pub total_bytes_freed: u64,
    pub per_module: HashMap<String, ModuleReport>,
}
