use serde::Deserialize;
use std::{fs, path::Path};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Profile {
    Safe,
    Full,
}

#[derive(Debug, Clone)]
pub struct CleanerConfig {
    pub profile: Profile,
    pub max_log_retention_days: u32,
    pub max_log_size_gb: f32,
    pub min_bigfile_size_mb: u64,
    pub enable_system_cache: bool,
    pub enable_user_cache: bool,
    pub enable_browser_cache: bool,
    pub enable_dev_cache: bool,
    pub enable_package_cache: bool,
    pub enable_desktop_cache: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileCleanerConfig {
    pub profile: Profile,
    pub dry_run: bool,
    pub root_paths: Vec<String>,
    pub max_log_retention_days: u32,
    pub max_log_size_gb: f32,
    pub min_bigfile_size_mb: u64,
    pub enable_system_cache: bool,
    pub enable_user_cache: bool,
    pub enable_browser_cache: bool,
    pub enable_dev_cache: bool,
    pub enable_package_cache: bool,
    pub enable_desktop_cache: bool,
}

impl FileCleanerConfig {
    pub fn load_from_file(path: &Path) -> Result<Self, crate::CleanerError> {
        let raw = fs::read_to_string(path)?;
        serde_json::from_str(&raw)
            .map_err(|e| crate::CleanerError::Config(format!("Invalid config JSON: {e}")))
    }

    pub fn to_runtime_config(&self) -> CleanerConfig {
        CleanerConfig {
            profile: self.profile.clone(),
            max_log_retention_days: self.max_log_retention_days,
            max_log_size_gb: self.max_log_size_gb,
            min_bigfile_size_mb: self.min_bigfile_size_mb,
            enable_system_cache: self.enable_system_cache,
            enable_user_cache: self.enable_user_cache,
            enable_browser_cache: self.enable_browser_cache,
            enable_dev_cache: self.enable_dev_cache,
            enable_package_cache: self.enable_package_cache,
            enable_desktop_cache: self.enable_desktop_cache,
        }
    }
}