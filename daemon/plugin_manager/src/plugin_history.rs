use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ipc_protocol::ipc_payload_interface::{PluginInfoDto, format_uuid_bytes};
use logger::Logger;

static LOGGER_PLUGIN_HISTORY: Logger = if cfg!(debug_assertions) {
    Logger::new("PLUGIN_HISTORY", logger::LogLevel::Debug, None)
} else {
    Logger::new(
        "PLUGIN_HISTORY",
        logger::LogLevel::Debug,
        Some("/var/log/griffon/griffon-daemon.log"),
    )
};

pub fn info(plugin_name: &str, plugin_uuid: [u8; 16], message: String) {
    write_history("INFO", plugin_name, plugin_uuid, message);
}

pub fn warn(plugin_name: &str, plugin_uuid: [u8; 16], message: String) {
    write_history("WARN", plugin_name, plugin_uuid, message);
}

pub fn error(plugin_name: &str, plugin_uuid: [u8; 16], message: String) {
    write_history("ERROR", plugin_name, plugin_uuid, message);
}

pub fn plugin_enabled(plugin: &PluginInfoDto) {
    info(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_enabled name=\"{}\" uuid=\"{}\" pid={} path=\"{}\"",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            plugin.path
        ),
    );
}

pub fn plugin_enable_failed(plugin: &PluginInfoDto, reason: &str) {
    error(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_enable_failed name=\"{}\" uuid=\"{}\" pid={} path=\"{}\" reason=\"{}\"",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            plugin.path,
            reason
        ),
    );
}

pub fn plugin_disabled(plugin: &PluginInfoDto) {
    info(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_disabled name=\"{}\" uuid=\"{}\" pid={} path=\"{}\"",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            plugin.path
        ),
    );
}

pub fn plugin_removed(plugin: &PluginInfoDto, enabled: bool) {
    info(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_removed name=\"{}\" uuid=\"{}\" pid={} path=\"{}\" enabled={}",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            plugin.path,
            enabled
        ),
    );
}

pub fn plugin_remove_kill_failed(plugin: &PluginInfoDto, reason: &str) {
    error(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_remove_kill_failed name=\"{}\" uuid=\"{}\" pid={} path=\"{}\" reason=\"{}\"",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            plugin.path,
            reason
        ),
    );
}

pub fn plugin_call_requested(
    plugin: &PluginInfoDto,
    request_id: u32,
    function_name: &str,
    args_count: usize,
) {
    info(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_call_requested name=\"{}\" uuid=\"{}\" pid={} request_id={} function=\"{}\" args_count={}",
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            plugin.pid,
            request_id,
            function_name,
            args_count
        ),
    );
}

pub fn plugin_call_rejected(
    plugin: &PluginInfoDto,
    request_id: u32,
    function_name: &str,
    reason: &str,
) {
    warn(
        &plugin.name,
        plugin.plugin_uuid,
        format!(
            "event=plugin_call_rejected reason=\"{}\" name=\"{}\" uuid=\"{}\" request_id={} function=\"{}\"",
            reason,
            plugin.name,
            format_uuid_bytes(&plugin.plugin_uuid),
            request_id,
            function_name
        ),
    );
}

pub fn plugin_closed(plugin_name: &str, plugin_uuid: [u8; 16], pid: u32, reason: &str) {
    warn(
        plugin_name,
        plugin_uuid,
        format!(
            "event=plugin_closed name=\"{}\" uuid=\"{}\" pid={} reason=\"{}\"",
            plugin_name,
            format_uuid_bytes(&plugin_uuid),
            pid,
            reason
        ),
    );
}

pub fn plugin_result(
    plugin_name: &str,
    plugin_uuid: [u8; 16],
    pid: u32,
    request_id: u32,
    ok: bool,
    output: &str,
) {
    info(
        plugin_name,
        plugin_uuid,
        format!(
            "event=plugin_result name=\"{}\" uuid=\"{}\" pid={} request_id={} ok={} output=\"{}\"",
            plugin_name,
            format_uuid_bytes(&plugin_uuid),
            pid,
            request_id,
            ok,
            output
        ),
    );
}

pub fn plugin_error(
    plugin_name: &str,
    plugin_uuid: [u8; 16],
    pid: u32,
    request_id: u32,
    message: &str,
) {
    error(
        plugin_name,
        plugin_uuid,
        format!(
            "event=plugin_error name=\"{}\" uuid=\"{}\" pid={} request_id={} message=\"{}\"",
            plugin_name,
            format_uuid_bytes(&plugin_uuid),
            pid,
            request_id,
            message
        ),
    );
}

pub fn plugin_unknown_message(plugin_name: &str, plugin_uuid: [u8; 16], pid: u32, message: &str) {
    warn(
        plugin_name,
        plugin_uuid,
        format!(
            "event=plugin_unknown_message name=\"{}\" uuid=\"{}\" pid={} message=\"{}\"",
            plugin_name,
            format_uuid_bytes(&plugin_uuid),
            pid,
            message
        ),
    );
}

fn write_history(level: &str, plugin_name: &str, plugin_uuid: [u8; 16], message: String) {
    let path = plugin_history_path(plugin_uuid);

    if let Some(parent) = path.parent()
        && let Err(e) = fs::create_dir_all(parent)
    {
        LOGGER_PLUGIN_HISTORY.error(format!(
            "Failed to create plugin history directory {}: {e}",
            parent.display()
        ));
        return;
    }

    let mut file = match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => file,
        Err(e) => {
            LOGGER_PLUGIN_HISTORY.error(format!(
                "Failed to open plugin history file {}: {e}",
                path.display()
            ));
            return;
        }
    };

    let line = format!(
        "[{}][{}][{}][{}] {}\n",
        now_unix_timestamp(),
        level,
        plugin_name,
        format_uuid_bytes(&plugin_uuid),
        message
    );

    if let Err(e) = file.write_all(line.as_bytes()) {
        LOGGER_PLUGIN_HISTORY.error(format!(
            "Failed to write plugin history file {}: {e}",
            path.display()
        ));
    }
}

fn plugin_history_path(plugin_uuid: [u8; 16]) -> PathBuf {
    let file_name = if plugin_uuid == [0; 16] {
        "history_unknown".to_string()
    } else {
        format!("history_{}", format_uuid_bytes(&plugin_uuid))
    };

    plugin_history_dir().join(file_name)
}

fn plugin_history_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("/tmp/griffon/plugin-history")
    } else {
        PathBuf::from("/var/log/griffon/plugin-history")
    }
}

fn now_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
