use logger::Logger;
use std::io;
use std::sync::mpsc;

mod dispatcher;
mod network;
mod types;

static LOGGER_NETWORK: Logger = if cfg!(debug_assertions) {
    Logger::new("DAEMON-INTERFACE-NETWORK", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "DAEMON-INTERFACE-NETWORK",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

pub const PLUGIN_DIR_PATH: &str = if cfg!(debug_assertions) {
    ".config/griffon"
} else {
    "/usr/lib/griffon/plugins"
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
                        LOGGER_NETWORK.error(format!("client error: {e}"));
                    }
                });
            }
            Err(e) => {
                LOGGER_NETWORK.error(format!("stream error: {e}"));
            }
        }
    }

    Ok(())
}
