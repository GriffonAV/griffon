pub mod analysis;
pub mod api;
pub mod cache_paths;
pub mod config;
pub mod context;
pub mod modules;
pub mod reports;
pub mod runner;
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString, RVec, Tuple2},
};
pub use analysis::*;
pub use api::*;
pub use cache_paths::*;
use chrono::Utc;
pub use config::*;
pub use context::*;
pub use modules::CleanerModule;
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
pub use reports::*;
pub use runner::*;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub type CleanerResult<T> = Result<T, CleanerError>;

#[derive(thiserror::Error, Debug)]
pub enum CleanerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Walkdir error: {0}")]
    Walkdir(#[from] walkdir::Error),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub fn default_modules() -> Vec<Box<dyn CleanerModule>> {
    vec![
        Box::new(modules::cache::CacheCleaner::new()),
        // Box::new(modules::logs::LogsCleaner::new()),
        // Box::new(modules::packages::PackagesCleaner::new()),
        // Box::new(modules::bigfiles::BigfilesScanner::new()),
    ]
}

fn human_readable(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

pub fn print_cache_report(global: &GlobalReport) {
    let cache_report = match global.per_module.get("cache") {
        Some(r) => r,
        None => {
            println!("Aucun rapport pour le module 'cache'.");
            return;
        }
    };

    println!("=== CacheCleaner Report ===");
    println!("Dry-run : {}", global.dry_run);
    println!("Total fichiers : {}", cache_report.files_touched);
    println!(
        "Total libéré : {}",
        human_readable(cache_report.bytes_freed)
    );

    if !cache_report.per_root_path.is_empty() {
        println!("\nPar dossier :");
        let mut entries: Vec<_> = cache_report.per_root_path.iter().collect();
        entries.sort_by_key(|(path, _)| *path);

        for (path, stats) in entries {
            println!(
                "- {} : {} fichiers, {}",
                path,
                stats.files_touched,
                human_readable(stats.bytes_freed),
            );
        }
    }

    if !cache_report.per_file_type.is_empty() {
        println!("\nPar type :");

        let mut file_types: Vec<_> = cache_report.per_file_type.iter().collect();
        file_types.sort_by(|a, b| b.1.bytes_freed.cmp(&a.1.bytes_freed));

        for (file_type, stats) in file_types
            .into_iter()
            .filter(|(_, stats)| stats.files_touched >= 3 || stats.bytes_freed >= 1024 * 1024)
            .take(10)
        {
            println!(
                "- {} : {} fichiers, {}",
                file_type,
                stats.files_touched,
                human_readable(stats.bytes_freed),
            );
        }
    }

    if cache_report.permission_denied > 0 {
        println!(
            "\nPermission denied : {} fichiers (lancer en root pour tout nettoyer)",
            cache_report.permission_denied
        );
    }

    if !cache_report.warnings.is_empty() {
        println!("\nWarnings :");
        for w in &cache_report.warnings {
            println!("- {}", w);
        }
    }

    if !cache_report.errors.is_empty() {
        println!("\nErreurs :");
        for e in &cache_report.errors {
            println!("- {}", e);
        }
    }

    println!("Durée totale : {} ms", global.total_duration_ms);
    println!("Durée module cache : {} ms", cache_report.duration_ms);
    println!("===========================");
}

pub fn selected_cache_categories(cfg: &CleanerConfig) -> Vec<String> {
    let mut res = Vec::new();

    if cfg.enable_system_cache {
        res.push("system".to_string());
    }
    if cfg.enable_user_cache {
        res.push("user".to_string());
    }
    if cfg.enable_browser_cache {
        res.push("browser".to_string());
    }
    if cfg.enable_dev_cache {
        res.push("devtools".to_string());
    }
    if cfg.enable_package_cache {
        res.push("package_manager".to_string());
    }
    if cfg.enable_desktop_cache {
        res.push("desktop_env".to_string());
    }

    res
}

fn parse_arg(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }
    None
}

pub fn build_execution_context() -> CleanerResult<(ExecutionContext, String)> {
    let config_path =
        parse_arg("--config").unwrap_or_else(|| "bench/configs/light.json".to_string());

    let file_cfg = FileCleanerConfig::load_from_file(PathBuf::from(&config_path).as_path())?;

    let ctx = ExecutionContext {
        config: file_cfg.to_runtime_config(),
        dry_run: file_cfg.dry_run,
        root_paths: file_cfg.root_paths.iter().map(PathBuf::from).collect(),
    };

    ctx.config.validate()?;

    Ok((ctx, config_path))
}

pub fn execute_cleaner_payload() -> CleanerResult<CleanerExportPayload> {
    let (ctx, _config_path) = build_execution_context()?;
    let output_path =
        parse_arg("--output").unwrap_or_else(|| "griffon_cleaner_report.json".to_string());

    let modules = default_modules();
    let report = run_modules(&ctx, &modules)?;
    let analysis = build_analysis_report(&report);

    print_cache_report(&report);
    print_analysis_report(&analysis);

    if let Err(e) = write_analysis_report_to_file(&analysis, Path::new(&output_path)) {
        eprintln!("Erreur lors de l'export JSON de l'analyse : {:?}", e);
    } else {
        println!("Report exporté dans {}", output_path);
    }

    let selected_scope = CleanerSelectionSummary {
        profile: ctx.config.profile.as_str().to_string(),
        enabled_categories: selected_cache_categories(&ctx.config),
        dry_run: ctx.dry_run,
    };

    println!(
        "Selected cache categories: {:?}",
        selected_scope.enabled_categories
    );

    Ok(CleanerExportPayload {
        generated_at: Utc::now().to_rfc3339(),
        plugin_name: env!("CARGO_PKG_NAME").to_string(),
        plugin_version: env!("CARGO_PKG_VERSION").to_string(),
        run_id: Uuid::new_v4().to_string(),
        selected_scope,
        report,
        analysis,
    })
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(
        RString::from("author"),
        RString::from("Ewen Emeraud"),
    ));
    info.push(Tuple2(
        RString::from("name"),
        RString::from("Griffon Cleaner"),
    ));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Plugin Cleaner"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("run/list_candidates/delete_selected"),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    println!("[LIBCLEAN](msg) Received message: {}", msg.as_str());

    if msg.as_str() == "fn:run" {
        return match execute_cleaner_payload() {
            Ok(payload) => match serde_json::to_string(&payload) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR json serialize analysis: {e}")),
            },
            Err(err) => RString::from(format!("ERR cleaner: {}", err)),
        };
    }

    let req: CleanerPluginRequest = match serde_json::from_str(msg.as_str()) {
        Ok(req) => req,
        Err(e) => return RString::from(format!("ERR invalid request json: {e}")),
    };

    match req.function.as_str() {
        "run" => match execute_cleaner_payload() {
            Ok(payload) => match serde_json::to_string(&payload) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR json serialize analysis: {e}")),
            },
            Err(err) => RString::from(format!("ERR cleaner: {}", err)),
        },

        "list_candidates" => {
            let (ctx, _) = match build_execution_context() {
                Ok(v) => v,
                Err(e) => return RString::from(format!("ERR context: {}", e)),
            };

            let cleaner = modules::cache::CacheCleaner::new();

            match cleaner.collect_cache_candidates(&ctx) {
                Ok(items) => {
                    let resp = ListCandidatesResponse { ok: true, items };
                    match serde_json::to_string(&resp) {
                        Ok(json) => RString::from(json),
                        Err(e) => RString::from(format!("ERR json serialize: {e}")),
                    }
                }
                Err(e) => RString::from(format!("ERR list_candidates: {}", e)),
            }
        }

        "delete_selected" => {
            let payload = match req.payload {
                Some(value) => value,
                None => return RString::from("ERR missing payload"),
            };

            let delete_req: DeleteSelectedRequest = match serde_json::from_value(payload) {
                Ok(v) => v,
                Err(e) => {
                    return RString::from(format!("ERR invalid delete_selected payload: {e}"))
                }
            };

            let (ctx, _) = match build_execution_context() {
                Ok(v) => v,
                Err(e) => return RString::from(format!("ERR context: {}", e)),
            };

            let cleaner = modules::cache::CacheCleaner::new();
            let resp = cleaner.delete_selected_paths(&ctx, &delete_req.items);

            match serde_json::to_string(&resp) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR json serialize: {e}")),
            }
        }

        other => RString::from(format!("ERR unknown function: {other}")),
    }
}

#[export_root_module]
pub fn get_library() -> PluginRoot_Ref {
    PluginRoot {
        plugin: PluginI {
            init,
            handle_message,
        }
        .leak_into_prefix(),
    }
    .leak_into_prefix()
}
