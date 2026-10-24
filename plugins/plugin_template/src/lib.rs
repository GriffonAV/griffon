use plugin_sdk::{PluginRouter, export_plugin, params::NoParams};
use serde::Serialize;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::{Duration, Instant};

static RUNNING: AtomicBool = AtomicBool::new(false);

lazy_static::lazy_static! {
    static ref HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
}

#[derive(Serialize)]
struct Ack {
    message: String,
}

fn build_router() -> PluginRouter {
    let mut router = PluginRouter::new();

    router.add_method("start", start);
    router.add_method("stop", stop);

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

fn start(_args: NoParams) -> Result<Ack, String> {
    start_thread();
    Ok(Ack {
        message: "ACK LIB1 start".to_string(),
    })
}

fn stop(_args: NoParams) -> Result<Ack, String> {
    RUNNING.store(false, Ordering::SeqCst);
    println!("[LIB1] Stop signal received.");

    if let Some(handle) = HANDLE.lock().unwrap().take() {
        let _ = handle.join();
        println!("[LIB1] Thread stopped.");
    }

    Ok(Ack {
        message: "ACK LIB1 stop".to_string(),
    })
}

fn start_thread() {
    if RUNNING.load(Ordering::SeqCst) {
        println!("[LIB1] Already running");
        return;
    }

    RUNNING.store(true, Ordering::SeqCst);
    let handle = spawn(|| {
        let _ = run_loop();
    });
    *HANDLE.lock().unwrap() = Some(handle);
}

fn run_loop() -> Result<(), String> {
    let interval = Duration::from_secs(5);
    let mut next_time = Instant::now() + interval;

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
