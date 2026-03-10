use griffon_cleaner::{
    ExecutionContext, CleanerConfig, Profile,
    run_modules, default_modules,
    print_cache_report, whats_enabled_modules,
    build_analysis_report, print_analysis_report,
    CleanerExportPayload,
};
use chrono::Utc;
use uuid::Uuid;

fn main() {
    let config = CleanerConfig {
        profile: Profile::Safe,
        max_log_retention_days: 30,
        max_log_size_gb: 2.0,
        min_bigfile_size_mb: 100,

        enable_system_cache: true,
        enable_user_cache: true,
        enable_browser_cache: false,     // on évite de casser les sessions de navigation des users
        enable_dev_cache: true,
        enable_package_cache: true,
        enable_desktop_cache: true,
    };

    let ctx = ExecutionContext {
        config,
        dry_run: true, // garde true pour tester sinon tu vas vraiment supprimer des fichiers :)
        root_paths: vec!["/".into()],
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

            let json = serde_json::to_string_pretty(&payload);

            match json {
                Ok(json_str) => {
                    std::fs::write("griffon_cleaner_report.json", json_str)
                        .expect("Failed to write report to file");
                    println!("Report exporté dans griffon_cleaner_report.json");
                }
                Err(e) => {
                    eprintln!("Erreur lors de la sérialisation du rapport : {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Erreur lors de l'exécution du cleaner : {:?}", e);
        }
    }
}
