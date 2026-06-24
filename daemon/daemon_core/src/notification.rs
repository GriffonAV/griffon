use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ipc_protocol::ipc_payload_interface::format_uuid_bytes;
use notify_rust::{Notification, Timeout, Urgency};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    #[serde(default = "default_config_version")]
    pub version: u32,

    #[serde(default)]
    pub general: GeneralSettings,

    #[serde(default)]
    pub plugins: HashMap<String, PluginSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralSettings {
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginSettings {
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
}

impl Default for NotificationConfig {
    fn default() -> Self {
        Self {
            version: 1,
            general: GeneralSettings::default(),
            plugins: HashMap::new(),
        }
    }
}

impl Default for GeneralSettings {
    fn default() -> Self {
        Self {
            notifications_enabled: true,
        }
    }
}

impl Default for PluginSettings {
    fn default() -> Self {
        Self {
            notifications_enabled: true,
        }
    }
}

impl NotificationConfig {
    pub fn load() -> Self {
        let path = notification_config_path();

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => return Self::default(),
        };

        serde_json::from_str(&content).unwrap_or_else(|_| Self::default())
    }

    pub fn save(&self) -> Result<(), String> {
        let path = notification_config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create notification config directory {}: {e}",
                    parent.display()
                )
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize notification config: {e}"))?;

        fs::write(&path, content).map_err(|e| {
            format!(
                "failed to write notification config file {}: {e}",
                path.display()
            )
        })?;

        Ok(())
    }

    pub fn switch_plugin_notification(plugin_uuid: [u8; 16]) -> Result<bool, String> {
        let mut config = Self::load();
        let uuid = format_uuid_bytes(&plugin_uuid);

        let plugin_settings = config.plugins.entry(uuid).or_default();

        plugin_settings.notifications_enabled = !plugin_settings.notifications_enabled;

        let new_status = plugin_settings.notifications_enabled;

        config.save()?;

        Ok(new_status)
    }

    pub fn is_enabled_for_plugin(&self, plugin_uuid: [u8; 16]) -> bool {
        if !self.general.notifications_enabled {
            return false;
        }

        let uuid = format_uuid_bytes(&plugin_uuid);

        self.plugins
            .get(&uuid)
            .map(|plugin| plugin.notifications_enabled)
            .unwrap_or(true)
    }
}

pub fn send_plugin_response_notification(
    plugin_uuid: [u8; 16],
    plugin_name: &str,
    function_name: &str,
    ok: bool,
) -> Result<(), String> {
    let config = NotificationConfig::load();

    if !config.is_enabled_for_plugin(plugin_uuid) {
        return Ok(());
    }

    let title = if ok {
        format!("{plugin_name} finished")
    } else {
        format!("{plugin_name} failed")
    };

    let body = if ok {
        format!("Function `{function_name}` answered successfully.")
    } else {
        format!("Function `{function_name}` returned an error.")
    };

    send_system_notification(
        &title,
        &body,
        if ok {
            NotificationLevel::Success
        } else {
            NotificationLevel::Error
        },
    )
}

pub fn send_system_notification(
    title: &str,
    body: &str,
    level: NotificationLevel,
) -> Result<(), String> {
    let urgency = match level {
        NotificationLevel::Success => Urgency::Normal,
        NotificationLevel::Error => Urgency::Critical,
    };

    Notification::new()
        .appname("Griffon")
        .summary(title)
        .body(body)
        .icon("emblem-default") // TODO: replace with Griffon icon
        .urgency(urgency)
        .timeout(Timeout::Milliseconds(5000))
        .show()
        .map(|_| ())
        .map_err(|e| format!("failed to send system notification: {e}"))
}

#[derive(Debug, Clone, Copy)]
pub enum NotificationLevel {
    Success,
    Error,
}

fn notification_config_path() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("daemon/config_griffon_daemon.json")
    } else {
        PathBuf::from("/etc/griffon/config_griffon_daemon.json")
    }
}

fn default_true() -> bool {
    true
}

fn default_config_version() -> u32 {
    1
}
