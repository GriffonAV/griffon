// src/modules/cache.rs

use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};
use crate::cache_paths::{KNOWN_CACHE_PATHS, expand_home, CacheCategory};
use std::{fs};
use std::path::{Path};
use walkdir::WalkDir;
use std::collections::hash_map::Entry;
use crate::TypeStats;
use crate::PathStats;

pub struct CacheCleaner;

impl CacheCleaner {
    pub fn new() -> Self {
        Self
    }

    fn default_cache_paths(ctx: &ExecutionContext) -> Vec<(String, std::path::PathBuf)> {
        let cfg = &ctx.config;

        KNOWN_CACHE_PATHS
            .iter()
            .filter(|cache| match cache.category {
                CacheCategory::System       => cfg.enable_system_cache,
                CacheCategory::User         => cfg.enable_user_cache,
                CacheCategory::Browser      => cfg.enable_browser_cache,
                CacheCategory::DevTools     => cfg.enable_dev_cache,
                CacheCategory::PackageManager => cfg.enable_package_cache,
                CacheCategory::DesktopEnv   => cfg.enable_desktop_cache,
            })
            .filter_map(|cache| {
                expand_home(cache.pattern).map(|p| (cache.pattern.to_string(), p))
            })
            .collect()
    }

    fn file_type_key(path: &Path) -> String {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) if !ext.is_empty() => ext.to_string(),
            _ => "no_ext".to_string(),
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
            // Rien à faire si le chemin n'existe pas
            return Ok(());
        }

        // On parcourt récursivement le dossier
        for entry_res in WalkDir::new(path).into_iter() {
            let entry = match entry_res {
                Ok(e) => e,
                Err(e) => {
                    report.warnings.push(format!(
                        "Erreur walkdir dans {}: {e}",
                        path.display()
                    ));
                    continue;
                }
            };

            // On ne touche qu'aux fichiers, pas aux dossiers ici
            if entry.file_type().is_file() {
                let file_path = entry.path();

                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        report.warnings.push(format!(
                            "Impossible de lire les métadonnées de {}: {e}",
                            file_path.display()
                        ));
                        continue;
                    }
                };

                let size = metadata.len();

                if !dry_run {
                    if let Err(e) = fs::remove_file(file_path) {
                        report.warnings.push(format!(
                            "Impossible de supprimer {}: {e}",
                            file_path.display()
                        ));
                        continue;
                    }
                }

                report.files_touched += 1;
                report.bytes_freed += size;

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
        }

        Ok(())
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

        let cache_paths = Self::default_cache_paths(ctx);

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
