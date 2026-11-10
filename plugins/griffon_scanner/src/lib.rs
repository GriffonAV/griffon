pub mod scanner_engine;
pub mod scanner_quarantine;
pub mod scanner_updater;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use plugin_sdk::params::NoParams;
use plugin_sdk::{PluginRouter, export_plugin};
use serde::{Deserialize, Serialize};

use crate::scanner_engine::ScanEngine;
use crate::scanner_engine::scanargs::{PrepArgs, ScanArgs};
use crate::scanner_engine::yara_engine::threat_category::ThreatCategory;
use crate::scanner_quarantine::Quarantine;
use crate::scanner_updater::ScannerUpdater;

// =========================================================
// Engine state
// =========================================================

static ENGINE_STATE: AtomicU8 = AtomicU8::new(0);

const STATE_STOPPED: u8 = 0;
const STATE_LOADING: u8 = 1;
const STATE_READY: u8 = 2;
const STATE_ERROR: u8 = 3;

fn engine_state_label(state: u8) -> &'static str {
    match state {
        STATE_STOPPED => "stopped",
        STATE_LOADING => "loading",
        STATE_READY => "ready",
        STATE_ERROR => "error",
        _ => "unknown",
    }
}

lazy_static::lazy_static! {
    static ref ENGINE: Mutex<Option<ScanEngine>> = Mutex::new(None);
}

/// Returned whenever a function requires the engine and it isn't ready.azer
/// `Some(RString)` short-circuits the caller with that error JSON;
/// `None` means the engine is ready to use.
fn engine_not_ready() -> Option<EngineStatus> {
    match ENGINE_STATE.load(Ordering::SeqCst) {
        STATE_READY => None,
        other => Some(EngineStatus {
            state: engine_state_label(other).to_string(),
            message: match other {
                STATE_LOADING => "Engine is still loading signatures, try again shortly".into(),
                STATE_STOPPED => "Engine is not running, call start first".into(),
                STATE_ERROR => "Engine failed to initialize".into(),
                _ => "Engine is in an unknown state".into(),
            },
        }),
    }
}

// =========================================================
// Shared response / request types
// =========================================================

#[derive(Serialize)]
struct EngineStatus {
    state: String,
    message: String,
}

#[derive(Serialize)]
struct Ack {
    ok: bool,
    message: String,
}

impl Ack {
    fn ok(message: impl Into<String>) -> Self {
        Self {
            ok: true,
            message: message.into(),
        }
    }
}

#[derive(Deserialize)]
pub struct ScanOptions {
    paths: Vec<String>,

    #[serde(default)]
    archive: bool,

    #[serde(default)]
    folder: bool,

    #[serde(default)]
    threading: String,

    #[serde(default)]
    threads: String,

    #[serde(default)]
    threats: Vec<String>,
}
#[derive(Deserialize)]
struct PathTarget {
    paths: Vec<String>,
}

#[derive(Deserialize)]
struct QuarantineTarget {
    /// Name of the quarantined item, as returned by `q_list`.
    file_name: Vec<String>,
}

// =========================================================
// Generic command registry
// =========================================================

#[derive(Serialize)]
struct GuiThreat {
    path: String,
    name: String,
    severity: String,
}

#[derive(Serialize)]
struct GuiScanResponse {
    message: String,
    total_scanned: u64,
    total_skipped: u64,
    total_errors: u64,
    total_threats: u64,
    time_taken: f64,
    threats: Vec<GuiThreat>,
}

fn build_router() -> PluginRouter {
    PluginRouter::new()
        .method("check", |_: NoParams| -> Result<EngineStatus, String> {
            let state = ENGINE_STATE.load(Ordering::SeqCst);
            Ok(EngineStatus {
                state: engine_state_label(state).to_string(),
                message: match state {
                    STATE_READY => "Engine is ready".into(),
                    STATE_LOADING => "Engine is still loading signatures".into(),
                    STATE_ERROR => "Engine failed to initialize".into(),
                    _ => "Engine is not running".into(),
                },
            })
        })
        .method("stop", |_: NoParams| -> Result<Ack, String> {
            Ok(stop_engine())
        })
        .method(
            "scan",
            |opts: ScanOptions| -> Result<GuiScanResponse, String> {
                if let Some(status) = engine_not_ready() {
                    return Err(status.message);
                }

                let mut lock = ENGINE.lock().unwrap();
                let engine = lock.as_mut().ok_or("Engine state is invalid")?;

                let default_download_path = dirs::download_dir()
                    .ok_or("Failed to get default download directory")?
                    .to_string_lossy()
                    .to_string();

                let paths = if opts.paths.is_empty() {
                    Vec::new()
                } else if opts.paths.len() == 1 && opts.paths[0] == "##/download" {
                    vec![default_download_path]
                } else {
                    opts.paths.clone()
                };

                let scanargs = ScanArgs {
                    archives: opts.archive,
                    recursive: opts.folder,
                    paths: paths.iter().map(PathBuf::from).collect(),
                    threads: opts.threading,
                    nb_threads: opts.threads.parse::<u32>().unwrap_or(1),
                    include: opts
                        .threats
                        .iter()
                        .filter_map(|s| ThreatCategory::try_from_str(s))
                        .collect(),
                    exclude: Vec::new(),
                    yara_only: false,
                };

                let report = engine.scan(&scanargs);

                let gui_threats: Vec<GuiThreat> = report
                    .results
                    .iter()
                    .flat_map(|res| {
                        res.threats.iter().map(|t| GuiThreat {
                            path: res.path.to_string_lossy().to_string(), // Use the file path
                            name: t.name.clone(),
                            severity: t.severity.to_string(),
                        })
                    })
                    .collect();

                Ok(GuiScanResponse {
                    message: format!(
                        "Scan completed: {} files scanned, {} threats found",
                        report.total_scanned, report.total_threats
                    ),
                    total_scanned: report.total_scanned,
                    total_skipped: report.total_skipped,
                    total_errors: report.total_errors,
                    total_threats: report.total_threats,
                    threats: gui_threats,
                    time_taken: report.time_taken,
                })
            },
        )
        .method(
            "db_state",
            |_: NoParams| -> Result<serde_json::Value, String> {
                if let Some(status) = engine_not_ready() {
                    return Err(status.message);
                }

                let lock = ENGINE.lock().unwrap();
                let engine = lock.as_ref().ok_or("Engine state is invalid")?;
                let rules_count = engine.yara_rules.as_ref().map_or(0, |r| r.rule_count());

                let updater = ScannerUpdater::default();
                let update_status = match updater.check_for_updates() {
                    Ok(msg) => msg,
                    Err(e) => format!("Failed to check for updates: {e}"),
                };

                Ok(serde_json::json!({
                    "ok": true,
                    "message": format!("Engine is ready, {}.", update_status),
                    "rules_count": rules_count,
                }))
            },
        )
        .method("db_update", |_: NoParams| -> Result<Ack, String> {
            let updater = ScannerUpdater::default();
            updater
                .update()
                .map_err(|e| format!("Update failed: {e}"))?;

            if ENGINE_STATE.load(Ordering::SeqCst) == STATE_READY {
                log::info!(
                    "[LIBSCANNER] Signatures updated. Restarting engine to apply changes..."
                );
                stop_engine();
                std::thread::spawn(start_engine);
            }

            Ok(Ack::ok(
                "Rules updated successfully, Scanner will restart if it was running.",
            ))
        })
        .method("quarantine", |target: PathTarget| -> Result<Ack, String> {
            let q = Quarantine::new(&Quarantine::default_dir())
                .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
            if target.paths.is_empty() {
                return Err("No paths provided for quarantine".into());
            }
            let pathbufs = target.paths.iter().map(PathBuf::from).collect::<Vec<_>>();
            q.quarantine_files(&pathbufs)
                .map(|_| Ack::ok(format!("{} quarantined successfully", target.paths[0])))
                .map_err(|e| e.to_string())
        })
        .method("delete", |target: PathTarget| -> Result<Ack, String> {
            let q = Quarantine::new(&Quarantine::default_dir())
                .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
            if target.paths.is_empty() {
                return Err("No paths provided for quarantine".into());
            }
            let pathbufs = target.paths.iter().map(PathBuf::from).collect::<Vec<_>>();
            q.quarantine_files(&pathbufs)
                .map(|_| Ack::ok(format!("{} quarantined successfully", target.paths[0])))
                .map_err(|e| e.to_string())
        })
        .method(
            "q_list",
            |_: NoParams| -> Result<serde_json::Value, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                let manifests = q.list_sorted();

                Ok(serde_json::json!({
                    "quarantined_items": manifests,
                }))
            },
        )
        .method(
            "q_restore",
            |target: QuarantineTarget| -> Result<Ack, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                if target.file_name.is_empty() {
                    return Err("No names provided for restore".into());
                }
                q.restore_files(&target.file_name)
                    .map(|_path| Ack::ok("Files restored"))
                    .map_err(|e| format!("Failed to restore {}: {e}", target.file_name[0]))
            },
        )
        .method(
            "q_delete",
            |target: QuarantineTarget| -> Result<Ack, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                if target.file_name.is_empty() {
                    return Err("No names provided for delete".into());
                }
                q.delete_quarantined_files(&target.file_name)
                    .map(|_| Ack::ok("Deleted quarantined files"))
                    .map_err(|e| format!("Failed to delete quarantined file: {e}"))
            },
        )
}

fn init_background_engine() {
    std::thread::spawn(|| {
        start_engine();
    });
}

export_plugin! {
    meta: PluginMeta {
        name: "GriffonScan",
        uuid: "a75fcda4-4e8d-4988-b524-8bbd9ce09aeb",
        author: "DiaboloAB",
        description: "YARA & Hash Antivirus Scanner",
    },
    router: build_router,
    // 2. Pass the function name directly
    on_init: init_background_engine,
}
// =========================================================
// Engine lifecycle (unchanged logic, just no longer returns RString)
// =========================================================

fn start_engine() {
    if ENGINE_STATE.load(Ordering::SeqCst) == STATE_LOADING
        || ENGINE_STATE.load(Ordering::SeqCst) == STATE_READY
    {
        log::warn!("[LIBSCANNER] Engine already starting or running");
        return;
    }

    ENGINE_STATE.store(STATE_LOADING, Ordering::SeqCst);
    log::info!("[LIBSCANNER] Loading signatures into memory...");

    let mut engine = ScanEngine::new();
    let args = PrepArgs::default();

    match engine.prepare(&args) {
        Ok((hashes, rules)) => {
            log::info!("[LIBSCANNER] Ready — {} hashes, {} rulesets", hashes, rules);
            *ENGINE.lock().unwrap() = Some(engine);
            ENGINE_STATE.store(STATE_READY, Ordering::SeqCst);
        }
        Err(e) => {
            log::error!("[LIBSCANNER] Failed to prepare engine: {}", e);
            ENGINE_STATE.store(STATE_ERROR, Ordering::SeqCst);
        }
    }
}

fn stop_engine() -> Ack {
    match ENGINE_STATE.load(Ordering::SeqCst) {
        STATE_STOPPED => return Ack::ok("Already stopped"),
        STATE_LOADING => {
            return Ack {
                ok: false,
                message: "Engine is still loading, wait for ready".into(),
            };
        }
        _ => {}
    }

    log::info!("[LIBSCANNER] Stopping engine...");
    *ENGINE.lock().unwrap() = None;
    ENGINE_STATE.store(STATE_STOPPED, Ordering::SeqCst);

    Ack::ok("Engine stopped")
}

#[cfg(test)]
mod router_tests {
    //! Router-level tests. They never start the engine, so ENGINE_STATE stays
    //! `stopped` and tests can run in parallel. Quarantine methods are left out
    //! because they write to the real default quarantine dir; tests/quarantine.rs
    //! covers that logic directly.
    use super::*;
    use serde_json::json;

    fn call(method: &str, params: serde_json::Value) -> Result<serde_json::Value, (i32, String)> {
        build_router()
            .call_json(method, params)
            .map_err(|e| (e.code, e.message))
    }

    #[test]
    fn exposes_every_method() {
        let router = build_router();
        assert_eq!(
            router.method_names(),
            [
                "check",
                "stop",
                "scan",
                "db_state",
                "db_update",
                "quarantine",
                "delete",
                "q_list",
                "q_restore",
                "q_delete",
            ]
        );
    }

    #[test]
    fn check_reports_stopped_engine() {
        let status = call("check", json!(null)).unwrap();
        assert_eq!(status["state"], "stopped");
        assert_eq!(status["message"], "Engine is not running");
    }

    #[test]
    fn check_accepts_raw_wire_forms() {
        let router = build_router();
        for raw in ["fn:check", "fn:check {}", "check null", "  fn:check  "] {
            let out: serde_json::Value =
                serde_json::from_str(router.handle_raw(raw).as_str()).unwrap();
            assert_eq!(out["state"], "stopped", "raw message {raw:?}");
        }
    }

    #[test]
    fn stop_is_idempotent_when_stopped() {
        let ack = call("stop", json!({})).unwrap();
        assert_eq!(ack, json!({ "ok": true, "message": "Already stopped" }));
    }

    #[test]
    fn scan_requires_ready_engine() {
        let (code, msg) = call("scan", json!({ "paths": ["/tmp"] })).unwrap_err();
        assert_eq!(code, 1);
        assert_eq!(msg, "Engine is not running, call start first");
    }

    #[test]
    fn db_state_requires_ready_engine() {
        let (code, _) = call("db_state", json!(null)).unwrap_err();
        assert_eq!(code, 1);
    }

    #[test]
    fn scan_rejects_missing_paths() {
        let (code, msg) = call("scan", json!({ "archive": true })).unwrap_err();
        assert_eq!(code, -32602);
        assert!(msg.contains("missing field `paths`"), "{msg}");
    }

    #[test]
    fn scan_rejects_wrong_types() {
        let (code, _) = call("scan", json!({ "paths": "not-a-list" })).unwrap_err();
        assert_eq!(code, -32602);
    }

    #[test]
    fn quarantine_methods_reject_bad_params_before_touching_disk() {
        // Deserialization fails before the handler (and Quarantine::new) runs.
        for (method, params) in [
            ("quarantine", json!({})),
            ("delete", json!({ "paths": 3 })),
            ("q_restore", json!({ "file_name": "single-string" })),
            ("q_delete", json!(null)),
        ] {
            let (code, _) = call(method, params).unwrap_err();
            assert_eq!(code, -32602, "{method}");
        }
    }

    #[test]
    fn unknown_method_is_reported() {
        let (code, msg) = call("format_disk", json!(null)).unwrap_err();
        assert_eq!(code, -32601);
        assert!(msg.contains("format_disk"), "{msg}");
    }

    #[test]
    fn malformed_payload_json_is_rejected() {
        let out: serde_json::Value =
            serde_json::from_str(build_router().handle_raw("fn:scan {oops").as_str()).unwrap();
        assert_eq!(out["ok"], false);
        assert_eq!(out["code"], -32602);
    }

    #[test]
    fn scan_options_defaults() {
        let opts: ScanOptions = serde_json::from_value(json!({ "paths": ["a"] })).unwrap();
        assert_eq!(opts.paths, ["a"]);
        assert!(!opts.archive && !opts.folder);
        assert!(opts.threading.is_empty() && opts.threads.is_empty() && opts.threats.is_empty());
    }

    #[test]
    fn engine_state_labels() {
        assert_eq!(engine_state_label(STATE_STOPPED), "stopped");
        assert_eq!(engine_state_label(STATE_LOADING), "loading");
        assert_eq!(engine_state_label(STATE_READY), "ready");
        assert_eq!(engine_state_label(STATE_ERROR), "error");
        assert_eq!(engine_state_label(42), "unknown");
    }
}
