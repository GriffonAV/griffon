use serde::Serialize;
use crate::{GlobalReport, ModuleReport};

#[derive(Debug, Clone, Serialize)]
pub enum CleanerEventLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Serialize)]
pub enum CleanerEventKind {
    RunStarted,
    RunFinished,
    ModuleStarted,
    ModuleFinished,
    PathScanned,
    FileDetected,
    FileDeleted,
    DeletionFailed,
    MetadataReadFailed,
    PermissionDenied,
    ModuleWarning,
    ModuleError,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanerEvent {
    pub ts_unix_ms: u128,
    pub level: CleanerEventLevel,
    pub kind: CleanerEventKind,
    pub module_id: Option<String>,
    pub path: Option<String>,
    pub root_label: Option<String>,
    pub file_type: Option<String>,
    pub bytes: Option<u64>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct ModuleMetrics {
    pub duration_ms: u128,
    pub candidate_files_count: u64,
    pub deleted_files_count: u64,
    pub skipped_files_count: u64,
    pub missing_paths_count: u64,
    pub existing_paths_count: u64,
    pub warnings_total: u64,
    pub errors_total: u64,
    pub permission_denied_total: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModuleAnalysisSummary {
    pub module_id: String,
    pub duration_ms: u128,
    pub files_touched: u64,
    pub bytes_freed: u64,
    pub warnings_count: u64,
    pub errors_count: u64,
    pub permission_denied: u64,

    pub candidate_files_count: u64,
    pub deleted_files_count: u64,
    pub skipped_files_count: u64,
    pub missing_paths_count: u64,
    pub existing_paths_count: u64,

    pub delete_success_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnalysisReport {
    pub events: Vec<CleanerEvent>,
    pub per_module_metrics: std::collections::HashMap<String, ModuleMetrics>,

    pub dry_run: bool,
    pub total_duration_ms: u128,
    pub total_files_touched: u64,
    pub total_bytes_freed: u64,
    pub total_warnings: u64,
    pub total_errors: u64,
    pub total_permission_denied: u64,

    pub modules_by_bytes_freed: Vec<ModuleAnalysisSummary>,
    pub modules_by_duration: Vec<ModuleAnalysisSummary>,
    pub modules_by_warnings: Vec<ModuleAnalysisSummary>,
}

fn module_summary(module: &ModuleReport) -> ModuleAnalysisSummary {
    let delete_success_rate = if module.candidate_files_count == 0 {
        0.0
    } else {
        (module.deleted_files_count as f64 / module.candidate_files_count as f64) * 100.0
    };

    ModuleAnalysisSummary {
        module_id: module.module_id.clone(),
        duration_ms: module.duration_ms,
        files_touched: module.files_touched,
        bytes_freed: module.bytes_freed,
        warnings_count: module.warnings.len() as u64,
        errors_count: module.errors.len() as u64,
        permission_denied: module.permission_denied,

        candidate_files_count: module.candidate_files_count,
        deleted_files_count: module.deleted_files_count,
        skipped_files_count: module.skipped_files_count,
        missing_paths_count: module.missing_paths_count,
        existing_paths_count: module.existing_paths_count,

        delete_success_rate,
    }
}

pub fn build_analysis_report(global: &GlobalReport) -> AnalysisReport {
    let summaries: Vec<ModuleAnalysisSummary> =
        global.per_module.values().map(module_summary).collect();

    let total_warnings = summaries.iter().map(|m| m.warnings_count).sum();
    let total_errors = summaries.iter().map(|m| m.errors_count).sum();
    let total_permission_denied = summaries.iter().map(|m| m.permission_denied).sum();

    let mut modules_by_bytes_freed = summaries.clone();
    modules_by_bytes_freed.sort_by(|a, b| b.bytes_freed.cmp(&a.bytes_freed));

    let mut modules_by_duration = summaries.clone();
    modules_by_duration.sort_by(|a, b| b.duration_ms.cmp(&a.duration_ms));

    let mut modules_by_warnings = summaries.clone();
    modules_by_warnings.sort_by(|a, b| b.warnings_count.cmp(&a.warnings_count));

    AnalysisReport {
        events: vec![],
        per_module_metrics: Default::default(),
        dry_run: global.dry_run,
        total_duration_ms: global.total_duration_ms,
        total_files_touched: global.total_files_touched,
        total_bytes_freed: global.total_bytes_freed,
        total_warnings,
        total_errors,
        total_permission_denied,
        modules_by_bytes_freed,
        modules_by_duration,
        modules_by_warnings,
    }
}

pub fn print_analysis_report(report: &AnalysisReport) {
    println!("=== Cleaner Analysis Report ===");
    println!("Dry-run: {}", report.dry_run);
    println!("Total duration: {} ms", report.total_duration_ms);
    println!("Total files touched: {}", report.total_files_touched);
    println!("Total bytes freed: {}", report.total_bytes_freed);
    println!("Total warnings: {}", report.total_warnings);
    println!("Total errors: {}", report.total_errors);
    println!("Total permission denied: {}", report.total_permission_denied);

    println!("\nTop modules by bytes freed:");
    for module in report.modules_by_bytes_freed.iter().take(5) {
        println!(
            "- {}: {} bytes, {} files, {} ms",
            module.module_id,
            module.bytes_freed,
            module.files_touched,
            module.duration_ms
        );
    }

    println!("\nTop modules by duration:");
    for module in report.modules_by_duration.iter().take(5) {
        println!(
            "- {}: {} ms, {} bytes, {} warnings",
            module.module_id,
            module.duration_ms,
            module.bytes_freed,
            module.warnings_count
        );
    }

    println!("===============================");
}