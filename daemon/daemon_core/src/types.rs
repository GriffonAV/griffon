use std::sync::mpsc;

use ipc_protocol::ipc_payload_interface::InterfaceResponse;

#[derive(Debug)]
pub enum DaemonTask {
    CallPlugin {
        request_id: u32,
        plugin_uuid: [u8; 16],
        fn_name: String,
        args: Vec<String>,
        reply_tx: mpsc::Sender<InterfaceResponse>,
    },
    RefreshPlugins {
        request_id: u32,
        reply_tx: mpsc::Sender<InterfaceResponse>,
    },
    SwitchStatusPlugin {
        request_id: u32,
        plugin_uuid: [u8; 16],
        reply_tx: mpsc::Sender<InterfaceResponse>,
    },
}
