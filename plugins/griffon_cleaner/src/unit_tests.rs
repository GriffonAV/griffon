use super::*;
use serde_json::json;

pub(crate) fn context(dry_run: bool) -> ExecutionContext {
    ExecutionContext {
        config: CleanerConfig {
            profile: Profile::Safe,
            max_log_retention_days: 7,
            max_log_size_gb: 1.0,
            min_bigfile_size_mb: 10,
            enable_system_cache: false,
            enable_user_cache: true,
            enable_browser_cache: false,
            enable_dev_cache: false,
            enable_package_cache: false,
            enable_desktop_cache: false,
        },
        dry_run,
        root_paths: vec![],
        filters: CleanerFilters::default(),
    }
}
#[test]
fn boolean_filter_aliases_and_nested_precedence() {
    for v in [
        json!(true),
        json!("TRUE"),
        json!(" yes "),
        json!("1"),
        json!("on"),
    ] {
        assert_eq!(parse_bool_value(&v).unwrap(), Some(true));
    }
    for v in [
        json!(false),
        json!("false"),
        json!("no"),
        json!("0"),
        json!("off"),
    ] {
        assert_eq!(parse_bool_value(&v).unwrap(), Some(false));
    }
    for v in [json!(null), json!("")] {
        assert_eq!(parse_bool_value(&v).unwrap(), None);
    }
    for v in [json!(1), json!([]), json!("maybe")] {
        assert!(parse_bool_value(&v).is_err());
    }
    for alias in ["dry_run", "dryRun", "dry-run"] {
        let mut v = json!({"filters":{"file_types":["log"],"dry_run":false}});
        v[alias] = json!(true);
        let f = parse_filters_from_value(v).unwrap();
        assert_eq!(f.dry_run, Some(true));
        assert_eq!(f.file_types, vec!["log"]);
    }
    for v in [
        json!({"value":{"items":["log"]}}),
        json!({"file_types":{"value":["log"]}}),
        json!({"items":{"file_types":["log"]}}),
        json!([{"id":" log "}]),
        json!("log"),
        json!("[\"log\"]"),
    ] {
        assert_eq!(parse_filters_from_value(v).unwrap().file_types, vec!["log"]);
    }
    for v in [json!(null), json!({}), json!(" "), json!([" "])] {
        assert!(parse_filters_from_value(v).unwrap().file_types.is_empty());
    }
    for v in [
        json!(3),
        json!([1]),
        json!("[broken"),
        json!({"file_types":false}),
    ] {
        assert!(parse_filters_from_value(v).is_err());
    }
    assert_eq!(
        parse_filters_from_value(json!(" log,tmp  json "))
            .unwrap()
            .file_types,
        vec!["log", "tmp", "json"]
    );
    assert_eq!(
        parse_filters_from_value(json!(["log"])).unwrap().file_types,
        vec!["log"]
    );
}
#[test]
fn filters_normalize_and_selected_paths_take_precedence() {
    let f = CleanerFilters {
        file_types: vec![" .LOG ".into()],
        dry_run: Some(true),
    };
    assert!(f.has_file_type_filter());
    assert!(f.matches_file_type("log"));
    assert!(!f.matches_file_type("json"));
    assert!(CleanerFilters::default().matches_file_type("anything"));
    let mut r = DeleteSelectedRequest {
        items: vec!["old".into()],
        paths: vec!["new".into()],
        file_types: f.file_types.clone(),
        dry_run: f.dry_run,
    };
    assert_eq!(r.selected_paths(), vec!["new"]);
    assert_eq!(r.to_filters().dry_run, Some(true));
    r.paths.clear();
    assert_eq!(r.selected_paths(), vec!["old"]);
}
#[test]
fn configuration_roundtrip_validation_and_errors() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let value = json!({"profile":"full","dry_run":true,"root_paths":["/fixture"],"max_log_retention_days":9,"max_log_size_gb":2.5,"min_bigfile_size_mb":99,"enable_system_cache":true,"enable_user_cache":true,"enable_browser_cache":true,"enable_dev_cache":true,"enable_package_cache":true,"enable_desktop_cache":true});
    std::fs::write(&path, value.to_string()).unwrap();
    let cfg = FileCleanerConfig::load_from_file(&path).unwrap();
    let runtime = cfg.to_runtime_config();
    assert_eq!(runtime.profile.as_str(), "full");
    assert_eq!(runtime.max_log_retention_days, 9);
    assert_eq!(runtime.max_log_size_gb, 2.5);
    assert_eq!(runtime.min_bigfile_size_mb, 99);
    assert_eq!(
        selected_cache_categories(&runtime),
        vec![
            "system",
            "user",
            "browser",
            "devtools",
            "package_manager",
            "desktop_env"
        ]
    );
    assert!(runtime.validate().is_ok());
    let mut cfg = context(true).config;
    cfg.enable_user_cache = false;
    assert!(cfg.validate().is_err());
    assert!(selected_cache_categories(&cfg).is_empty());
    assert_eq!(cfg.profile.as_str(), "safe");
    std::fs::write(&path, "{}").unwrap();
    assert!(matches!(
        FileCleanerConfig::load_from_file(&path),
        Err(CleanerError::Config(_))
    ));
    assert!(FileCleanerConfig::load_from_file(&path.with_extension("missing")).is_err());
    for cache in KNOWN_CACHE_PATHS {
        assert!(path_allowed_in_profile(cache, &Profile::Full));
    }
    assert_eq!(expand_home("/fixture").unwrap(), PathBuf::from("/fixture"));
}
struct FakeModule {
    id: &'static str,
    fail: bool,
}
impl CleanerModule for FakeModule {
    fn id(&self) -> &'static str {
        self.id
    }
    fn description(&self) -> &'static str {
        "fixture"
    }
    fn run(&self, ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        if self.fail {
            return Err(CleanerError::Internal("fixture failure".into()));
        }
        let mut r = ModuleReport::empty(self.id);
        r.files_touched = 2;
        r.bytes_freed = 100;
        r.candidate_files_count = 2;
        r.deleted_files_count = if ctx.dry_run { 0 } else { 2 };
        r.warnings = vec!["warning".into()];
        r.errors = vec!["error".into()];
        r.permission_denied = 1;
        Ok(r)
    }
}
#[test]
fn runner_aggregates_both_modes_and_propagates_failure() {
    for dry in [true, false] {
        let modules: Vec<Box<dyn CleanerModule>> = vec![
            Box::new(FakeModule {
                id: "a",
                fail: false,
            }),
            Box::new(FakeModule {
                id: "b",
                fail: false,
            }),
        ];
        let r = run_modules(&context(dry), &modules).unwrap();
        assert_eq!(r.total_bytes_freed, 200);
        assert_eq!(r.total_files_touched, 4);
        assert_eq!(r.total_warnings, 2);
        assert_eq!(r.total_errors, 2);
        assert_eq!(r.total_permission_denied, 2);
        assert_eq!(
            r.per_module["a"].deleted_files_count,
            if dry { 0 } else { 2 }
        );
    }
    assert!(run_modules(
        &context(true),
        &[Box::new(FakeModule {
            id: "fail",
            fail: true
        })]
    )
    .is_err());
    assert_eq!(
        run_modules(&context(true), &[]).unwrap().total_bytes_freed,
        0
    );
    let r = FakeModule {
        id: "a",
        fail: false,
    }
    .dry_run(&context(false))
    .unwrap();
    assert_eq!(r.deleted_files_count, 0);
}
fn payload() -> CleanerExportPayload {
    let mut r = run_modules(&context(true), &[]).unwrap();
    for (id, bytes, duration) in [
        ("cache", 4096, 2000),
        ("docker", 1024, 1000),
        ("empty", 0, 0),
        ("logs", 0, 0),
        ("packages", 0, 0),
        ("bigfiles", 0, 0),
    ] {
        let mut m = ModuleReport::empty(id);
        m.bytes_freed = bytes;
        m.duration_ms = duration;
        if bytes > 0 {
            m.files_touched = 4;
            m.candidate_files_count = 4;
            m.deleted_files_count = 2;
            m.warnings = (0..7).map(|i| format!("warning {i}")).collect();
            m.errors = vec![
                "Permission denied".into(),
                "not found".into(),
                "timeout".into(),
                "invalid data".into(),
                "I/O error".into(),
                "other".into(),
            ];
            m.permission_denied = 2;
            for i in 0..12 {
                m.per_root_path.insert(
                    format!("root{i}"),
                    PathStats {
                        files_touched: 4,
                        bytes_freed: i * 100,
                    },
                );
                m.per_file_type.insert(
                    format!("type{i}"),
                    TypeStats {
                        files_touched: 4,
                        bytes_freed: i * 100,
                    },
                );
            }
        }
        if id == "docker" {
            m.warnings.extend(
                [
                    "Docker action plan:",
                    "- Unused volumes: docker volume prune -f [disabled] - persistent data",
                    "- Build cache: docker builder prune -f [enabled] - unused",
                    "- Custom: docker custom",
                    "- docker container prune -f",
                ]
                .map(str::to_string),
            );
        }
        r.per_module.insert(id.into(), m);
    }
    r.total_bytes_freed = 5120;
    r.total_files_touched = 8;
    r.total_duration_ms = 2000;
    CleanerExportPayload {
        analysis: build_analysis_report(&r),
        report: r,
        selected_scope: CleanerSelectionSummary {
            profile: "safe".into(),
            selected_file_types: vec!["log".into()],
            enabled_categories: vec!["user".into()],
            dry_run: true,
        },
        generated_at: "fixture".into(),
        plugin_name: "cleaner".into(),
        plugin_version: "1".into(),
        run_id: "fixture-id".into(),
    }
}
#[test]
fn analysis_rates_rankings_and_exports() {
    let p = payload();
    let a = &p.analysis;
    assert_eq!(a.modules_by_bytes_freed[0].module_id, "cache");
    assert_eq!(a.modules_by_duration[0].module_id, "cache");
    assert_eq!(a.modules_by_efficiency[0].bytes_per_second, 2048.0);
    let c = &a.modules_by_bytes_freed[0];
    assert_eq!(c.delete_success_rate, 50.0);
    assert_eq!(c.avg_bytes_per_file, 1024.0);
    assert_eq!(c.warning_rate, 175.0);
    let z = a
        .modules_by_duration
        .iter()
        .find(|m| m.module_id == "empty")
        .unwrap();
    assert_eq!(z.delete_success_rate, 0.0);
    assert_eq!(z.bytes_per_second, 0.0);
    let v: serde_json::Value = serde_json::from_str(&analysis_report_to_json(a).unwrap()).unwrap();
    assert_eq!(v["total_bytes_freed"], 5120);
    let dir = tempfile::tempdir().unwrap();
    write_analysis_report_to_file(a, &dir.path().join("a.json")).unwrap();
    assert!(write_analysis_report_to_file(a, dir.path()).is_err());
    print_analysis_report(a);
    print_module_summary(&p.report);
    print_cache_report(&p.report);
    assert_eq!(human_readable(1024), "1.00 KB");
}
#[test]
fn frontend_sorts_limits_and_separates_docker_plans() {
    let p = payload();
    let f = build_front_cleaner_payload(&p);
    assert_eq!(f.modules[0].id, "cache");
    assert_eq!(f.metadata.run_id, "fixture-id");
    assert_eq!(f.selected_scope.selected_file_types, vec!["log"]);
    let c = &f.modules[0];
    assert_eq!(c.warnings_preview.len(), 5);
    assert!(c.warnings_truncated && c.errors_truncated);
    assert_eq!(c.top_root_paths.len(), 10);
    assert_eq!(c.top_root_paths[0].bytes, 1100);
    assert_eq!(c.top_file_types.len(), 10);
    let d = f.modules.iter().find(|m| m.id == "docker").unwrap();
    assert_eq!(d.actions.len(), 3);
    assert!(!d.actions[0].enabled);
    assert!(d.actions[1].enabled);
    assert_eq!(d.actions[2].status, "unknown");
    assert!(!d
        .warnings_preview
        .iter()
        .any(|w| w == "Docker action plan:"));
    assert!(serde_json::to_value(&f).unwrap()["metadata"]["run_id"] == "fixture-id");
    assert!(serde_json::to_value(&p).unwrap()["run_id"] == "fixture-id");
}
#[test]
fn metrics_do_not_double_count_permissions_and_handle_zero_duration() {
    let p = payload();
    let m = build_cleaner_metrics_json(&p.report, build_cleaner_metrics_labels(true));
    assert_eq!(m.files_cleaned_total, 4);
    assert_eq!(m.run_duration_seconds, 2.0);
    assert_eq!(m.bytes_freed_per_second, 2560.0);
    assert_eq!(m.errors_by_type["permission_denied"], 4);
    for key in [
        "file_not_found",
        "timed_out",
        "invalid_input",
        "io_error",
        "module_error",
    ] {
        assert_eq!(m.errors_by_type[key], 2);
    }
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/metrics.json");
    write_cleaner_metrics_json_to_file(&m, &path).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(v["files_cleaned_total"], 4);
    assert!(write_cleaner_metrics_json_to_file(&m, dir.path()).is_err());
    assert!(serde_json::to_value(&m).unwrap()["errors_by_type"]["permission_denied"] == 4);
    let empty = run_modules(&context(false), &[]).unwrap();
    let m = build_cleaner_metrics_json(&empty, build_cleaner_metrics_labels(false));
    assert_eq!(m.bytes_freed_per_second, 0.0);
    assert_eq!(m.mode, "aggressive");
}
#[test]
fn dispatch_rejects_bad_requests_before_side_effects() {
    fn error(raw: &str) -> serde_json::Value {
        let response: serde_json::Value =
            serde_json::from_str(handle_message(raw.into()).as_str()).unwrap();
        assert_eq!(response["ok"], false, "{raw}");
        assert!(response["message"].as_str().is_some_and(|m| !m.is_empty()));
        response
    }
    for cmd in [
        "run",
        "run_front",
        "run_raw",
        "run_metrics",
        "list_candidates",
    ] {
        for prefix in ["", "fn:"] {
            assert_eq!(error(&format!("{prefix}{cmd} {{"))["code"], -32602);
            let raw = format!("{prefix}{cmd} {{\"dry_run\":\"invalid\"}}");
            assert_eq!(error(&raw)["code"], 1);
        }
        let msg = json!({"method":cmd,"params":{"dry_run":"invalid"}}).to_string();
        assert_eq!(error(&msg)["code"], 1);
    }
    for msg in [
        "delete_selected",
        "delete_selected [1]",
        r#"{"method":"delete_selected","params":false}"#,
    ] {
        assert_eq!(error(msg)["code"], 1);
    }
    assert_eq!(error("delete_selected {")["code"], -32602);
    for msg in ["fn:delete_selected []", "delete_selected {}"] {
        assert_eq!(error(msg)["code"], 1);
        assert!(error(msg)["message"]
            .as_str()
            .unwrap()
            .contains("at least one path"));
    }
    assert_eq!(error("unknown")["code"], -32601);
    assert_eq!(error(r#"{"method":"unknown"}"#)["code"], -32601);
    assert!(init().is_ok());
    let _ = get_library();
    assert_eq!(default_modules().len(), 2);
}
