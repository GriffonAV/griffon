use crate::api::{CandidateKind, CleanerCandidate, DeleteFailure, DeleteSelectedResponse};
use crate::cache_paths::{expand_home, CacheCategory, KNOWN_CACHE_PATHS};
use crate::PathStats;
use crate::Profile;
use crate::TypeStats;
use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};
use std::cmp::Reverse;
use std::collections::hash_map::Entry;
use std::path::{Path, PathBuf};
use std::{fs, io::ErrorKind};
use walkdir::WalkDir;

#[derive(Default)]
pub struct CacheCleaner;

impl CacheCleaner {
    pub fn new() -> Self {
        Self
    }

    fn is_root() -> bool {
        #[cfg(target_family = "unix")]
        {
            // SAFETY: libc::geteuid is safe to call here and has no preconditions.
            unsafe { libc::geteuid() == 0 }
        }

        #[cfg(not(target_family = "unix"))]
        {
            false
        }
    }

    fn default_cache_paths_with_logs(
        ctx: &ExecutionContext,
        report: &mut ModuleReport,
    ) -> Vec<(String, PathBuf)> {
        let cfg = &ctx.config;
        let is_root = Self::is_root();
        let mut out = Vec::new();

        for cache in KNOWN_CACHE_PATHS {
            let category_enabled = match cache.category {
                CacheCategory::System => cfg.enable_system_cache,
                CacheCategory::User => cfg.enable_user_cache,
                CacheCategory::Browser => cfg.enable_browser_cache,
                CacheCategory::DevTools => cfg.enable_dev_cache,
                CacheCategory::PackageManager => cfg.enable_package_cache,
                CacheCategory::DesktopEnv => cfg.enable_desktop_cache,
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
                report
                    .warnings
                    .push(format!("Path skipped (requires root): {}", cache.pattern));
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

    fn category_key(category: CacheCategory) -> &'static str {
        match category {
            CacheCategory::System => "system",
            CacheCategory::User => "user",
            CacheCategory::Browser => "browser",
            CacheCategory::DevTools => "devtools",
            CacheCategory::PackageManager => "package_manager",
            CacheCategory::DesktopEnv => "desktop_env",
        }
    }

    fn root_label_to_category(root_label: &str) -> &'static str {
        for cache in KNOWN_CACHE_PATHS {
            if cache.pattern == root_label {
                return Self::category_key(cache.category);
            }
        }
        "unknown"
    }

    fn file_type_key(path: &Path) -> String {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if file_name.contains("Packages")
            || file_name.contains("InRelease")
            || file_name.contains("Translation-")
        {
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

    fn dir_size(path: &Path) -> std::io::Result<u64> {
        let mut total = 0;

        for entry in WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }

    pub fn collect_cache_candidates(
        &self,
        ctx: &ExecutionContext,
    ) -> CleanerResult<Vec<CleanerCandidate>> {
        let mut items = Vec::new();
        let mut report = ModuleReport::empty(self.id());
        let cache_paths = Self::default_cache_paths_with_logs(ctx, &mut report);

        for (root_label, root_path) in cache_paths {
            if !root_path.exists() || !root_path.is_dir() {
                continue;
            }

            let category = Self::root_label_to_category(&root_label).to_string();

            let entries = match fs::read_dir(&root_path) {
                Ok(entries) => entries,
                Err(_) => continue,
            };

            for entry_res in entries {
                let entry = match entry_res {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                let path = entry.path();

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };

                let kind = if metadata.is_file() {
                    CandidateKind::File
                } else if metadata.is_dir() {
                    CandidateKind::Directory
                } else {
                    continue;
                };

                let size = if metadata.is_file() {
                    metadata.len()
                } else {
                    Self::dir_size(&path).unwrap_or(0)
                };

                if size == 0 {
                    continue;
                }

                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                items.push(CleanerCandidate {
                    path: path.display().to_string(),
                    name,
                    category: category.clone(),
                    kind,
                    size,
                });
            }
        }

        items.sort_by_key(|item| Reverse(item.size));
        Ok(items)
    }

    fn is_dangerous_delete_target(path: &Path) -> bool {
        let dangerous_paths = [
            Path::new("/"),
            Path::new("/bin"),
            Path::new("/boot"),
            Path::new("/dev"),
            Path::new("/etc"),
            Path::new("/home"),
            Path::new("/lib"),
            Path::new("/lib64"),
            Path::new("/opt"),
            Path::new("/proc"),
            Path::new("/root"),
            Path::new("/run"),
            Path::new("/sbin"),
            Path::new("/srv"),
            Path::new("/sys"),
            Path::new("/tmp"),
            Path::new("/usr"),
            Path::new("/var"),
            Path::new("/var/cache"),
            Path::new("/var/lib"),
            Path::new("/var/log"),
        ];

        dangerous_paths.contains(&path)
    }

    fn canonicalize_allowed_roots(allowed_roots: &[PathBuf]) -> Vec<PathBuf> {
        allowed_roots
            .iter()
            .filter_map(|root| root.canonicalize().ok())
            .collect()
    }

    fn validate_delete_target(
        raw_path: &Path,
        allowed_roots: &[PathBuf],
    ) -> Result<PathBuf, String> {
        if raw_path.as_os_str().is_empty() {
            return Err("Empty path is not allowed".to_string());
        }

        if !raw_path.exists() {
            return Err("Path does not exist".to_string());
        }

        let link_metadata =
            fs::symlink_metadata(raw_path).map_err(|e| format!("symlink metadata failed: {e}"))?;

        if link_metadata.file_type().is_symlink() {
            return Err("Refusing to delete symbolic link".to_string());
        }

        let canonical_path = raw_path
            .canonicalize()
            .map_err(|e| format!("canonicalize failed: {e}"))?;

        if Self::is_dangerous_delete_target(&canonical_path) {
            return Err("Refusing to delete dangerous top-level/system path".to_string());
        }

        let canonical_allowed_roots = Self::canonicalize_allowed_roots(allowed_roots);

        if canonical_allowed_roots.is_empty() {
            return Err("No valid allowed cleaner roots found".to_string());
        }

        let is_allowed = canonical_allowed_roots
            .iter()
            .any(|root| canonical_path.starts_with(root));

        if !is_allowed {
            return Err("Path is outside allowed cleaner scope".to_string());
        }

        if canonical_allowed_roots
            .iter()
            .any(|root| &canonical_path == root)
        {
            return Err("Refusing to delete an entire cleaner root directly".to_string());
        }

        Ok(canonical_path)
    }

    pub fn delete_selected_paths(
        &self,
        ctx: &ExecutionContext,
        items: &[String],
    ) -> DeleteSelectedResponse {
        let mut deleted_count = 0;
        let mut deleted_bytes = 0;
        let mut failed = Vec::new();

        let mut report = ModuleReport::empty(self.id());
        let allowed_roots: Vec<PathBuf> = Self::default_cache_paths_with_logs(ctx, &mut report)
            .into_iter()
            .map(|(_, path)| path)
            .collect();

        for item in items {
            let raw_path = Path::new(item);

            let path = match Self::validate_delete_target(raw_path, &allowed_roots) {
                Ok(path) => path,
                Err(error) => {
                    failed.push(DeleteFailure {
                        path: item.clone(),
                        error,
                    });
                    continue;
                }
            };

            let metadata = match fs::metadata(&path) {
                Ok(m) => m,
                Err(e) => {
                    failed.push(DeleteFailure {
                        path: item.clone(),
                        error: format!("metadata failed: {e}"),
                    });
                    continue;
                }
            };

            let size = if metadata.is_file() {
                metadata.len()
            } else if metadata.is_dir() {
                match Self::dir_size(&path) {
                    Ok(v) => v,
                    Err(e) => {
                        failed.push(DeleteFailure {
                            path: item.clone(),
                            error: format!("dir_size failed: {e}"),
                        });
                        continue;
                    }
                }
            } else {
                failed.push(DeleteFailure {
                    path: item.clone(),
                    error: "Unsupported path type".to_string(),
                });
                continue;
            };

            if ctx.dry_run {
                deleted_count += 1;
                deleted_bytes += size;
                continue;
            }

            let delete_result = if metadata.is_file() {
                fs::remove_file(&path)
            } else {
                fs::remove_dir_all(&path)
            };

            match delete_result {
                Ok(_) => {
                    deleted_count += 1;
                    deleted_bytes += size;
                }
                Err(e) => {
                    failed.push(DeleteFailure {
                        path: item.clone(),
                        error: format!("delete failed: {e}"),
                    });
                }
            }
        }

        DeleteSelectedResponse {
            ok: failed.is_empty(),
            dry_run: ctx.dry_run,
            deleted_count,
            deleted_bytes,
            failed,
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

                    report
                        .warnings
                        .push(format!("Erreur walkdir dans {}: {e}", path.display()));
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
