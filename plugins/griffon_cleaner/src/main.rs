use chrono::Utc;
use griffon_cleaner::{
    build_analysis_report, default_modules, print_analysis_report, print_cache_report, run_modules,
    whats_enabled_modules, CleanerExportPayload, ExecutionContext, FileCleanerConfig,
};
use std::path::PathBuf;
use uuid::Uuid;

fn parse_arg(flag: &str) -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == flag {
            return args.next();
        }
    }
    None
}

fn main() {
    let config_path =
        parse_arg("--config").unwrap_or_else(|| "bench/configs/light.json".to_string());

    let output_path =
        parse_arg("--output").unwrap_or_else(|| "griffon_cleaner_report.json".to_string());

    let file_cfg = match FileCleanerConfig::load_from_file(PathBuf::from(&config_path).as_path()) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("Erreur chargement config: {:?}", e);
            std::process::exit(1);
        }
    };

    let ctx = ExecutionContext {
        config: file_cfg.to_runtime_config(),
        dry_run: file_cfg.dry_run,
        root_paths: file_cfg.root_paths.iter().map(PathBuf::from).collect(),
    };

    let modules = default_modules();

    match run_modules(&ctx, &modules) {
        Ok(report) => {
            print_cache_report(&report);

            let analysis = build_analysis_report(&report);
            print_analysis_report(&analysis);

            let enabled = whats_enabled_modules(&ctx.config);
            println!("Enabled Cache Modules: {:?}", enabled);

            let payload = CleanerExportPayload {
                generated_at: Utc::now().to_rfc3339(),
                plugin_name: env!("CARGO_PKG_NAME").to_string(),
                plugin_version: env!("CARGO_PKG_VERSION").to_string(),
                run_id: Uuid::new_v4().to_string(),
                report,
                analysis,
            };

            match serde_json::to_string_pretty(&payload) {
                Ok(json_str) => {
                    std::fs::write(&output_path, json_str).expect("Failed to write report to file");
                    println!("Report exporté dans {}", output_path);
                }
                Err(e) => {
                    eprintln!("Erreur lors de la sérialisation du rapport : {:?}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Erreur lors de l'exécution du cleaner : {:?}", e);
            std::process::exit(1);
        }
    }
}
