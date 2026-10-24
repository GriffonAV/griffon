use plugin_sdk::{PluginRouter, export_plugin, params::NoParams};
use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};

// =========================================================
// État global (Inchangé)
// =========================================================
static RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
}

// =========================================================
// Types de retour pour le JSON
// =========================================================
#[derive(Serialize)]
struct Ack {
    message: String,
}

// =========================================================
// Configuration du SDK
// =========================================================
fn build_router() -> PluginRouter {
    let mut router = PluginRouter::new();

    // Enregistrement de "start"
    router.add_method("start", |_: NoParams| -> Result<Ack, String> {
        start_thread();
        Ok(Ack {
            message: "ACK LIB1 start".to_string(),
        })
    });

    // Enregistrement de "stop"
    router.add_method("stop", |_: NoParams| -> Result<Ack, String> {
        stop();
        Ok(Ack {
            message: "ACK LIB1 stop".to_string(),
        })
    });

    router
}

export_plugin! {
    meta: PluginMeta {
        name: "Template Plugin",
        uuid: "d0c90f71-b15c-4b7d-afb0-033316c871f9",
        author: "Author Name",
        description: "This is the template plugin",
    },
    router: build_router,
}

// =========================================================
// Logique métier (Nettoyée de RResult / RString)
// =========================================================
fn start_thread() {
    if RUNNING.load(Ordering::SeqCst) {
        println!("[LIB1] Already running");
        return;
    }

    RUNNING.store(true, Ordering::SeqCst);
    let handle = spawn(|| {
        let _ = start();
    });
    *HANDLE.lock().unwrap() = Some(handle);
}

fn start() -> Result<(), String> {
    let interval = Duration::from_secs(5);
    let mut next_time = Instant::now() + interval;
    RUNNING.store(true, Ordering::SeqCst);

    while RUNNING.load(Ordering::SeqCst) {
        println!("[LIB1] Hi from plugin test 1!");
        if let Some(d) = next_time.checked_duration_since(Instant::now()) {
            sleep(d);
        }
        while next_time <= Instant::now() {
            next_time += interval;
        }
    }
    Ok(())
}

fn stop() {
    RUNNING.store(false, Ordering::SeqCst);
    println!("[LIB1] Stop signal received.");

    if let Some(handle) = HANDLE.lock().unwrap().take() {
        let _ = handle.join();
        println!("[LIB1] Thread stopped.");
    }
}
