// src/context.rs
use std::path::PathBuf;
use crate::CleanerConfig;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub config: CleanerConfig,
    pub dry_run: bool,
    pub root_paths: Vec<PathBuf>,
    // tu peux ajouter: logger, runtime handle, etc.
}
