use super::*;
use crate::unit_tests::context;
use std::os::unix::process::ExitStatusExt;
fn output(code: i32, out: &str, err: &str) -> std::io::Result<Output> {
    Ok(Output {
        status: std::process::ExitStatus::from_raw(code << 8),
        stdout: out.as_bytes().to_vec(),
        stderr: err.as_bytes().to_vec(),
    })
}
#[test]
fn human_sizes_and_reclaimable_parsing() {
    for (s, n) in [
        ("1 B", 1),
        ("1 KiB", 1024),
        ("1.5MB", 1572864),
        ("2GB", 2147483648),
        ("1TiB", 1099511627776),
        ("bad", 0),
        ("1unknown", 0),
    ] {
        assert_eq!(DockerCleaner::parse_human_size_to_bytes(s), n);
    }
    assert_eq!(
        DockerCleaner::extract_reclaimable_bytes(r#"{"Reclaimable":"1,5MB (50%)"}"#),
        1572864
    );
    assert_eq!(DockerCleaner::extract_reclaimable_bytes("{}"), 0);
    assert_eq!(
        DockerCleaner::extract_reclaimable_bytes("\"Reclaimable\":\"unfinished"),
        0
    );
}
#[test]
fn dry_run_never_prunes_and_safe_excludes_volumes() {
    for (profile, count) in [(Profile::Safe, 3), (Profile::Full, 4)] {
        let mut ctx = context(true);
        ctx.config.profile = profile;
        let mut calls = Vec::new();
        let r = DockerCleaner::new()
            .run_with(&ctx, &mut |a| {
                calls.push(a.join(" "));
                assert!(!a.contains(&"prune"));
                if a[0] == "system" {
                    output(0, "{\"Reclaimable\":\"2KB\"}\n{}", "")
                } else {
                    output(0, "", "")
                }
            })
            .unwrap();
        assert_eq!(calls.len(), 3);
        assert_eq!(r.bytes_freed, 2048);
        assert_eq!(r.skipped_files_count, count);
        assert_eq!(r.deleted_files_count, 0);
        assert_eq!(r.warning_count, r.warnings.len() as u64);
    }
}
#[test]
fn execution_reports_success_exit_failure_and_spawn_failure() {
    let mut ctx = context(false);
    ctx.config.profile = Profile::Full;
    let mut pruned = Vec::new();
    let r = DockerCleaner::new()
        .run_with(&ctx, &mut |a| {
            if a.contains(&"prune") {
                pruned.push(a[0].to_string());
                match a[0] {
                    "image" => return output(1, "out", "denied"),
                    "builder" => return Err(std::io::Error::other("missing")),
                    _ => return output(0, "deleted", "notice"),
                }
            }
            output(0, "", "")
        })
        .unwrap();
    assert_eq!(pruned, vec!["container", "image", "builder", "volume"]);
    assert_eq!(r.deleted_files_count, 2);
    assert_eq!(r.errors.len(), 2);
    assert!(r.warnings.iter().any(|w| w.contains("notice")));
}
#[test]
fn missing_cli_daemon_and_estimation_failures() {
    let c = DockerCleaner::new();
    let r = c
        .run_with(&context(true), &mut |_| {
            Err(std::io::Error::other("missing"))
        })
        .unwrap();
    assert_eq!(r.warning_count, 1);
    assert_eq!(r.bytes_freed, 0);
    let r = c
        .run_with(&context(true), &mut |a| {
            output(if a[0] == "info" { 1 } else { 0 }, "", "")
        })
        .unwrap();
    assert_eq!(r.permission_denied, 1);
    assert_eq!(
        DockerCleaner::estimate_reclaimable_bytes(&mut |_| output(1, "", "")),
        0
    );
    assert!(!c.description().is_empty());
}
