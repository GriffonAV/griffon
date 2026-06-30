pub mod scanner_engine;
pub mod scanner_quarantine;
pub mod scanner_updater;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

use abi_stable::std_types::{RVec, Tuple2};
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString},
};
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};
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

/// Returned whenever a function requires the engine and it isn't ready.
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

/// Unit-like input for functions that take no arguments.
/// Accepts `null`, `{}`, or a missing payload — anything else is a usage error.
#[derive(Deserialize, Default)]
struct NoArgs {}

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
    threads: u32,

    #[serde(default)]
    threats: Vec<String>,
}
#[derive(Deserialize)]
struct PathTarget {
    path: String,
}

#[derive(Deserialize)]
struct QuarantineTarget {
    /// Name of the quarantined item, as returned by `q_list`.
    name: String,
}

// =========================================================
// Generic command registry
// =========================================================

type Handler = Box<dyn Fn(serde_json::Value) -> RString + Send + Sync>;

/// Wraps a typed `(input) -> Result<output, String>` function into a
/// `Handler` that deserializes its JSON input and serializes its JSON
/// output automatically. This is the only place generic (de)serialization
/// logic lives — every entry in the registry is just business logic.
fn command<I, O, F>(f: F) -> Handler
where
    I: serde::de::DeserializeOwned,
    O: Serialize,
    F: Fn(I) -> Result<O, String> + Send + Sync + 'static,
{
    Box::new(move |payload: serde_json::Value| -> RString {
        let input: I = match serde_json::from_value(payload) {
            Ok(v) => v,
            Err(e) => return RString::from(format!("ERR invalid arguments: {e}")),
        };

        match f(input) {
            Ok(output) => match serde_json::to_string(&output) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR json serialize: {e}")),
            },
            Err(e) => RString::from(format!("ERR: {e}")),
        }
    })
}

fn registry() -> &'static HashMap<&'static str, Handler> {
    static REGISTRY: OnceLock<HashMap<&'static str, Handler>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut m: HashMap<&'static str, Handler> = HashMap::new();

        m.insert(
            "check",
            command(|_: NoArgs| -> Result<EngineStatus, String> {
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
            }),
        );

        m.insert(
            "stop",
            command(|_: NoArgs| -> Result<Ack, String> { Ok(stop_engine()) }),
        );

        m.insert(
            "scan",
            command(|opts: ScanOptions| -> Result<serde_json::Value, String> {
                if let Some(status) = engine_not_ready() {
                    return Err(status.message);
                }

                let mut lock = ENGINE.lock().unwrap();
                let engine = lock.as_mut().ok_or("Engine state is invalid")?;

                let path = PathBuf::from(&opts.paths[0]);
                let scanargs = ScanArgs {
                    archives: opts.archive,
                    recursive: opts.folder,
                    path: path.clone(),

                    // enum to string threads to string
                    threads: opts.threading,
                    nb_threads: opts.threads,
                    include: opts
                        .threats
                        .iter()
                        .filter_map(|s| ThreatCategory::try_from_str(s))
                        .collect(),
                    exclude: Vec::new(),
                    yara_only: false,
                };
                let report = engine.scan(path.as_path(), &scanargs);

                serde_json::to_value(&report)
                    .map_err(|e| format!("Failed to serialize report: {e}"))
            }),
        );

        m.insert(
            "db_state",
            command(|_: NoArgs| -> Result<serde_json::Value, String> {
                if let Some(status) = engine_not_ready() {
                    return Err(status.message);
                }

                let lock = ENGINE.lock().unwrap();
                let engine = lock.as_ref().ok_or("Engine state is invalid")?;
                let rules_count = engine.yara_rules.as_ref().map_or(0, |r| r.rule_count());

                Ok(serde_json::json!({
                    "ok": true,
                    "yara_count": rules_count,
                }))
            }),
        );

        m.insert(
            "db_update",
            command(|_: NoArgs| -> Result<Ack, String> {
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

                Ok(Ack::ok("Update completed successfully"))
            }),
        );

        m.insert(
            "quarantine",
            command(|target: PathTarget| -> Result<Ack, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                let pathbuf = PathBuf::from(&target.path);

                q.quarantine_file(&pathbuf)
                    .map(|_| Ack::ok(format!("{} quarantined successfully", target.path)))
                    .map_err(|e| format!("Failed to quarantine {}: {e}", target.path))
            }),
        );

        // "delete" is currently an alias for "quarantine" pending a real
        // permanent-delete implementation.
        m.insert(
            "delete",
            command(|target: PathTarget| -> Result<Ack, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                let pathbuf = PathBuf::from(&target.path);

                q.quarantine_file(&pathbuf)
                    .map(|_| Ack::ok(format!("{} quarantined successfully", target.path)))
                    .map_err(|e| format!("Failed to quarantine {}: {e}", target.path))
            }),
        );

        m.insert(
            "q_list",
            command(|_: NoArgs| -> Result<serde_json::Value, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;
                let manifests = q.list_sorted();

                serde_json::to_value(&manifests)
                    .map_err(|e| format!("Failed to serialize list: {e}"))
            }),
        );

        m.insert(
            "restore",
            command(|target: QuarantineTarget| -> Result<Ack, String> {
                let q = Quarantine::new(&Quarantine::default_dir())
                    .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;

                q.restore_file(&target.name)
                    .map(|path| Ack::ok(format!("Restored to {}", path.display())))
                    .map_err(|e| format!("Failed to restore {}: {e}", target.name))
            }),
        );

        // m.insert(
        //     "q_delete",
        //     command(|target: QuarantineTarget| -> Result<Ack, String> {
        //         let q = Quarantine::new(&Quarantine::default_dir())
        //             .map_err(|e| format!("Failed to initialize quarantine: {e}"))?;

        //         q.delete_file(&target.name)
        //             .map(|_| Ack::ok(format!("{} permanently deleted", target.name)))
        //             .map_err(|e| format!("Failed to delete {}: {e}", target.name))
        //     }),
        // );

        m
    })
}

// =========================================================
// Plugin entry points
// =========================================================

/// Splits the wire format `"fn:<name> <json-payload>"` into the function
/// name and its raw JSON payload string. The payload is optional — a bare
/// `"fn:check"` or `"fn:check "` is treated as having no arguments.
///
/// Also tolerates a leading `"fn:"` being omitted (`"check ..."`), in case
/// some caller sends the bare name.
fn split_message(raw: &str) -> (&str, &str) {
    let raw = raw.trim();
    let (head, rest) = match raw.split_once(char::is_whitespace) {
        Some((h, r)) => (h, r.trim()),
        None => (raw, ""),
    };
    let name = head.strip_prefix("fn:").unwrap_or(head);
    (name, rest)
}

/// Parses the payload portion of a message into a `serde_json::Value`.
/// An empty string becomes `Value::Null`, so `NoArgs` (and any other type
/// with all-optional / no fields) still deserializes correctly.
fn parse_payload(raw_payload: &str) -> Result<serde_json::Value, String> {
    if raw_payload.is_empty() {
        return Ok(serde_json::Value::Null);
    }
    serde_json::from_str(raw_payload).map_err(|e| format!("invalid payload json: {e}"))
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();
    std::thread::spawn(|| {
        start_engine();
    });

    info.push(Tuple2(RString::from("author"), RString::from("DiaboloAB")));
    info.push(Tuple2(RString::from("name"), RString::from("GriffonScan")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("YARA & Hash Antivirus Scanner"),
    ));
    info.push(Tuple2(
        RString::from("UUID"),
        RString::from("a75fcda4-4e8d-4988-b524-8bbd9ce09aeb"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from(
            "check/stop/scan/quarantine/delete/db_state/db_update/q_list/q_delete/restore",
        ),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    let raw = msg.as_str().trim();
    log::info!("[LIBSCANNER] Received message: {}", raw);

    let (function, raw_payload) = split_message(raw);

    let payload = match parse_payload(raw_payload) {
        Ok(v) => v,
        Err(e) => return RString::from(format!("ERR {e}")),
    };

    match registry().get(function) {
        Some(handler) => handler(payload),
        None => RString::from(format!("ERR unknown function: {function}")),
    }
}

#[export_root_module]
pub fn get_library() -> PluginRoot_Ref {
    PluginRoot {
        plugin: PluginI {
            init,
            handle_message,
        }
        .leak_into_prefix(),
    }
    .leak_into_prefix()
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
