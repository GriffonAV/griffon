use griffon_cleaner::{
    ExecutionContext, CleanerConfig, Profile,
    run_modules, default_modules, GlobalReport,
};
use abi_stable::{
    std_types::{RResult, RString},
};



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
            let enabled = whats_enabled_modules(&ctx.config);
            println!("Enabled Cache Modules: {:?}", enabled);
        }
        Err(e) => {
            eprintln!("Erreur lors de l'exécution du cleaner : {:?}", e);
        }
    }
}
