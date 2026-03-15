// src/runner.rs
use crate::{CleanerModule, CleanerResult, ExecutionContext, GlobalReport, ModuleReport};
use std::collections::HashMap;

pub fn run_modules(
    ctx: &ExecutionContext,
    modules: &[Box<dyn CleanerModule>],
) -> CleanerResult<GlobalReport> {
    let mut global = GlobalReport {
        dry_run: ctx.dry_run,
        total_files_touched: 0,
        total_bytes_freed: 0,
        per_module: HashMap::new(),
    };

    for module in modules {
        let report: ModuleReport = if ctx.dry_run {
            module.dry_run(ctx)?
        } else {
            module.run(ctx)?
        };

        global.total_files_touched += report.files_touched;
        global.total_bytes_freed += report.bytes_freed;
        global
            .per_module
            .insert(report.module_id.clone(), report);
    }

    Ok(global)
}
