use std::io;
use std::sync::mpsc;

mod dispatcher;
mod network;
mod types;

pub const PLUGIN_DIR_PATH: &str = if cfg!(debug_assertions) {
    ".config/griffon"
} else {
    "/usr/lib/griffonav/plugins"
};

fn main() -> io::Result<()> {
    let (task_tx, task_rx) = mpsc::channel::<types::DaemonTask>();

    let listener = network::setup_listener()?;
    dispatcher::start_dispatcher(task_rx, PLUGIN_DIR_PATH);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let tx = task_tx.clone();
                std::thread::spawn(move || {
                    if let Err(e) = network::handle_client(stream, tx) {
                        eprintln!("[DAEMON](ERROR) client error: {e}");
                    }
                });
            }
            Err(e) => {
                eprintln!("[DAEMON](ERROR) stream error: {e}");
            }
        }
    }

    Ok(())
}
