pub mod analysis;
pub mod api;
pub mod cache_paths;
pub mod config;
pub mod context;
pub mod front_report;
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
pub use front_report::*;
pub use modules::CleanerModule;
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
pub use reports::*;
pub use runner::*;
use std::cmp::Reverse;
use std::path::{Path, PathBuf};
use uuid::Uuid;
use logger::{LogLevel, Logger};

pub type CleanerResult<T> = Result<T, CleanerError>;

static LOGGER_CLEANER: Logger = if cfg!(debug_assertions) {
    Logger::new("PLUGIN-CLEANER", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "DAEMON-INTERFACE-NETWORK",
        LogLevel::Debug,
        Some("/var/log/griffon/griffon_cleaner.log"),
    )
};

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
        Box::new(modules::docker::DockerCleaner::new()),
        // Box::new(modules::logs::LogsCleaner::new()),
        // Box::new(modules::packages::PackagesCleaner::new()),
        // Box::new(modules::bigfiles::BigfilesScanner::new()),
    ]
}

pub fn print_module_summary(global: &GlobalReport) {
    println!("\n=== Cleaner Modules Summary ===");
    println!("Dry-run : {}", global.dry_run);
    println!("Total touched : {}", global.total_files_touched);
    println!(
        "Total reclaimable/freed : {}",
        human_readable(global.total_bytes_freed)
    );
    println!("Total warnings : {}", global.total_warnings);
    println!("Total errors : {}", global.total_errors);
    println!(
        "Total permission denied : {}",
        global.total_permission_denied
    );
    println!("Total duration : {} ms", global.total_duration_ms);

    let mut modules: Vec<_> = global.per_module.iter().collect();
    modules.sort_by_key(|(_, report)| std::cmp::Reverse(report.bytes_freed));

    for (module_id, report) in modules {
        println!("\n--- Module: {} ---", module_id);
        println!("Touched : {}", report.files_touched);
        println!("Freed/Reclaimable : {}", human_readable(report.bytes_freed));
        println!("Warnings : {}", report.warnings.len());
        println!("Errors : {}", report.errors.len());
        println!("Permission denied : {}", report.permission_denied);
        println!("Duration : {} ms", report.duration_ms);

        for warning in report.warnings.iter().take(5) {
            println!("Warning: {}", warning);
        }

        for error in report.errors.iter().take(5) {
            println!("Error: {}", error);
        }
    }

    println!("===============================");
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
        file_types.sort_by_key(|file_type| Reverse(file_type.1.bytes_freed));

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
    let default_config_path = if cfg!(debug_assertions) {
        "bench/configs/light.json"
    } else {
        "/etc/griffon/plugins/griffon_cleaner/config.json"
    };

    let config_path = parse_arg("--config").unwrap_or_else(|| default_config_path.to_string());

    LOGGER_CLEANER.debug(format!(
        "Building execution context with config path: {}",
        config_path
    ));

    let file_cfg = match FileCleanerConfig::load_from_file(Path::new(&config_path)) {
        Ok(cfg) => {
            LOGGER_CLEANER.debug("Cleaner config file loaded successfully");
            cfg
        }
        Err(e) => {
            LOGGER_CLEANER.error(format!(
                "Failed to load cleaner config from {}: {}",
                config_path, e
            ));
            return Err(e);
        }
    };

    let ctx = ExecutionContext {
        config: file_cfg.to_runtime_config(),
        dry_run: file_cfg.dry_run,
        root_paths: file_cfg.root_paths.iter().map(PathBuf::from).collect(),
    };

    if let Err(e) = ctx.config.validate() {
        LOGGER_CLEANER.error(format!("Cleaner config validation failed: {}", e));
        return Err(e);
    }

    LOGGER_CLEANER.info(format!(
        "Execution context ready: profile={}, dry_run={}, root_paths={}",
        ctx.config.profile.as_str(),
        ctx.dry_run,
        ctx.root_paths.len()
    ));

    Ok((ctx, config_path))
}
pub fn execute_cleaner_payload() -> CleanerResult<CleanerExportPayload> {
    LOGGER_CLEANER.info("Starting cleaner payload execution");

    let (ctx, config_path) = build_execution_context()?;

    let output_path =
        parse_arg("--output").unwrap_or_else(|| "griffon_cleaner_report.json".to_string());

    LOGGER_CLEANER.debug(format!(
        "Cleaner execution parameters: config={}, output={}",
        config_path, output_path
    ));

    let modules = default_modules();

    LOGGER_CLEANER.debug(format!(
        "Loaded {} cleaner module(s)",
        modules.len()
    ));

    let report = match run_modules(&ctx, &modules) {
        Ok(report) => {
            LOGGER_CLEANER.info(format!(
                "Cleaner modules completed: touched={}, bytes={}, warnings={}, errors={}, permission_denied={}, duration_ms={}",
                report.total_files_touched,
                report.total_bytes_freed,
                report.total_warnings,
                report.total_errors,
                report.total_permission_denied,
                report.total_duration_ms
            ));
            report
        }
        Err(e) => {
            LOGGER_CLEANER.error(format!("Cleaner modules execution failed: {}", e));
            return Err(e);
        }
    };

    let analysis = build_analysis_report(&report);

    LOGGER_CLEANER.debug("Analysis report built successfully");

    print_cache_report(&report);
    print_module_summary(&report);
    print_analysis_report(&analysis);

    if let Err(e) = write_analysis_report_to_file(&analysis, Path::new(&output_path)) {
        LOGGER_CLEANER.error(format!(
            "Failed to export cleaner analysis report to {}: {:?}",
            output_path, e
        ));
    } else {
        LOGGER_CLEANER.info(format!(
            "Cleaner analysis report exported to {}",
            output_path
        ));
    }

    let selected_scope = CleanerSelectionSummary {
        profile: ctx.config.profile.as_str().to_string(),
        enabled_categories: selected_cache_categories(&ctx.config),
        dry_run: ctx.dry_run,
    };

    LOGGER_CLEANER.debug(format!(
        "Selected cache categories: {:?}",
        selected_scope.enabled_categories
    ));

    let generated_at = Utc::now().to_rfc3339();
    let run_id = Uuid::new_v4().to_string();

    LOGGER_CLEANER.info(format!(
        "Cleaner payload execution completed: run_id={}",
        run_id
    ));

    Ok(CleanerExportPayload {
        generated_at,
        plugin_name: env!("CARGO_PKG_NAME").to_string(),
        plugin_version: env!("CARGO_PKG_VERSION").to_string(),
        run_id,
        selected_scope,
        report,
        analysis,
    })
}
pub fn execute_cleaner_front_payload() -> CleanerResult<FrontCleanerPayload> {
    LOGGER_CLEANER.info("Building front cleaner payload");

    let raw_payload = execute_cleaner_payload()?;
    let front_payload = build_front_cleaner_payload(&raw_payload);

    LOGGER_CLEANER.info("Front cleaner payload built successfully");

    Ok(front_payload)
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    LOGGER_CLEANER.debug("Plugin init called");
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
        RString::from("run/run_raw/run_front/list_candidates/delete_selected"),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("123e4567-e23b-12d3-a456-426614174000"),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    let raw = msg.as_str().trim();

    LOGGER_CLEANER.debug(format!("Received plugin message: {}", raw));

    match raw {
        "fn:run" | "run" | "fn:run_front" | "run_front" => {
            LOGGER_CLEANER.debug("Matched direct command: run_front");
            LOGGER_CLEANER.info("Command received: run_front");

            return match execute_cleaner_front_payload() {
                Ok(payload) => {
                    LOGGER_CLEANER.debug("Front payload generated successfully");

                    match serde_json::to_string(&payload) {
                        Ok(json) => {
                            LOGGER_CLEANER.debug("Front payload serialized successfully");
                            LOGGER_CLEANER.info("run_front completed successfully");
                            RString::from(json)
                        }
                        Err(e) => {
                            LOGGER_CLEANER.debug(format!(
                                "Front payload serialization failed: {}",
                                e
                            ));
                            LOGGER_CLEANER.error(format!(
                                "Failed to serialize front payload: {}",
                                e
                            ));
                            RString::from(format!("ERR json serialize front payload: {e}"))
                        }
                    }
                }
                Err(err) => {
                    LOGGER_CLEANER.debug(format!("execute_cleaner_front_payload failed: {}", err));
                    LOGGER_CLEANER.error(format!("Cleaner run_front failed: {}", err));
                    RString::from(format!("ERR cleaner: {}", err))
                }
            };
        }

        "fn:run_raw" | "run_raw" => {
            LOGGER_CLEANER.debug("Matched direct command: run_raw");
            LOGGER_CLEANER.info("Command received: run_raw");

            return match execute_cleaner_payload() {
                Ok(payload) => {
                    LOGGER_CLEANER.debug("Raw cleaner payload generated successfully");

                    match serde_json::to_string(&payload) {
                        Ok(json) => {
                            LOGGER_CLEANER.debug("Raw cleaner payload serialized successfully");
                            LOGGER_CLEANER.info("run_raw completed successfully");
                            RString::from(json)
                        }
                        Err(e) => {
                            LOGGER_CLEANER.debug(format!(
                                "Raw payload serialization failed: {}",
                                e
                            ));
                            LOGGER_CLEANER.error(format!(
                                "Failed to serialize raw payload: {}",
                                e
                            ));
                            RString::from(format!("ERR json serialize raw payload: {e}"))
                        }
                    }
                }
                Err(err) => {
                    LOGGER_CLEANER.debug(format!("execute_cleaner_payload failed: {}", err));
                    LOGGER_CLEANER.error(format!("Cleaner run_raw failed: {}", err));
                    RString::from(format!("ERR cleaner: {}", err))
                }
            };
        }

        "fn:list_candidates" | "list_candidates" => {
            LOGGER_CLEANER.debug("Matched direct command: list_candidates");
            LOGGER_CLEANER.info("Command received: list_candidates");

            let (ctx, _) = match build_execution_context() {
                Ok(v) => {
                    LOGGER_CLEANER.debug("Execution context built for list_candidates");
                    v
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!(
                        "build_execution_context failed for list_candidates: {}",
                        e
                    ));
                    LOGGER_CLEANER.error(format!(
                        "Failed to build context for list_candidates: {}",
                        e
                    ));
                    return RString::from(format!("ERR context: {}", e));
                }
            };

            LOGGER_CLEANER.debug("Creating CacheCleaner for list_candidates");
            let cleaner = modules::cache::CacheCleaner::new();

            return match cleaner.collect_cache_candidates(&ctx) {
                Ok(items) => {
                    LOGGER_CLEANER.debug(format!(
                        "collect_cache_candidates returned {} item(s)",
                        items.len()
                    ));

                    let resp = ListCandidatesResponse { ok: true, items };

                    LOGGER_CLEANER.info(format!(
                        "Found {} cache candidate(s)",
                        resp.items.len()
                    ));

                    match serde_json::to_string(&resp) {
                        Ok(json) => {
                            LOGGER_CLEANER.debug("list_candidates response serialized successfully");
                            RString::from(json)
                        }
                        Err(e) => {
                            LOGGER_CLEANER.debug(format!(
                                "list_candidates response serialization failed: {}",
                                e
                            ));
                            LOGGER_CLEANER.error(format!(
                                "Failed to serialize list_candidates response: {}",
                                e
                            ));
                            RString::from(format!("ERR json serialize: {e}"))
                        }
                    }
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!("collect_cache_candidates failed: {}", e));
                    LOGGER_CLEANER.error(format!("list_candidates failed: {}", e));
                    RString::from(format!("ERR list_candidates: {}", e))
                }
            };
        }

        _ => {
            LOGGER_CLEANER.debug("No direct command matched");
        }
    }

    if raw.starts_with("fn:delete_selected") || raw.starts_with("delete_selected") {
        LOGGER_CLEANER.debug("Matched raw delete_selected command");
        println!("[LIBCLEAN] delete_selected raw command: {}", raw);

        let args = raw
            .strip_prefix("fn:delete_selected")
            .or_else(|| raw.strip_prefix("delete_selected"))
            .unwrap_or("")
            .trim();

        LOGGER_CLEANER.debug(format!("delete_selected parsed args: {}", args));

        if args.is_empty() {
            LOGGER_CLEANER.debug("delete_selected failed: empty args");
            return RString::from("ERR delete_selected requires at least one path");
        }

        println!("[LIBCLEAN] delete_selected args: {}", args);

        let items: Vec<String> = if args.starts_with('[') {
            LOGGER_CLEANER.debug("delete_selected args format detected: JSON array");

            match serde_json::from_str(args) {
                Ok(v) => {
                    LOGGER_CLEANER.debug("delete_selected JSON args parsed successfully");
                    v
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!(
                        "delete_selected JSON args parsing failed: {}",
                        e
                    ));
                    return RString::from(format!("ERR delete_selected invalid JSON args: {}", e));
                }
            }
        } else {
            LOGGER_CLEANER.debug("delete_selected args format detected: whitespace separated paths");
            args.split_whitespace().map(|s| s.to_string()).collect()
        };

        LOGGER_CLEANER.debug(format!(
            "delete_selected parsed {} item(s)",
            items.len()
        ));

        if items.is_empty() {
            LOGGER_CLEANER.debug("delete_selected failed: items list is empty");
            return RString::from("ERR delete_selected requires at least one path");
        }

        println!(
            "[LIBCLEAN] delete_selected received {} path(s)",
            items.len()
        );

        for item in &items {
            LOGGER_CLEANER.debug(format!("delete_selected selected path: {}", item));
            println!("[LIBCLEAN] selected path: {}", item);
        }

        LOGGER_CLEANER.debug("Building execution context for delete_selected");

        let (ctx, _) = match build_execution_context() {
            Ok(v) => {
                LOGGER_CLEANER.debug("Execution context built for delete_selected");
                v
            }
            Err(e) => {
                LOGGER_CLEANER.debug(format!(
                    "build_execution_context failed for delete_selected: {}",
                    e
                ));
                return RString::from(format!("ERR context: {}", e));
            }
        };

        LOGGER_CLEANER.debug("Creating CacheCleaner for delete_selected");
        let cleaner = modules::cache::CacheCleaner::new();

        LOGGER_CLEANER.debug("Calling delete_selected_paths");
        let resp = cleaner.delete_selected_paths(&ctx, &items, true);
        LOGGER_CLEANER.debug("delete_selected_paths completed");

        return match serde_json::to_string(&resp) {
            Ok(json) => {
                LOGGER_CLEANER.debug("delete_selected response serialized successfully");
                RString::from(json)
            }
            Err(e) => {
                LOGGER_CLEANER.debug(format!(
                    "delete_selected response serialization failed: {}",
                    e
                ));
                RString::from(format!("ERR json serialize: {e}"))
            }
        };
    }

    LOGGER_CLEANER.debug("Trying to parse message as CleanerPluginRequest JSON");

    let req: CleanerPluginRequest = match serde_json::from_str(raw) {
        Ok(req) => req,
        Err(e) => return RString::from(format!("ERR invalid request json: {e}")),
    };

    match req.function.as_str() {
        "run" => {
            LOGGER_CLEANER.debug("Matched JSON command: run");

            match execute_cleaner_payload() {
                Ok(payload) => {
                    LOGGER_CLEANER.debug("JSON run payload generated successfully");

                    match serde_json::to_string(&payload) {
                        Ok(json) => {
                            LOGGER_CLEANER.debug("JSON run payload serialized successfully");
                            RString::from(json)
                        }
                        Err(e) => {
                            LOGGER_CLEANER.debug(format!(
                                "JSON run payload serialization failed: {}",
                                e
                            ));
                            RString::from(format!("ERR json serialize analysis: {e}"))
                        }
                    }
                }
                Err(err) => {
                    LOGGER_CLEANER.debug(format!("JSON run failed: {}", err));
                    RString::from(format!("ERR cleaner: {}", err))
                }
            }
        }

        "list_candidates" => {
            LOGGER_CLEANER.debug("Matched JSON command: list_candidates");

            let (ctx, _) = match build_execution_context() {
                Ok(v) => {
                    LOGGER_CLEANER.debug("Execution context built for JSON list_candidates");
                    v
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!(
                        "build_execution_context failed for JSON list_candidates: {}",
                        e
                    ));
                    return RString::from(format!("ERR context: {}", e));
                }
            };

            LOGGER_CLEANER.debug("Creating CacheCleaner for JSON list_candidates");
            let cleaner = modules::cache::CacheCleaner::new();

            match cleaner.collect_cache_candidates(&ctx) {
                Ok(items) => {
                    LOGGER_CLEANER.debug(format!(
                        "JSON list_candidates returned {} item(s)",
                        items.len()
                    ));

                    let resp = ListCandidatesResponse { ok: true, items };

                    match serde_json::to_string(&resp) {
                        Ok(json) => {
                            LOGGER_CLEANER.debug(
                                "JSON list_candidates response serialized successfully",
                            );
                            RString::from(json)
                        }
                        Err(e) => {
                            LOGGER_CLEANER.debug(format!(
                                "JSON list_candidates response serialization failed: {}",
                                e
                            ));
                            RString::from(format!("ERR json serialize: {e}"))
                        }
                    }
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!("JSON list_candidates failed: {}", e));
                    RString::from(format!("ERR list_candidates: {}", e))
                }
            }
        }

        "delete_selected" => {
            LOGGER_CLEANER.debug("Matched JSON command: delete_selected");

            let payload = match req.payload {
                Some(value) => {
                    LOGGER_CLEANER.debug("JSON delete_selected payload found");
                    value
                }
                None => {
                    LOGGER_CLEANER.debug("JSON delete_selected failed: missing payload");
                    return RString::from("ERR missing payload");
                }
            };

            let delete_req: DeleteSelectedRequest = match serde_json::from_value::<DeleteSelectedRequest>(payload) {
                Ok(v) => v,
                Err(e) => {
                    return RString::from(format!("ERR invalid delete_selected payload: {e}"))
                }
            };

            LOGGER_CLEANER.debug("Building execution context for JSON delete_selected");

            let (ctx, _) = match build_execution_context() {
                Ok(v) => {
                    LOGGER_CLEANER.debug("Execution context built for JSON delete_selected");
                    v
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!(
                        "build_execution_context failed for JSON delete_selected: {}",
                        e
                    ));
                    return RString::from(format!("ERR context: {}", e));
                }
            };

            LOGGER_CLEANER.debug("Creating CacheCleaner for JSON delete_selected");
            let cleaner = modules::cache::CacheCleaner::new();

            LOGGER_CLEANER.debug("Calling delete_selected_paths from JSON delete_selected");
            let resp = cleaner.delete_selected_paths(&ctx, &delete_req.items, true);
            LOGGER_CLEANER.debug("JSON delete_selected delete_selected_paths completed");

            match serde_json::to_string(&resp) {
                Ok(json) => {
                    LOGGER_CLEANER.debug(
                        "JSON delete_selected response serialized successfully",
                    );
                    RString::from(json)
                }
                Err(e) => {
                    LOGGER_CLEANER.debug(format!(
                        "JSON delete_selected response serialization failed: {}",
                        e
                    ));
                    RString::from(format!("ERR json serialize: {e}"))
                }
            }
        }

        other => {
            LOGGER_CLEANER.debug(format!("Unknown JSON function received: {}", other));
            RString::from(format!("ERR unknown function: {other}"))
        }
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
