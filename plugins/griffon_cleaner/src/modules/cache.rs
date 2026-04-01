// src/modules/cache.rs

use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};
use crate::cache_paths::{KNOWN_CACHE_PATHS, expand_home, CacheCategory};
use std::{fs, io::ErrorKind};
use std::path::{Path};
use walkdir::WalkDir;
use std::collections::hash_map::Entry;
use crate::TypeStats;
use crate::PathStats;
use crate::Profile;

pub struct CacheCleaner;

impl CacheCleaner {
    pub fn new() -> Self {
        Self
    }

    fn is_root() -> bool {
        #[cfg(target_family = "unix")]
        {
            unsafe { libc::geteuid() == 0 }
        }

        #[cfg(not(target_family = "unix"))]
        {
            false
        }
    }

    fn default_cache_paths(ctx: &ExecutionContext) -> Vec<(String, std::path::PathBuf)> {
        let cfg = &ctx.config;
        let is_root = Self::is_root();

        KNOWN_CACHE_PATHS
            .iter()
            .filter(|cache| match cache.category {
                CacheCategory::System         => cfg.enable_system_cache,
                CacheCategory::User           => cfg.enable_user_cache,
                CacheCategory::Browser        => cfg.enable_browser_cache,
                CacheCategory::DevTools       => cfg.enable_dev_cache,
                CacheCategory::PackageManager => cfg.enable_package_cache,
                CacheCategory::DesktopEnv     => cfg.enable_desktop_cache,
            })
            .filter(|cache| crate::cache_paths::path_allowed_in_profile(cache, &cfg.profile))
            .filter(|cache| !cache.requires_root || is_root)
            .filter(|cache| !cache.dangerous || matches!(cfg.profile, Profile::Full))
            .filter_map(|cache| {
                expand_home(cache.pattern).map(|p| (cache.pattern.to_string(), p))
            })
            .collect()
    }

    fn default_cache_paths_with_logs(ctx: &ExecutionContext, report: &mut ModuleReport) -> Vec<(String, std::path::PathBuf)> {
        let cfg = &ctx.config;
        let is_root = Self::is_root();
        let mut out = Vec::new();

        for cache in KNOWN_CACHE_PATHS {
            let category_enabled = match cache.category {
                CacheCategory::System         => cfg.enable_system_cache,
                CacheCategory::User           => cfg.enable_user_cache,
                CacheCategory::Browser        => cfg.enable_browser_cache,
                CacheCategory::DevTools       => cfg.enable_dev_cache,
                CacheCategory::PackageManager => cfg.enable_package_cache,
                CacheCategory::DesktopEnv     => cfg.enable_desktop_cache,
            };

            if !category_enabled {
                continue;
            }

            if !crate::cache_paths::path_allowed_in_profile(cache, &cfg.profile) {
                report.warnings.push(format!(
                    "Path skipped by profile {:?}: {}",
                    cfg.profile, cache.pattern
                ));
                continue;
            }

            if cache.requires_root && !is_root {
                report.warnings.push(format!(
                    "Path skipped (requires root): {}",
                    cache.pattern
                ));
                continue;
            }

            if cache.dangerous && !matches!(cfg.profile, Profile::Full) {
                report.warnings.push(format!(
                    "Dangerous path skipped outside Full profile: {}",
                    cache.pattern
                ));
                continue;
            }

            if let Some(path) = expand_home(cache.pattern) {
                out.push((cache.pattern.to_string(), path));
            }
        }

        out
    }

    fn file_type_key(path: &Path) -> String {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.contains("Packages") || file_name.contains("InRelease") || file_name.contains("Translation-") {
            return "apt_index".to_string();
        }

        match path.extension().and_then(|ext| ext.to_str()) {
            Some(ext) if !ext.trim().is_empty() => ext.to_lowercase(),
            None => "no_ext".to_string(),
            _ => "unknown".to_string(),
        }
    }

    fn bump_permission_denied_from_walkdir(report: &mut ModuleReport, e: &walkdir::Error) {
        let is_permission_denied = e
            .io_error()
            .map(|io_err| io_err.kind() == std::io::ErrorKind::PermissionDenied)
            .unwrap_or(false)
            || e.to_string().to_lowercase().contains("permission denied");

        if is_permission_denied {
            report.permission_denied += 1;
        }
    }

    fn clean_cache_dir(
        &self,
        root_label: &str,
        path: &Path,
        dry_run: bool,
        report: &mut ModuleReport,
    ) -> CleanerResult<()> {
        if !path.exists() {
            report.missing_paths_count += 1;
            return Ok(());
        }

        report.existing_paths_count += 1;

        for entry_res in WalkDir::new(path).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    Self::bump_permission_denied_from_walkdir(report, &e);

                    report.warnings.push(format!(
                        "Erreur walkdir dans {}: {e}",
                        path.display()
                    ));
                    report.warning_count += 1;
                    continue;
                }
            };

            if entry.file_type().is_file() {
                let file_path = entry.path();
                report.candidate_files_count += 1;

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        Self::bump_permission_denied_from_walkdir(report, &e);

                        report.warnings.push(format!(
                            "Impossible de lire les métadonnées de {}: {e}",
                            file_path.display()
                        ));
                        report.warning_count += 1;
                        continue;
                    }
                };

                let size = metadata.len();

                if dry_run {
                    report.skipped_files_count += 1;
                    report.files_touched += 1;
                    report.bytes_freed += size;

                    Self::bump_type_stats(report, file_path, size);
                    Self::bump_root_stats(report, root_label, size);
                    continue;
                }

                match fs::remove_file(file_path) {
                    Ok(_) => {
                        report.deleted_files_count += 1;
                        report.files_touched += 1;
                        report.bytes_freed += size;

                        Self::bump_type_stats(report, file_path, size);
                        Self::bump_root_stats(report, root_label, size);
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::PermissionDenied {
                            report.permission_denied += 1;
                        }

                        report.warnings.push(format!(
                            "Impossible de supprimer {}: {e}",
                            file_path.display()
                        ));
                        report.warning_count += 1;
                    }
                }
            }
        }

        Ok(())
    }

    fn bump_root_stats(report: &mut ModuleReport, root_label: &str, size: u64) {
        match report.per_root_path.entry(root_label.to_string()) {
            Entry::Occupied(mut e) => {
                let stats = e.get_mut();
                stats.files_touched += 1;
                stats.bytes_freed += size;
            }
            Entry::Vacant(e) => {
                e.insert(PathStats {
                    files_touched: 1,
                    bytes_freed: size,
                });
            }
        }
    }

    fn bump_type_stats(report: &mut ModuleReport, file_path: &Path, size: u64) {
        let type_key = Self::file_type_key(file_path);

        match report.per_file_type.entry(type_key) {
            Entry::Occupied(mut e) => {
                let stats = e.get_mut();
                stats.files_touched += 1;
                stats.bytes_freed += size;
            }
            Entry::Vacant(e) => {
                e.insert(TypeStats {
                    files_touched: 1,
                    bytes_freed: size,
                });
            }
        }
    }
}

impl CleanerModule for CacheCleaner {
    fn id(&self) -> &'static str {
        "cache"
    }

    fn description(&self) -> &'static str {
        "Clean system and user cache directories."
    }

    fn run(&self, ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        let mut report = ModuleReport::empty(self.id());

        let cache_paths = Self::default_cache_paths_with_logs(ctx, &mut report);

        for (label, path) in cache_paths {
            if let Err(e) = self.clean_cache_dir(&label, &path, ctx.dry_run, &mut report) {
                report.warnings.push(format!(
                    "Erreur lors du nettoyage de {}: {e}",
                    path.display()
                ));
            }
        }

        Ok(report)
    }
}
