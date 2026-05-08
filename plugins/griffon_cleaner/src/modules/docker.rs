use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport, Profile};
use std::process::Command;

#[derive(Default)]
pub struct DockerCleaner;

struct DockerCleanupAction {
    name: &'static str,
    args: Vec<&'static str>,
    enabled: bool,
    reason: &'static str,
}

impl DockerCleaner {
    pub fn new() -> Self {
        Self
    }

    fn docker_available() -> bool {
        Command::new("docker")
            .arg("--version")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn docker_daemon_available() -> bool {
        Command::new("docker")
            .args(["info", "--format", "{{json .ServerVersion}}"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn run_docker_command(args: &[&str]) -> std::io::Result<(bool, String, String)> {
        let output = Command::new("docker").args(args).output()?;

        Ok((
            output.status.success(),
            String::from_utf8_lossy(&output.stdout).to_string(),
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }

    fn estimate_reclaimable_bytes() -> u64 {
        let output = Command::new("docker")
            .args(["system", "df", "--format", "{{json .}}"])
            .output();

        let output = match output {
            Ok(output) if output.status.success() => output,
            _ => return 0,
        };

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut total = 0;

        for line in stdout.lines() {
            if line.contains("\"Reclaimable\"") {
                total += Self::extract_reclaimable_bytes(line);
            }
        }

        total
    }

    fn extract_reclaimable_bytes(line: &str) -> u64 {
        let marker = "\"Reclaimable\":\"";
        let start = match line.find(marker) {
            Some(index) => index + marker.len(),
            None => return 0,
        };

        let rest = &line[start..];
        let end = match rest.find('"') {
            Some(index) => index,
            None => return 0,
        };

        let value = &rest[..end];

        let cleaned = value
            .split('(')
            .next()
            .unwrap_or(value)
            .trim()
            .replace(',', ".");

        Self::parse_human_size_to_bytes(&cleaned)
    }

    fn parse_human_size_to_bytes(value: &str) -> u64 {
        let mut number = String::new();
        let mut unit = String::new();

        for c in value.chars() {
            if c.is_ascii_digit() || c == '.' {
                number.push(c);
            } else if !c.is_whitespace() {
                unit.push(c);
            }
        }

        let number = match number.parse::<f64>() {
            Ok(number) => number,
            Err(_) => return 0,
        };

        let multiplier = match unit.to_lowercase().as_str() {
            "b" => 1.0,
            "kb" | "kib" => 1024.0,
            "mb" | "mib" => 1024.0 * 1024.0,
            "gb" | "gib" => 1024.0 * 1024.0 * 1024.0,
            "tb" | "tib" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
            _ => 0.0,
        };

        (number * multiplier) as u64
    }

    fn cleanup_actions(profile: &Profile) -> Vec<DockerCleanupAction> {
        vec![
            DockerCleanupAction {
                name: "Stopped containers",
                args: vec!["container", "prune", "-f"],
                enabled: true,
                reason: "Remove stopped containers.",
            },
            DockerCleanupAction {
                name: "Dangling images",
                args: vec!["image", "prune", "-f"],
                enabled: true,
                reason: "Remove dangling Docker images.",
            },
            DockerCleanupAction {
                name: "Build cache",
                args: vec!["builder", "prune", "-f"],
                enabled: true,
                reason: "Remove unused Docker build cache.",
            },
            DockerCleanupAction {
                name: "Unused volumes",
                args: vec!["volume", "prune", "-f"],
                enabled: matches!(profile, Profile::Full),
                reason: "Only enabled in full profile because volumes may contain persistent data.",
            },
        ]
    }

    fn format_action_plan(actions: &[DockerCleanupAction]) -> Vec<String> {
        let mut lines = Vec::new();

        lines.push("Docker action plan:".to_string());

        for action in actions {
            let status = if action.enabled {
                "enabled"
            } else {
                "disabled"
            };

            lines.push(format!(
                "- {}: docker {} [{}] - {}",
                action.name,
                action.args.join(" "),
                status,
                action.reason
            ));
        }

        lines
    }
}

impl CleanerModule for DockerCleaner {
    fn id(&self) -> &'static str {
        "docker"
    }

    fn description(&self) -> &'static str {
        "Clean unused Docker containers, images, build cache and optionally volumes."
    }

    fn run(&self, ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        let mut report = ModuleReport::empty(self.id());

        if !Self::docker_available() {
            report
                .warnings
                .push("Docker CLI is not installed.".to_string());
            report.warning_count += 1;
            return Ok(report);
        }

        if !Self::docker_daemon_available() {
            report
                .warnings
                .push("Docker daemon is not available or permission is denied.".to_string());
            report.warning_count += 1;
            report.permission_denied += 1;
            return Ok(report);
        }

        let actions = Self::cleanup_actions(&ctx.config.profile);

        for line in Self::format_action_plan(&actions) {
            report.warnings.push(line);
        }

        let estimated_reclaimable = Self::estimate_reclaimable_bytes();

        if ctx.dry_run {
            report.bytes_freed = estimated_reclaimable;
            report.files_touched = 0;
            report.skipped_files_count =
                actions.iter().filter(|action| action.enabled).count() as u64;

            report.warnings.push(
                "Dry-run mode: Docker cleanup was not executed. Size is estimated from docker system df."
                    .to_string(),
            );

            report
                .warnings
                .push("Docker commands that would be executed:".to_string());

            for action in actions.iter().filter(|action| action.enabled) {
                report
                    .warnings
                    .push(format!("- docker {}", action.args.join(" ")));
            }

            report.warning_count = report.warnings.len() as u64;

            return Ok(report);
        }

        report
            .warnings
            .push("Executed Docker cleanup actions:".to_string());

        for action in actions.iter().filter(|action| action.enabled) {
            match Self::run_docker_command(&action.args) {
                Ok((true, stdout, stderr)) => {
                    report.files_touched += 1;
                    report.deleted_files_count += 1;

                    report.warnings.push(format!(
                        "- {}: docker {} -> success",
                        action.name,
                        action.args.join(" ")
                    ));

                    if !stdout.trim().is_empty() {
                        report.warnings.push(format!(
                            "docker {} output: {}",
                            action.args.join(" "),
                            stdout.trim()
                        ));
                    }

                    if !stderr.trim().is_empty() {
                        report.warnings.push(format!(
                            "docker {} stderr: {}",
                            action.args.join(" "),
                            stderr.trim()
                        ));
                    }
                }
                Ok((false, stdout, stderr)) => {
                    report.errors.push(format!(
                        "- {}: docker {} -> failed. stdout: {} stderr: {}",
                        action.name,
                        action.args.join(" "),
                        stdout.trim(),
                        stderr.trim()
                    ));
                }
                Err(e) => {
                    report.errors.push(format!(
                        "- {}: failed to run docker {}: {e}",
                        action.name,
                        action.args.join(" ")
                    ));
                }
            }
        }

        report.bytes_freed = estimated_reclaimable;
        report.deleted_files_count = report.files_touched;
        report.warning_count = report.warnings.len() as u64;

        Ok(report)
    }
}
