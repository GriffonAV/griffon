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
    modules.sort_by_key(|(_, report)| Reverse(report.bytes_freed));

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
    build_execution_context_with_filters(CleanerFilters::default())
}

pub fn build_execution_context_with_filters(
    filters: CleanerFilters,
) -> CleanerResult<(ExecutionContext, String)> {
    let default_config_path = if cfg!(debug_assertions) {
        "bench/configs/light.json"
    } else {
        "/etc/griffon/plugins/griffon_cleaner/light.json"
    };

    let config_path =
        parse_arg("--config").unwrap_or_else(|| default_config_path.to_string());

    let file_cfg = FileCleanerConfig::load_from_file(PathBuf::from(&config_path).as_path())?;

    let dry_run = filters.dry_run.unwrap_or(file_cfg.dry_run);

    let ctx = ExecutionContext {
        config: file_cfg.to_runtime_config(),
        dry_run,
        root_paths: file_cfg.root_paths.iter().map(PathBuf::from).collect(),
        filters,
    };

    ctx.config.validate()?;

    Ok((ctx, config_path))
}

pub fn execute_cleaner_payload() -> CleanerResult<CleanerExportPayload> {
    execute_cleaner_payload_with_filters(CleanerFilters::default())
}

pub fn execute_cleaner_payload_with_filters(
    filters: CleanerFilters,
) -> CleanerResult<CleanerExportPayload> {
    let (ctx, _config_path) = build_execution_context_with_filters(filters)?;
    let output_path =
        parse_arg("--output").unwrap_or_else(|| "griffon_cleaner_report.json".to_string());

    let modules = default_modules();
    let report = run_modules(&ctx, &modules)?;
    let analysis = build_analysis_report(&report);

    print_cache_report(&report);
    print_module_summary(&report);
    print_analysis_report(&analysis);

    if let Err(e) = write_analysis_report_to_file(&analysis, Path::new(&output_path)) {
        eprintln!("Erreur lors de l'export JSON de l'analyse : {:?}", e);
    } else {
        println!("Report exporté dans {}", output_path);
    }

    let selected_scope = CleanerSelectionSummary {
        profile: ctx.config.profile.as_str().to_string(),
        enabled_categories: selected_cache_categories(&ctx.config),
        selected_file_types: ctx.filters.file_types.clone(),
        dry_run: ctx.dry_run,
    };

    println!(
        "Selected cache categories: {:?}",
        selected_scope.enabled_categories
    );

    println!(
        "Selected file types: {:?}",
        selected_scope.selected_file_types
    );

    println!("Selected dry-run mode: {}", selected_scope.dry_run);

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

pub fn execute_cleaner_front_payload() -> CleanerResult<FrontCleanerPayload> {
    execute_cleaner_front_payload_with_filters(CleanerFilters::default())
}

pub fn execute_cleaner_front_payload_with_filters(
    filters: CleanerFilters,
) -> CleanerResult<FrontCleanerPayload> {
    let raw_payload = execute_cleaner_payload_with_filters(filters)?;
    Ok(build_front_cleaner_payload(&raw_payload))
}

fn parse_bool_value(value: &serde_json::Value) -> Result<Option<bool>, String> {
    if value.is_null() {
        return Ok(None);
    }

    if let Some(boolean) = value.as_bool() {
        return Ok(Some(boolean));
    }

    if let Some(text) = value.as_str() {
        let normalized = text.trim().to_lowercase();

        return match normalized.as_str() {
            "" => Ok(None),
            "true" | "1" | "yes" | "on" => Ok(Some(true)),
            "false" | "0" | "no" | "off" => Ok(Some(false)),
            _ => Err(format!("invalid dry_run value: {text}")),
        };
    }

    Err(format!("invalid dry_run value type: {value}"))
}

fn parse_dry_run_from_object(value: &serde_json::Value) -> Result<Option<bool>, String> {
    let dry_run_value = value
        .get("dry_run")
        .or_else(|| value.get("dryRun"))
        .or_else(|| value.get("dry-run"));

    match dry_run_value {
        Some(value) => parse_bool_value(value),
        None => Ok(None),
    }
}

fn parse_string_list_from_value(value: serde_json::Value) -> Result<Vec<String>, String> {
    if value.is_null() {
        return Ok(Vec::new());
    }

    if let Some(items_value) = value.get("items") {
        return parse_string_list_from_value(items_value.clone());
    }

    if let Some(value_value) = value.get("value") {
        return parse_string_list_from_value(value_value.clone());
    }

    if let Some(file_types_value) = value.get("file_types") {
        return parse_string_list_from_value(file_types_value.clone());
    }

    if let Some(array) = value.as_array() {
        let mut items = Vec::new();

        for item in array {
            if let Some(text) = item.as_str() {
                items.push(text.trim().to_string());
                continue;
            }

            if let Some(id) = item.get("id").and_then(|id| id.as_str()) {
                items.push(id.trim().to_string());
                continue;
            }

            return Err(format!("invalid file_types item: {item}"));
        }

        return Ok(items
            .into_iter()
            .filter(|item| !item.trim().is_empty())
            .collect());
    }

    if let Some(text) = value.as_str() {
        let cleaned = text.trim();

        if cleaned.is_empty() {
            return Ok(Vec::new());
        }

        if cleaned.starts_with('[') {
            return serde_json::from_str::<Vec<String>>(cleaned)
                .map_err(|e| format!("invalid stringified file_types array: {e}"));
        }

        return Ok(cleaned
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|item| !item.trim().is_empty())
            .map(|item| item.trim().to_string())
            .collect());
    }

    serde_json::from_value::<Vec<String>>(value)
        .map_err(|e| format!("invalid file_types array: {e}"))
}

fn parse_filters_from_value(value: serde_json::Value) -> Result<CleanerFilters, String> {
    if value.is_null() {
        return Ok(CleanerFilters::default());
    }

    if value.is_array() || value.is_string() {
        let file_types = parse_string_list_from_value(value)?;

        return Ok(CleanerFilters {
            file_types,
            ..Default::default()
        });
    }

    if !value.is_object() {
        return serde_json::from_value(value).map_err(|e| format!("invalid cleaner filters: {e}"));
    }

    let outer_dry_run = parse_dry_run_from_object(&value)?;

    if let Some(filters_value) = value.get("filters") {
        let mut filters = parse_filters_from_value(filters_value.clone())?;

        if outer_dry_run.is_some() {
            filters.dry_run = outer_dry_run;
        }

        return Ok(filters);
    }

    if let Some(value_value) = value.get("value") {
        let mut filters = parse_filters_from_value(value_value.clone())?;

        if outer_dry_run.is_some() {
            filters.dry_run = outer_dry_run;
        }

        return Ok(filters);
    }

    if let Some(file_types_value) = value.get("file_types") {
        let file_types = parse_string_list_from_value(file_types_value.clone())?;

        return Ok(CleanerFilters {
            file_types,
            dry_run: outer_dry_run,
        });
    }

    if let Some(items_value) = value.get("items") {
        let file_types = parse_string_list_from_value(items_value.clone())?;

        return Ok(CleanerFilters {
            file_types,
            dry_run: outer_dry_run,
        });
    }

    let mut filters: CleanerFilters =
        serde_json::from_value(value).map_err(|e| format!("invalid cleaner filters: {e}"))?;

    if outer_dry_run.is_some() {
        filters.dry_run = outer_dry_run;
    }

    Ok(filters)
}

fn parse_filters_from_payload(
    payload: Option<serde_json::Value>,
) -> Result<CleanerFilters, String> {
    match payload {
        Some(value) => parse_filters_from_value(value),
        None => Ok(CleanerFilters::default()),
    }
}

fn parse_filters_from_command_args(args: &str) -> Result<CleanerFilters, String> {
    let args = args.trim();

    if args.is_empty() {
        return Ok(CleanerFilters::default());
    }

    if args.starts_with('{') || args.starts_with('[') {
        let value: serde_json::Value =
            serde_json::from_str(args).map_err(|e| format!("invalid filters json: {e}"))?;

        return parse_filters_from_value(value);
    }

    Ok(CleanerFilters {
        file_types: args
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|item| !item.trim().is_empty())
            .map(|item| item.trim().to_string())
            .collect(),
        ..Default::default()
    })
}

fn serialize_front_payload(payload: FrontCleanerPayload) -> RString {
    match serde_json::to_string(&payload) {
        Ok(json) => RString::from(json),
        Err(e) => RString::from(format!("ERR json serialize front payload: {e}")),
    }
}

fn serialize_raw_payload(payload: CleanerExportPayload) -> RString {
    match serde_json::to_string(&payload) {
        Ok(json) => RString::from(json),
        Err(e) => RString::from(format!("ERR json serialize raw payload: {e}")),
    }
}

fn execute_run_front_with_args(args: &str) -> RString {
    let filters = match parse_filters_from_command_args(args) {
        Ok(filters) => filters,
        Err(e) => return RString::from(format!("ERR invalid run_front filters: {e}")),
    };

    match execute_cleaner_front_payload_with_filters(filters) {
        Ok(payload) => serialize_front_payload(payload),
        Err(err) => RString::from(format!("ERR cleaner: {}", err)),
    }
}

fn execute_run_raw_with_args(args: &str) -> RString {
    let filters = match parse_filters_from_command_args(args) {
        Ok(filters) => filters,
        Err(e) => return RString::from(format!("ERR invalid run_raw filters: {e}")),
    };

    match execute_cleaner_payload_with_filters(filters) {
        Ok(payload) => serialize_raw_payload(payload),
        Err(err) => RString::from(format!("ERR cleaner: {}", err)),
    }
}

fn execute_list_candidates_with_args(args: &str) -> RString {
    let filters = match parse_filters_from_command_args(args) {
        Ok(filters) => filters,
        Err(e) => return RString::from(format!("ERR invalid list_candidates filters: {e}")),
    };

    let (ctx, _) = match build_execution_context_with_filters(filters) {
        Ok(v) => v,
        Err(e) => return RString::from(format!("ERR context: {}", e)),
    };

    let cleaner = modules::cache::CacheCleaner::new();

    match cleaner.collect_cache_candidates(&ctx) {
        Ok(items) => {
            let resp = ListCandidatesResponse { ok: true, items };
            println!("[LIBCLEAN] Found {} cache candidates", resp.items.len());

            match serde_json::to_string(&resp) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR json serialize: {e}")),
            }
        }
        Err(e) => RString::from(format!("ERR list_candidates: {}", e)),
    }
}

fn execute_delete_selected_from_request(delete_req: DeleteSelectedRequest) -> RString {
    let selected_paths = delete_req.selected_paths();

    if selected_paths.is_empty() {
        return RString::from("ERR delete_selected requires at least one path");
    }

    let filters = delete_req.to_filters();

    let (ctx, _) = match build_execution_context_with_filters(filters) {
        Ok(v) => v,
        Err(e) => return RString::from(format!("ERR context: {}", e)),
    };

    let cleaner = modules::cache::CacheCleaner::new();
    let resp = cleaner.delete_selected_paths(&ctx, &selected_paths, true);

    match serde_json::to_string(&resp) {
        Ok(json) => RString::from(json),
        Err(e) => RString::from(format!("ERR json serialize: {e}")),
    }
}

fn execute_delete_selected_with_args(args: &str) -> RString {
    let args = args.trim();

    if args.is_empty() {
        return RString::from("ERR delete_selected requires at least one path");
    }

    println!("[LIBCLEAN] delete_selected args: {}", args);

    if args.starts_with('{') {
        let delete_req: DeleteSelectedRequest = match serde_json::from_str(args) {
            Ok(req) => req,
            Err(e) => {
                return RString::from(format!("ERR delete_selected invalid JSON object: {e}"));
            }
        };

        return execute_delete_selected_from_request(delete_req);
    }

    let items: Vec<String> = if args.starts_with('[') {
        match serde_json::from_str(args) {
            Ok(v) => v,
            Err(e) => {
                return RString::from(format!("ERR delete_selected invalid JSON args: {}", e));
            }
        }
    } else {
        args.split_whitespace().map(|s| s.to_string()).collect()
    };

    if items.is_empty() {
        return RString::from("ERR delete_selected requires at least one path");
    }

    println!(
        "[LIBCLEAN] delete_selected received {} path(s)",
        items.len()
    );

    for item in &items {
        println!("[LIBCLEAN] selected path: {}", item);
    }

    let delete_req = DeleteSelectedRequest {
        items,
        ..Default::default()
    };

    execute_delete_selected_from_request(delete_req)
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

    println!("[LIBCLEAN](msg) Received message: {}", raw);

    match raw {
        "fn:run" | "run" | "fn:run_front" | "run_front" => {
            return match execute_cleaner_front_payload() {
                Ok(payload) => serialize_front_payload(payload),
                Err(err) => RString::from(format!("ERR cleaner: {}", err)),
            };
        }

        "fn:run_raw" | "run_raw" => {
            return match execute_cleaner_payload() {
                Ok(payload) => serialize_raw_payload(payload),
                Err(err) => RString::from(format!("ERR cleaner: {}", err)),
            };
        }

        "fn:list_candidates" | "list_candidates" => {
            return execute_list_candidates_with_args("");
        }

        _ => {}
    }

    if raw.starts_with("fn:delete_selected") || raw.starts_with("delete_selected") {
        println!("[LIBCLEAN] delete_selected raw command: {}", raw);

        let args = raw
            .strip_prefix("fn:delete_selected")
            .or_else(|| raw.strip_prefix("delete_selected"))
            .unwrap_or("")
            .trim();

        return execute_delete_selected_with_args(args);
    }

    if raw.starts_with("fn:run_front") || raw.starts_with("run_front") {
        let args = raw
            .strip_prefix("fn:run_front")
            .or_else(|| raw.strip_prefix("run_front"))
            .unwrap_or("")
            .trim();

        return execute_run_front_with_args(args);
    }

    if raw.starts_with("fn:run_raw") || raw.starts_with("run_raw") {
        let args = raw
            .strip_prefix("fn:run_raw")
            .or_else(|| raw.strip_prefix("run_raw"))
            .unwrap_or("")
            .trim();

        return execute_run_raw_with_args(args);
    }

    if raw.starts_with("fn:run") || raw.starts_with("run") {
        let args = raw
            .strip_prefix("fn:run")
            .or_else(|| raw.strip_prefix("run"))
            .unwrap_or("")
            .trim();

        return execute_run_front_with_args(args);
    }

    if raw.starts_with("fn:list_candidates") || raw.starts_with("list_candidates") {
        let args = raw
            .strip_prefix("fn:list_candidates")
            .or_else(|| raw.strip_prefix("list_candidates"))
            .unwrap_or("")
            .trim();

        return execute_list_candidates_with_args(args);
    }

    let req: CleanerPluginRequest = match serde_json::from_str(raw) {
        Ok(req) => req,
        Err(e) => return RString::from(format!("ERR invalid request json: {e}")),
    };

    match req.function.as_str() {
        "run" | "run_raw" => {
            let filters = match parse_filters_from_payload(req.payload) {
                Ok(filters) => filters,
                Err(e) => return RString::from(format!("ERR invalid cleaner filters: {e}")),
            };

            match execute_cleaner_payload_with_filters(filters) {
                Ok(payload) => serialize_raw_payload(payload),
                Err(err) => RString::from(format!("ERR cleaner: {}", err)),
            }
        }

        "run_front" => {
            let filters = match parse_filters_from_payload(req.payload) {
                Ok(filters) => filters,
                Err(e) => return RString::from(format!("ERR invalid cleaner filters: {e}")),
            };

            match execute_cleaner_front_payload_with_filters(filters) {
                Ok(payload) => serialize_front_payload(payload),
                Err(err) => RString::from(format!("ERR cleaner: {}", err)),
            }
        }

        "list_candidates" => {
            let filters = match parse_filters_from_payload(req.payload) {
                Ok(filters) => filters,
                Err(e) => return RString::from(format!("ERR invalid cleaner filters: {e}")),
            };

            let (ctx, _) = match build_execution_context_with_filters(filters) {
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
                    return RString::from(format!("ERR invalid delete_selected payload: {e}"));
                }
            };

            execute_delete_selected_from_request(delete_req)
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
