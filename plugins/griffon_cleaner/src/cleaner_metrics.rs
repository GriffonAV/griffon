use crate::GlobalReport;
use chrono::Utc;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct CleanerMetricsLabels {
    pub scenario: String,
    pub version: String,
    pub mode: String,
    pub plugin: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CleanerMetricsJson {
    pub scenario: String,
    pub version: String,
    pub mode: String,
    pub plugin: String,
    pub timestamp: String,

    pub bytes_freed_total: u64,
    pub files_scanned_total: u64,
    pub files_cleaned_total: u64,
    pub run_duration_seconds: f64,
    pub bytes_freed_per_second: f64,
    pub errors_by_type: BTreeMap<String, u64>,
}

pub fn build_cleaner_metrics_json(
    report: &GlobalReport,
    labels: CleanerMetricsLabels,
) -> CleanerMetricsJson {
    let run_duration_seconds = report.total_duration_ms as f64 / 1000.0;
    let bytes_freed_per_second = if run_duration_seconds <= 0.0 {
        0.0
    } else {
        report.total_bytes_freed as f64 / run_duration_seconds
    };

    let files_cleaned_total = report
        .per_module
        .values()
        .map(|module| module.deleted_files_count)
        .sum();

    let mut errors_by_type = BTreeMap::new();

    for module in report.per_module.values() {
        let mut permission_denied_from_messages = 0;

        for error in &module.errors {
            let error_type = classify_error_message(error);

            if error_type == "permission_denied" {
                permission_denied_from_messages += 1;
            }

            *errors_by_type.entry(error_type.to_string()).or_insert(0) += 1;
        }

        if module.permission_denied > permission_denied_from_messages {
            *errors_by_type
                .entry("permission_denied".to_string())
                .or_insert(0) += module.permission_denied - permission_denied_from_messages;
        }
    }

    CleanerMetricsJson {
        scenario: labels.scenario,
        version: labels.version,
        mode: labels.mode,
        plugin: labels.plugin,
        timestamp: labels.timestamp,

        bytes_freed_total: report.total_bytes_freed,
        files_scanned_total: report.total_files_touched,
        files_cleaned_total,
        run_duration_seconds,
        bytes_freed_per_second,
        errors_by_type,
    }
}

pub fn build_cleaner_metrics_labels(dry_run: bool) -> CleanerMetricsLabels {
    CleanerMetricsLabels {
        scenario: parse_arg("--scenario").unwrap_or_else(|| "basic_clean".to_string()),
        version: parse_arg("--version").unwrap_or_else(default_version_label),
        mode: parse_arg("--mode").unwrap_or_else(|| default_mode_label(dry_run)),
        plugin: parse_arg("--plugin").unwrap_or_else(|| env!("CARGO_PKG_NAME").to_string()),
        timestamp: parse_arg("--timestamp").unwrap_or_else(|| Utc::now().to_rfc3339()),
    }
}

fn default_version_label() -> String {
    let version = env!("CARGO_PKG_VERSION");

    if version.starts_with('v') {
        version.to_string()
    } else {
        format!("v{}", version)
    }
}

fn default_mode_label(dry_run: bool) -> String {
    if dry_run {
        "safe".to_string()
    } else {
        "aggressive".to_string()
    }
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

pub fn write_cleaner_metrics_json_to_file(
    metrics: &CleanerMetricsJson,
    output_path: &Path,
) -> Result<(), crate::CleanerError> {
    let json = serde_json::to_string_pretty(metrics)
        .map_err(|e| crate::CleanerError::Internal(format!("JSON serialize error: {e}")))?;

    if let Some(parent) = output_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    fs::write(output_path, json)?;
    Ok(())
}

fn classify_error_message(error: &str) -> &'static str {
    let normalized = error.to_lowercase();

    if normalized.contains("permission denied") || normalized.contains("access denied") {
        return "permission_denied";
    }

    if normalized.contains("not found")
        || normalized.contains("no such file")
        || normalized.contains("does not exist")
    {
        return "file_not_found";
    }

    if normalized.contains("timed out") || normalized.contains("timeout") {
        return "timed_out";
    }

    if normalized.contains("invalid input") || normalized.contains("invalid data") {
        return "invalid_input";
    }

    if normalized.contains("io error") || normalized.contains("i/o error") {
        return "io_error";
    }

    "module_error"
}
