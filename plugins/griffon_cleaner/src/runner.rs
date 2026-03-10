use crate::{CleanerModule, CleanerResult, ExecutionContext, GlobalReport, ModuleReport};
use std::collections::HashMap;
use std::time::Instant;

pub fn run_modules(
    ctx: &ExecutionContext,
    modules: &[Box<dyn CleanerModule>],
) -> CleanerResult<GlobalReport> {
    let run_started = Instant::now();

    let mut global = GlobalReport {
        dry_run: ctx.dry_run,
        total_files_touched: 0,
        total_bytes_freed: 0,
        total_duration_ms: 0,
        per_module: HashMap::new(),
        total_warnings: 0,
        total_errors: 0,
        total_permission_denied: 0,
    };

    for module in modules {
        let module_started = Instant::now();

        let mut report: ModuleReport = if ctx.dry_run {
            module.dry_run(ctx)?
        } else {
            module.run(ctx)?
        };

        report.duration_ms = module_started.elapsed().as_millis();

        global.total_files_touched += report.files_touched;
        global.total_bytes_freed += report.bytes_freed;
        global.total_warnings += report.warnings.len() as u64;
        global.total_errors += report.errors.len() as u64;
        global.total_permission_denied += report.permission_denied;

        global.per_module.insert(report.module_id.clone(), report);
    }

    global.total_duration_ms = run_started.elapsed().as_millis();

    Ok(global)
}