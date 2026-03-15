pub mod config;
pub mod context;
pub mod reports;
pub mod runner;
pub mod modules;
pub mod cache_paths;

pub use config::*;
pub use context::*;
pub use reports::*;
pub use runner::*;
pub use cache_paths::*;
pub use modules::CleanerModule;
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString, RVec, Tuple2},
};
use interface::{PluginI, PluginRoot, PluginRoot_Ref};

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
        Box::new(modules::logs::LogsCleaner::new()),
        Box::new(modules::packages::PackagesCleaner::new()),
        Box::new(modules::bigfiles::BigfilesScanner::new()),
    ]
}

fn make_ctx() -> ExecutionContext {
    ExecutionContext {
        config: make_config(),
        dry_run: true,
        root_paths: vec!["/".into()],
    }
}

fn make_modules() -> Vec<Box<dyn crate::CleanerModule>> {
    default_modules()
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

fn print_cache_report(global: &GlobalReport) {
    // On récupère le report du module cache
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
    println!("Total libéré : {}", human_readable(cache_report.bytes_freed));

    // Par dossier racine
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

    // Par type de fichier
    if !cache_report.per_file_type.is_empty() {
        println!("\nPar type :");
        let mut entries: Vec<_> = cache_report.per_file_type.iter().collect();
        entries.sort_by_key(|(typ, _)| *typ);

        for (typ, stats) in entries {
            println!(
                "- {} : {} fichiers, {}",
                typ,
                stats.files_touched,
                human_readable(stats.bytes_freed),
            );
        }
    }

    // Permissions
    if cache_report.permission_denied > 0 {
        println!(
            "\nPermission denied : {} fichiers (lancer en root pour tout nettoyer)",
            cache_report.permission_denied
        );
    }

    // Warnings éventuels
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

    println!("===========================");
    println!("Module Used:");
}

fn whats_enabled_modules(cfg: &CleanerConfig) -> Vec<&'static str> {
    let mut res = Vec::new();

    if cfg.enable_system_cache {
        res.push("System");
    }
    if cfg.enable_user_cache {
        res.push("User");
    }
    if cfg.enable_browser_cache {
        res.push("Browser");
    }
    if cfg.enable_dev_cache {
        res.push("DevTools");
    }
    if cfg.enable_package_cache {
        res.push("PackageManager");
    }
    if cfg.enable_desktop_cache {
        res.push("DesktopEnv");
    }

    res
}

fn make_config() -> CleanerConfig {
    CleanerConfig {
        profile: Profile::Safe,
        max_log_retention_days: 30,
        max_log_size_gb: 2.0,
        min_bigfile_size_mb: 100,

        enable_system_cache: true,
        enable_user_cache: true,
        enable_browser_cache: false,
        enable_dev_cache: true,
        enable_package_cache: true,
        enable_desktop_cache: true,
    }
}

fn run() -> RResult<GlobalReport, RString> {
    let ctx = make_ctx();
    let modules = make_modules();

    match run_modules(&ctx, &modules) {
        Ok(report) => {
            // print_cache_report(&report);
            RResult::ROk(report)
        }
        Err(e) => RResult::RErr(RString::from(format!(
            "Erreur lors de l'exécution du cleaner : {:?}",
            e
        ))),
    }
}
#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(
        RString::from("author"),
        RString::from("Ewen Emeraud"),
    ));
    info.push(Tuple2(RString::from("name"), RString::from("Test Name1")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Plugin Cleaner"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("run"),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    println!("[LIBCLEAN](msg) Received message: {}", msg.as_str());

    match msg.as_str() {
        "fn:run" => match run() {
            RResult::ROk(report) => {
                match serde_json::to_string(&report) {
                    Ok(json) => RString::from(json),
                    Err(e) => RString::from(format!("ERR json serialize: {e}")),
                }
            }
            RResult::RErr(err) => RString::from(format!("ERR cleaner: {}", err)),
        },
        _ => RString::from(format!("ACK LIBCLEAN {}\n", msg.as_str())),
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
