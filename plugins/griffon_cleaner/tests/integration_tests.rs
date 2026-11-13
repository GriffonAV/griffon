#![cfg(unix)]

const _: () = assert!(
    cfg!(debug_assertions),
    "integration fixtures require debug config/history paths"
);

use serde_json::{json, Value};
use std::os::unix::fs::PermissionsExt;
use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

// A fresh process isolates the plugin's OnceLock router, cwd and environment.
// No global environment mutation in the parallel parent test runner.
fn scenario(name: &str) {
    let dir = tempfile::tempdir().unwrap();
    let log_path = dir.path().join("child.log");
    let log = fs::File::create(&log_path).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "isolated_session", "--ignored", "--nocapture"])
        .current_dir(dir.path())
        .env("GRIFFON_INTEGRATION_CASE", name)
        .env("GRIFFON_INTEGRATION_ROOT", dir.path())
        .env("GRIFFON_CLEANER_CONFIG", dir.path().join("config.json"))
        .env("HOME", dir.path().join("home"))
        .env("PATH", dir.path().join("bin"))
        .stdin(Stdio::null())
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait().unwrap() {
            break status;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!(
                "scenario {name} timed out:\n{}",
                fs::read_to_string(&log_path).unwrap()
            );
        }
        std::thread::sleep(Duration::from_millis(20));
    };
    assert!(
        status.success(),
        "scenario {name}:\n{}",
        fs::read_to_string(log_path).unwrap()
    );
}

fn fixture_root() -> std::path::PathBuf {
    let root = std::path::PathBuf::from(
        std::env::var_os("GRIFFON_INTEGRATION_ROOT").expect("only run through scenario()"),
    );
    assert_eq!(std::env::current_dir().unwrap(), root);
    assert_eq!(std::env::var_os("HOME").unwrap(), root.join("home"));
    assert_eq!(std::env::var_os("PATH").unwrap(), root.join("bin"));
    fs::create_dir_all(root.join("home")).unwrap();
    fs::create_dir_all(root.join("bin")).unwrap();
    root
}

fn executable(path: &Path, script: &str) {
    fs::write(path, script).unwrap();
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
}

fn call(method: &str, params: Value) -> Value {
    let plugin = griffon_cleaner::get_library().plugin();
    let response = (plugin
        .handle_message()
        .expect("plugin must export handle_message"))(
        json!({"method":method,"params":params}).to_string().into(),
    );
    serde_json::from_str(response.as_str()).expect("plugin response must be valid JSON")
}

fn setup(root: &Path) {
    fs::create_dir_all(root.join("home/.cache/nested")).unwrap();
    fs::write(root.join("home/.cache/a.log"), b"abc").unwrap();
    fs::write(root.join("home/.cache/nested/b.LOG"), b"12345").unwrap();
    fs::write(root.join("home/.cache/keep.json"), b"keep").unwrap();
    fs::write(root.join("outside.log"), b"outside").unwrap();
    fs::write(
        root.join("config.json"),
        json!({
            "profile":"safe", "dry_run":true, "root_paths":[],
            "max_log_retention_days":7, "max_log_size_gb":1.0, "min_bigfile_size_mb":10,
            "enable_system_cache":false, "enable_user_cache":true,
            "enable_browser_cache":false, "enable_dev_cache":false,
            "enable_package_cache":false, "enable_desktop_cache":false
        })
        .to_string(),
    )
    .unwrap();
    // Intercept even Docker availability checks. A real daemon is never contacted.
    executable(
        &root.join("bin/docker"),
        "#!/bin/sh\nprintf '%s\\n' \"$*\" >> docker-calls.txt\nexit 1\n",
    );
}

#[test]
fn dry_run_reports_match_files_without_deletion() {
    scenario("dry_run");
}
#[test]
fn real_run_deletes_only_matching_cache_files() {
    scenario("real_run");
}
#[test]
fn candidate_selection_roundtrip_and_scope_rejection() {
    scenario("selection");
}
#[test]
fn frontend_and_metrics_agree_with_scan() {
    scenario("presentation");
}
#[test]
fn bad_config_and_requests_do_not_modify_files() {
    scenario("errors");
}

#[test]
fn docker_dry_run_plans_match_profile_without_pruning() {
    scenario("docker_dry");
}
#[test]
fn docker_partial_failure_preserves_counts_and_continues() {
    scenario("docker_execution");
}
#[test]
fn docker_daemon_failure_preserves_cache_results() {
    scenario("docker_daemon_error");
}

#[test]
#[ignore = "subprocess entry point; invoked by the integration scenarios"]
fn isolated_session() {
    let root = fixture_root();
    setup(&root);
    let cache = root.join("home/.cache");
    match std::env::var("GRIFFON_INTEGRATION_CASE").unwrap().as_str() {
        "dry_run" | "real_run" => {
            let dry = std::env::var("GRIFFON_INTEGRATION_CASE").unwrap() == "dry_run";
            let result = call("run_raw", json!({"file_types":[".log"],"dry_run":dry}));
            assert!(result.get("report").is_some(), "{result}");
            assert_eq!(result["report"]["dry_run"], dry);
            assert_eq!(result["report"]["total_files_touched"], 2);
            assert_eq!(result["report"]["total_bytes_freed"], 8);
            assert_eq!(result["analysis"]["total_bytes_freed"], 8);
            let report = &result["report"]["per_module"]["cache"];
            assert_eq!(report["candidate_files_count"], 2);
            assert_eq!(report["deleted_files_count"], if dry { 0 } else { 2 });
            assert_eq!(report["skipped_files_count"], if dry { 2 } else { 0 });
            assert_eq!(
                result["selected_scope"]["enabled_categories"],
                json!(["user"])
            );
            assert_eq!(cache.join("a.log").exists(), dry);
            assert_eq!(cache.join("nested/b.LOG").exists(), dry);
            let exported: Value = serde_json::from_str(
                &fs::read_to_string(root.join("griffon_cleaner_report.json")).unwrap(),
            )
            .unwrap();
            assert_eq!(exported, result["analysis"]);
            let again = call("run_raw", json!({"file_types":["log"],"dry_run":true}));
            assert_eq!(
                again["report"]["total_bytes_freed"],
                if dry { 8 } else { 0 }
            );
            assert!(fs::read_to_string(root.join("docker-calls.txt"))
                .unwrap()
                .lines()
                .all(|s| s == "--version"));
        }
        "selection" => {
            let listed = call("list_candidates", json!({"file_types":["log"]}));
            assert_eq!(listed["ok"], true);
            let items = listed["items"].as_array().unwrap();
            assert_eq!(items.len(), 2);
            assert_eq!(items[0]["size"], 5);
            assert_eq!(items[1]["size"], 3);
            let paths: Vec<_> = items
                .iter()
                .map(|v| v["path"].as_str().unwrap().to_string())
                .collect();
            let preview = call(
                "delete_selected",
                json!({"paths":paths,"file_types":["log"],"dry_run":true}),
            );
            assert_eq!(preview["deleted_count"], 2);
            assert_eq!(preview["deleted_bytes"], 8);
            assert!(cache.join("a.log").exists());
            std::os::unix::fs::symlink(root.join("outside.log"), cache.join("link.log")).unwrap();
            let mut selected = paths;
            selected.push(root.join("outside.log").display().to_string());
            selected.push(cache.join("link.log").display().to_string());
            let deleted = call(
                "delete_selected",
                json!({"paths":selected,"file_types":["log"],"dry_run":false}),
            );
            assert_eq!(deleted["ok"], false);
            assert_eq!(deleted["deleted_count"], 2);
            assert_eq!(deleted["deleted_bytes"], 8);
            assert_eq!(deleted["failed"].as_array().unwrap().len(), 2);
            assert!(!cache.join("a.log").exists());
            assert!(!cache.join("nested/b.LOG").exists());
            assert!(fs::symlink_metadata(cache.join("link.log"))
                .unwrap()
                .file_type()
                .is_symlink());
        }
        "presentation" => {
            let front = call("run_front", json!({"file_types":["log"],"dry_run":true}));
            let metrics = call("run_metrics", json!({"file_types":["log"],"dry_run":true}));
            assert_eq!(front["summary"]["total_reclaimable_bytes"], 8);
            assert_eq!(front["summary"]["total_files_touched"], 2);
            assert_eq!(
                metrics["bytes_freed_total"],
                front["summary"]["total_reclaimable_bytes"]
            );
            assert_eq!(metrics["files_scanned_total"], 2);
            assert_eq!(metrics["files_cleaned_total"], 0);
            assert_eq!(front["selected_scope"]["dry_run"], true);
            assert_eq!(front["modules"][0]["id"], "cache");
            assert!(front["metadata"]["run_id"]
                .as_str()
                .is_some_and(|s| !s.is_empty()));
            assert!(cache.join("a.log").exists());
        }
        "errors" => {
            let response = call("run_raw", json!({"dry_run":"invalid"}));
            assert_eq!(response["ok"], false);
            assert_eq!(response["code"], 1);
            assert_eq!(call("unknown", json!({}))["code"], -32601);
            assert_eq!(call("delete_selected", json!({}))["ok"], false);
            fs::write(root.join("config.json"), "broken").unwrap();
            for method in ["list_candidates", "run_raw", "run_front", "run_metrics"] {
                let response = call(method, json!({"dry_run":false}));
                assert_eq!(response["ok"], false, "{method}: {response}");
                assert!(response["message"]
                    .as_str()
                    .unwrap()
                    .contains("Invalid config JSON"));
            }
            assert!(cache.join("a.log").exists());
            assert!(cache.join("nested/b.LOG").exists());
            assert!(!root.join("docker-calls.txt").exists());
        }
        "docker_dry" | "docker_execution" => {
            let dry = std::env::var("GRIFFON_INTEGRATION_CASE").unwrap() == "docker_dry";
            executable(
                &root.join("bin/docker"),
                r##"#!/bin/sh
printf '%s\n' "$*" >> docker-calls.txt
case "$*" in
  '--version'|'info --format {{json .ServerVersion}}') exit 0 ;;
  'system df --format {{json .}}') printf '%s\n' '{"Reclaimable":"2KB (100%)"}'; exit 0 ;;
  'image prune -f') printf '%s\n' 'permission denied by fixture' >&2; exit 1 ;;
  'container prune -f'|'builder prune -f'|'volume prune -f') printf '%s\n' 'fixture deleted'; exit 0 ;;
  *) printf '%s\n' 'unexpected docker command' >&2; exit 99 ;;
esac
"##,
            );
            for profile in ["safe", "full"] {
                let path = root.join("config.json");
                let mut config: Value =
                    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
                config["profile"] = json!(profile);
                fs::write(&path, config.to_string()).unwrap();
                fs::write(root.join("docker-calls.txt"), "").unwrap();
                // No fixture cache file matches: isolate Docker accounting in the global report.
                let response = call(
                    "run_raw",
                    json!({"dry_run":dry,"file_types":["not_present"]}),
                );
                let docker = &response["report"]["per_module"]["docker"];
                let enabled = if profile == "safe" { 3 } else { 4 };
                assert_eq!(docker["bytes_freed"], 2048);
                assert_eq!(
                    docker["deleted_files_count"],
                    if dry { 0 } else { enabled - 1 }
                );
                assert_eq!(docker["skipped_files_count"], if dry { enabled } else { 0 });
                assert_eq!(
                    docker["errors"].as_array().unwrap().len(),
                    if dry { 0 } else { 1 }
                );
                assert_eq!(response["report"]["total_bytes_freed"], 2048);
                assert_eq!(
                    response["analysis"]["total_errors"],
                    if dry { 0 } else { 1 }
                );
                let mut expected = vec![
                    "--version",
                    "info --format {{json .ServerVersion}}",
                    "system df --format {{json .}}",
                ];
                if !dry {
                    expected.extend(["container prune -f", "image prune -f", "builder prune -f"]);
                    if profile == "full" {
                        expected.push("volume prune -f");
                    }
                }
                let commands = fs::read_to_string(root.join("docker-calls.txt")).unwrap();
                assert_eq!(commands.lines().collect::<Vec<_>>(), expected);
                if dry {
                    let front = call(
                        "run_front",
                        json!({"dry_run":true,"file_types":["not_present"]}),
                    );
                    let module = front["modules"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .find(|m| m["id"] == "docker")
                        .unwrap();
                    let actions = module["actions"].as_array().unwrap();
                    assert_eq!(actions.len(), 4);
                    let volumes = actions
                        .iter()
                        .find(|a| a["command"] == "docker volume prune -f")
                        .unwrap();
                    assert_eq!(volumes["enabled"], profile == "full");
                    assert!(!fs::read_to_string(root.join("docker-calls.txt"))
                        .unwrap()
                        .contains("prune"));
                } else {
                    let metrics = call(
                        "run_metrics",
                        json!({"dry_run":false,"file_types":["not_present"]}),
                    );
                    assert_eq!(metrics["files_cleaned_total"], enabled - 1);
                    assert_eq!(metrics["errors_by_type"]["permission_denied"], 1);
                }
            }
            assert_eq!(fs::read(cache.join("a.log")).unwrap(), b"abc");
            assert_eq!(fs::read(cache.join("nested/b.LOG")).unwrap(), b"12345");
        }
        "docker_daemon_error" => {
            executable(&root.join("bin/docker"), "#!/bin/sh\nprintf '%s\\n' \"$*\" >> docker-calls.txt\ncase \"$1\" in --version) exit 0 ;; *) exit 1 ;; esac\n");
            let response = call("run_raw", json!({"dry_run":true,"file_types":["log"]}));
            assert_eq!(
                response["report"]["per_module"]["docker"]["permission_denied"],
                1
            );
            assert_eq!(response["analysis"]["total_permission_denied"], 1);
            assert_eq!(response["report"]["total_bytes_freed"], 8);
            assert_eq!(response["report"]["total_files_touched"], 2);
            let metrics = call("run_metrics", json!({"dry_run":true,"file_types":["log"]}));
            assert_eq!(metrics["errors_by_type"]["permission_denied"], 1);
            let commands = fs::read_to_string(root.join("docker-calls.txt")).unwrap();
            assert_eq!(
                commands.lines().collect::<Vec<_>>(),
                vec![
                    "--version",
                    "info --format {{json .ServerVersion}}",
                    "--version",
                    "info --format {{json .ServerVersion}}"
                ]
            );
            assert!(cache.join("a.log").exists());
            assert!(cache.join("nested/b.LOG").exists());
        }
        other => panic!("unknown scenario: {other}"),
    }
    assert_eq!(fs::read(cache.join("keep.json")).unwrap(), b"keep");
    assert_eq!(fs::read(root.join("outside.log")).unwrap(), b"outside");
}
