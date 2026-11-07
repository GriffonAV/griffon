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
