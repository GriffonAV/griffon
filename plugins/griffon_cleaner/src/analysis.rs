use crate::{GlobalReport, ModuleReport};
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct CleanerSelectionSummary {
    pub profile: String,
    pub enabled_categories: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Serialize)]
pub struct CleanerExportPayload {
    pub report: GlobalReport,
    pub analysis: AnalysisReport,
    pub selected_scope: CleanerSelectionSummary,
    pub generated_at: String,
    pub plugin_name: String,
    pub plugin_version: String,
    pub run_id: String,
}

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

#[derive(Debug, Clone, serde::Serialize)]
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
    pub warning_rate: f64,
    pub avg_bytes_per_file: f64,
    pub bytes_per_second: f64,
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
    pub modules_by_efficiency: Vec<ModuleAnalysisSummary>,
    pub modules_by_delete_success_rate: Vec<ModuleAnalysisSummary>,
}

fn module_summary(module: &ModuleReport) -> ModuleAnalysisSummary {
    let delete_success_rate = if module.candidate_files_count == 0 {
        0.0
    } else {
        (module.deleted_files_count as f64 / module.candidate_files_count as f64) * 100.0
    };

    let warning_rate = if module.files_touched == 0 {
        0.0
    } else {
        (module.warnings.len() as f64 / module.files_touched as f64) * 100.0
    };

    let avg_bytes_per_file = if module.files_touched == 0 {
        0.0
    } else {
        module.bytes_freed as f64 / module.files_touched as f64
    };

    let bytes_per_second = if module.duration_ms == 0 {
        0.0
    } else {
        module.bytes_freed as f64 / (module.duration_ms as f64 / 1000.0)
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
        warning_rate,
        avg_bytes_per_file,
        bytes_per_second,
    }
}

pub fn analysis_report_to_json(report: &AnalysisReport) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(report)
}

pub fn write_analysis_report_to_file(
    report: &AnalysisReport,
    output_path: &Path,
) -> Result<(), crate::CleanerError> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| crate::CleanerError::Internal(format!("JSON serialize error: {e}")))?;

    fs::write(output_path, json)?;
    Ok(())
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{size:.2} {}", UNITS[unit_idx])
}

fn human_bytes_f64(bytes: f64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut size = bytes;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{size:.2} {}", UNITS[unit_idx])
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

    let mut modules_by_efficiency = summaries.clone();
    modules_by_efficiency.sort_by(|a, b| {
        b.bytes_per_second
            .partial_cmp(&a.bytes_per_second)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut modules_by_delete_success_rate = summaries.clone();
    modules_by_delete_success_rate.sort_by(|a, b| {
        b.delete_success_rate
            .partial_cmp(&a.delete_success_rate)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

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
        modules_by_efficiency,
        modules_by_delete_success_rate,
    }
}

pub fn print_analysis_report(report: &AnalysisReport) {
    const TOP_LIMIT: usize = 10;

    println!("\n=== Cleaner Analysis Report ===");
    println!("Dry-run: {}", report.dry_run);
    println!("Durée totale: {} ms", report.total_duration_ms);
    println!("Fichiers touchés: {}", report.total_files_touched);
    println!("Octets libérés: {}", human_bytes(report.total_bytes_freed));
    println!("Warnings: {}", report.total_warnings);
    println!("Erreurs: {}", report.total_errors);
    println!("Permission denied: {}", report.total_permission_denied);

    println!("\nTop modules par octets libérés:");
    for module in report
        .modules_by_bytes_freed
        .iter()
        .filter(|m| m.bytes_freed > 0)
        .take(TOP_LIMIT)
    {
        println!(
            "- {} => {}, {} fichiers, {} ms, débit {} /s",
            module.module_id,
            human_bytes(module.bytes_freed),
            module.files_touched,
            module.duration_ms,
            human_bytes_f64(module.bytes_per_second),
        );
    }

    println!("\nTop modules par durée:");
    for module in report
        .modules_by_duration
        .iter()
        .filter(|m| m.duration_ms > 0)
        .take(TOP_LIMIT)
    {
        println!(
            "- {} => {} ms, {}, warnings {}, succès {:.2}%",
            module.module_id,
            module.duration_ms,
            human_bytes(module.bytes_freed),
            module.warnings_count,
            module.delete_success_rate,
        );
    }

    println!("\nDétails par module:");
    for module in report.modules_by_duration.iter().take(TOP_LIMIT) {
        if module.files_touched == 0
            && module.bytes_freed == 0
            && module.warnings_count == 0
            && module.errors_count == 0
        {
            continue;
        }

        println!(
            "- {} | candidats: {} | supprimés: {} | ignorés: {} | taille moyenne: {} | warning rate: {:.2}%",
            module.module_id,
            module.candidate_files_count,
            module.deleted_files_count,
            module.skipped_files_count,
            human_bytes_f64(module.avg_bytes_per_file),
            module.warning_rate,
        );

        println!("\nTop modules par efficacité:");
        for module in report
            .modules_by_efficiency
            .iter()
            .filter(|m| m.bytes_per_second > 0.0)
            .take(TOP_LIMIT)
        {
            println!(
                "- {} => {} /s, succès {:.2}%",
                module.module_id,
                human_bytes_f64(module.bytes_per_second),
                module.delete_success_rate,
            );
        }
    }

    println!("===============================");
}
