use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use tauri::State;

pub struct DaemonConnection(
    pub Mutex<Option<UnixStream>>,
    pub Mutex<u32>, // id_request
);

#[tauri::command]
pub fn get_daemon_status(state: State<'_, DaemonConnection>) -> bool {
    let guard = state.0.lock().unwrap();
    guard.is_some()
}
