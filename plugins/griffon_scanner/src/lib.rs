pub mod scanner_engine;
pub mod scanner_quarantine;
pub mod scanner_updater;

use std::path::Path;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU8, Ordering};

use abi_stable::std_types::{RVec, Tuple2};
use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString},
};
use plugin_interface::{PluginI, PluginRoot, PluginRoot_Ref};

use crate::scanner_engine::ScanEngine;
use crate::scanner_engine::scanargs::ScanArgs;
use crate::scanner_quarantine::Quarantine;
use crate::scanner_updater::ScannerUpdater;

static ENGINE_STATE: AtomicU8 = AtomicU8::new(0);

const STATE_STOPPED: u8 = 0;
const STATE_LOADING: u8 = 1;
const STATE_READY: u8 = 2;
const STATE_ERROR: u8 = 3;

macro_rules! require_ready {
    () => {
        match ENGINE_STATE.load(Ordering::SeqCst) {
            STATE_LOADING => {
                return RString::from(
                    "NOT_READY: Engine is still loading signatures, try again shortly",
                )
            }
            STATE_STOPPED => return RString::from("ERR: Engine is not running, call start first"),
            STATE_ERROR => return RString::from("ERR: Engine failed to initialize"),
            _ => {} // STATE_READY, continue
        }
    };
}

lazy_static::lazy_static! {
    static ref ENGINE: Mutex<Option<ScanEngine>> = Mutex::new(None);
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
        RString::from("123e4567-e89b-12d3-a456-426614174000"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("start/stop/check/scan/update/quarantine/restore/list"),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    let msg_str = msg.as_str();
    log::info!("[LIBSCANNER] Received message: {}", msg_str);

    if msg_str == "fn:check" {
        check_engine()
    } else if msg_str == "fn:stop" {
        stop_engine()
    } else if let Some(path_str) = msg_str.strip_prefix("scan:") {
        handle_scan(path_str)
    } else if msg_str == "update" {
        handle_update()
    } else if let Some(path_str) = msg_str.strip_prefix("fn:quarantine:") {
        handle_quarantine(path_str)
    } else if let Some(path_str) = msg_str.strip_prefix("fn:restore:") {
        handle_restore(path_str)
    } else if msg_str == "list" {
        handle_list()
    } else {
        RString::from(format!("ACK LIBSCANNER {}\n", msg_str))
    }
}

fn check_engine() -> RString {
    match ENGINE_STATE.load(Ordering::SeqCst) {
        STATE_READY => RString::from("ACK: Engine is ready"),
        STATE_LOADING => RString::from("ACK: Engine is still loading signatures"),
        STATE_ERROR => RString::from("ERR: Engine failed to initialize"),
        _ => RString::from("ERR: Engine is not running"),
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
    let args = ScanArgs::default();

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

fn stop_engine() -> RString {
    match ENGINE_STATE.load(Ordering::SeqCst) {
        STATE_STOPPED => return RString::from("ACK: Already stopped"),
        STATE_LOADING => return RString::from("ERR: Engine is still loading, wait for ready"),
        _ => {}
    }

    log::info!("[LIBSCANNER] Stopping engine...");
    *ENGINE.lock().unwrap() = None;
    ENGINE_STATE.store(STATE_STOPPED, Ordering::SeqCst);

    RString::from("ACK: Engine stopped")
}

fn handle_scan(path_str: &str) -> RString {
    require_ready!();

    let path = Path::new(path_str);
    if !path.exists() {
        return RString::from(format!("ERR: Path does not exist: {}", path_str));
    }

    log::info!("[LIBSCANNER] Scanning: {}", path_str);

    let mut lock = ENGINE.lock().unwrap();
    if let Some(engine) = lock.as_mut() {
        let args = ScanArgs::default();
        let report = engine.scan(path, &args);

        match serde_json::to_string(&report) {
            Ok(json) => RString::from(json),
            Err(e) => RString::from(format!("ERR: Failed to serialize report: {}", e)),
        }
    } else {
        RString::from("ERR: Engine state is invalid")
    }
}

fn handle_update() -> RString {
    let updater = ScannerUpdater::default();
    match updater.update() {
        Ok(_) => RString::from("ACK: Update completed successfully"),
        Err(e) => RString::from(format!("ERR: Update failed: {}", e)),
    }
}

fn handle_quarantine(path_str: &str) -> RString {
    let quarantine = Quarantine::new(&Quarantine::default_dir());
    match quarantine {
        Ok(q) => {
            let pathbuf = Path::new(path_str).to_path_buf();
            match q.quarantine_file(&pathbuf) {
                Ok(_) => RString::from(format!("ACK: {} quarantined successfully", path_str)),
                Err(e) => RString::from(format!("ERR: Failed to quarantine {}: {}", path_str, e)),
            }
        }
        Err(e) => RString::from(format!("ERR: Failed to initialize quarantine: {}", e)),
    }
}

fn handle_restore(file_name: &str) -> RString {
    RString::from(format!("ACK: Restore requested for {}", file_name))
}

fn handle_list() -> RString {
    let quarantine = Quarantine::new(&Quarantine::default_dir());
    match quarantine {
        Ok(q) => {
            let manifests = q.list_sorted();
            match serde_json::to_string(&manifests) {
                Ok(json) => RString::from(json),
                Err(e) => RString::from(format!("ERR: Failed to serialize list: {}", e)),
            }
        }
        Err(e) => RString::from(format!("ERR: Failed to initialize quarantine: {}", e)),
    }
}
