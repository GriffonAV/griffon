use abi_stable::{
    export_root_module,
    prefix_type::PrefixTypeTrait,
    sabi_extern_fn,
    std_types::{RResult, RString, RVec, Tuple2},
};
use interface::{PluginI, PluginRoot, PluginRoot_Ref};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;
use std::time::Instant;

static RUNNING: AtomicBool = AtomicBool::new(false);
use std::sync::Mutex;

lazy_static::lazy_static! {
    static ref HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
}

#[sabi_extern_fn]
pub extern "C" fn init() -> RResult<RVec<Tuple2<RString, RString>>, RString> {
    let mut info = RVec::new();

    info.push(Tuple2(
        RString::from("author"),
        RString::from("Test Author1"),
    ));
    info.push(Tuple2(RString::from("name"), RString::from("Test Name1")));
    info.push(Tuple2(
        RString::from("description"),
        RString::from("Test Description1"),
    ));
    info.push(Tuple2(
        RString::from("function"),
        RString::from("start/stop"),
    ));

    RResult::ROk(info)
}

#[sabi_extern_fn]
extern "C" fn handle_message(msg: RString) -> RString {
    print!("[LIB1](msg) Received message: {}", msg.as_str());

    let res = match msg.as_str() {
        "fn:start" => {
            start_thread();
            RString::from(format!("ACK LIB1 {}\n", msg.as_str()))
        }
        "fn:stop" => {
            stop();
            RString::from(format!("ACK LIB1 {}\n", msg.as_str()))
        }
        _ => RString::from(format!("ACK LIB1 {}\n", msg.as_str())),
    };
    res
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

fn start() -> RResult<(), RString> {
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

    RResult::ROk(())
}

fn stop() -> RResult<(), RString> {
    RUNNING.store(false, Ordering::SeqCst);
    println!("[LIB1] Stop signal received.");

    if let Some(handle) = HANDLE.lock().unwrap().take() {
        let _ = handle.join();
        println!("[LIB1] Thread stopped.");
    }

    RResult::ROk(())
}
