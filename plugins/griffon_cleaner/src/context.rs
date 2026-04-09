// src/context.rs
use crate::CleanerConfig;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExecutionContext {
    pub config: CleanerConfig,
    pub dry_run: bool,
    pub root_paths: Vec<PathBuf>,
    // tu peux ajouter: logger, runtime handle, etc.
}
