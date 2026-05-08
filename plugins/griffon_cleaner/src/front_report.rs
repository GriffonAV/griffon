use crate::{CleanerExportPayload, ModuleReport};
use serde::Serialize;

const TOP_LIMIT: usize = 10;
const WARNINGS_PREVIEW_LIMIT: usize = 5;
const ERRORS_PREVIEW_LIMIT: usize = 5;

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerPayload {
    pub summary: FrontCleanerSummary,
    pub modules: Vec<FrontCleanerModule>,
    pub selected_scope: FrontCleanerSelectedScope,
    pub metadata: FrontCleanerMetadata,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerSummary {
    pub dry_run: bool,
    pub total_reclaimable_bytes: u64,
    pub total_files_touched: u64,
    pub total_warnings: u64,
    pub total_errors: u64,
    pub total_permission_denied: u64,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerSelectedScope {
    pub profile: String,
    pub enabled_categories: Vec<String>,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerMetadata {
    pub run_id: String,
    pub generated_at: String,
    pub plugin_name: String,
    pub plugin_version: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerModule {
    pub id: String,
    pub label: String,
    pub reclaimable_bytes: u64,
    pub files_touched: u64,
    pub duration_ms: u128,

    pub warnings_count: u64,
    pub errors_count: u64,
    pub permission_denied: u64,

    pub candidate_files_count: u64,
    pub deleted_files_count: u64,
    pub skipped_files_count: u64,

    pub warnings_preview: Vec<String>,
    pub warnings_truncated: bool,

    pub errors_preview: Vec<String>,
    pub errors_truncated: bool,

    pub top_root_paths: Vec<FrontCleanerStatEntry>,
    pub top_file_types: Vec<FrontCleanerStatEntry>,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<FrontCleanerAction>,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerStatEntry {
    pub name: String,
    pub files_touched: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize)]
pub struct FrontCleanerAction {
    pub name: String,
    pub command: String,
    pub enabled: bool,
    pub status: String,
    pub reason: String,
}

pub fn build_front_cleaner_payload(payload: &CleanerExportPayload) -> FrontCleanerPayload {
    let mut modules: Vec<FrontCleanerModule> = payload
        .report
        .per_module
        .values()
        .map(build_front_module)
        .collect();

    modules.sort_by_key(|module| std::cmp::Reverse(module.reclaimable_bytes));

    FrontCleanerPayload {
        summary: FrontCleanerSummary {
            dry_run: payload.report.dry_run,
            total_reclaimable_bytes: payload.report.total_bytes_freed,
            total_files_touched: payload.report.total_files_touched,
            total_warnings: payload.report.total_warnings,
            total_errors: payload.report.total_errors,
            total_permission_denied: payload.report.total_permission_denied,
            duration_ms: payload.report.total_duration_ms,
        },
        modules,
        selected_scope: FrontCleanerSelectedScope {
            profile: payload.selected_scope.profile.clone(),
            enabled_categories: payload.selected_scope.enabled_categories.clone(),
            dry_run: payload.selected_scope.dry_run,
        },
        metadata: FrontCleanerMetadata {
            run_id: payload.run_id.clone(),
            generated_at: payload.generated_at.clone(),
            plugin_name: payload.plugin_name.clone(),
            plugin_version: payload.plugin_version.clone(),
        },
    }
}

fn build_front_module(module: &ModuleReport) -> FrontCleanerModule {
    let warnings_preview = module
        .warnings
        .iter()
        .take(WARNINGS_PREVIEW_LIMIT)
        .cloned()
        .collect();

    let errors_preview = module
        .errors
        .iter()
        .take(ERRORS_PREVIEW_LIMIT)
        .cloned()
        .collect();

    FrontCleanerModule {
        id: module.module_id.clone(),
        label: module_label(&module.module_id).to_string(),
        reclaimable_bytes: module.bytes_freed,
        files_touched: module.files_touched,
        duration_ms: module.duration_ms,

        warnings_count: module.warnings.len() as u64,
        errors_count: module.errors.len() as u64,
        permission_denied: module.permission_denied,

        candidate_files_count: module.candidate_files_count,
        deleted_files_count: module.deleted_files_count,
        skipped_files_count: module.skipped_files_count,

        warnings_preview,
        warnings_truncated: module.warnings.len() > WARNINGS_PREVIEW_LIMIT,

        errors_preview,
        errors_truncated: module.errors.len() > ERRORS_PREVIEW_LIMIT,

        top_root_paths: top_root_paths(module),
        top_file_types: top_file_types(module),

        actions: if module.module_id == "docker" {
            extract_docker_actions(&module.warnings)
        } else {
            Vec::new()
        },
    }
}

fn module_label(module_id: &str) -> &str {
    match module_id {
        "cache" => "Cache Cleaner",
        "docker" => "Docker Cleaner",
        "logs" => "Logs Cleaner",
        "packages" => "Packages Cleaner",
        "bigfiles" => "Big Files Scanner",
        _ => "Cleaner Module",
    }
}

fn top_root_paths(module: &ModuleReport) -> Vec<FrontCleanerStatEntry> {
    let mut entries: Vec<_> = module.per_root_path.iter().collect();

    entries.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.bytes_freed));

    entries
        .into_iter()
        .take(TOP_LIMIT)
        .map(|(path, stats)| FrontCleanerStatEntry {
            name: path.clone(),
            files_touched: stats.files_touched,
            bytes: stats.bytes_freed,
        })
        .collect()
}

fn top_file_types(module: &ModuleReport) -> Vec<FrontCleanerStatEntry> {
    let mut entries: Vec<_> = module.per_file_type.iter().collect();

    entries.sort_by_key(|(_, stats)| std::cmp::Reverse(stats.bytes_freed));

    entries
        .into_iter()
        .filter(|(_, stats)| stats.files_touched >= 3 || stats.bytes_freed >= 1024 * 1024)
        .take(TOP_LIMIT)
        .map(|(file_type, stats)| FrontCleanerStatEntry {
            name: file_type.clone(),
            files_touched: stats.files_touched,
            bytes: stats.bytes_freed,
        })
        .collect()
}

fn extract_docker_actions(warnings: &[String]) -> Vec<FrontCleanerAction> {
    warnings
        .iter()
        .filter_map(|line| parse_docker_action_line(line))
        .collect()
}

fn parse_docker_action_line(line: &str) -> Option<FrontCleanerAction> {
    if !line.starts_with("- ") || !line.contains(": docker ") {
        return None;
    }

    let line = line.strip_prefix("- ")?;
    let (name, rest) = line.split_once(": docker ")?;

    let (command_part, reason) = match rest.split_once(" - ") {
        Some((command_part, reason)) => (command_part.trim(), reason.trim()),
        None => (rest.trim(), ""),
    };

    let enabled = command_part.contains("[enabled]");
    let disabled = command_part.contains("[disabled]");

    let command = command_part
        .split('[')
        .next()
        .unwrap_or(command_part)
        .trim();

    let status = if enabled {
        "enabled"
    } else if disabled {
        "disabled"
    } else {
        "unknown"
    };

    Some(FrontCleanerAction {
        name: name.trim().to_string(),
        command: format!("docker {command}"),
        enabled,
        status: status.to_string(),
        reason: reason.to_string(),
    })
}
