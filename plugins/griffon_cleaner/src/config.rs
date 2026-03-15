// src/config.rs

#[derive(Debug, Clone)]
pub enum Profile {
    Safe,
    Full,
    Custom(CustomProfile),
}

#[derive(Debug, Clone)]
pub struct CustomProfile {
    pub enable_system_cache: bool,
    pub enable_user_cache: bool,
    pub enable_browser_cache: bool,
    pub enable_dev_cache: bool,
    pub enable_package_cache: bool,
    pub enable_desktop_cache: bool,
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