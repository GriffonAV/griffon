use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use ipc_protocol::ipc_payload_interface::format_uuid_bytes;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonConfig {
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

impl Default for DaemonConfig {
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

impl DaemonConfig {
    pub fn load() -> Self {
        let path = griffon_config_path();

        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => return Self::default(),
        };

        serde_json::from_str(&content).unwrap_or_else(|_| Self::default())
    }

    pub fn save(&self) -> Result<(), String> {
        let path = griffon_config_path();

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "failed to create griffon config directory {}: {e}",
                    parent.display()
                )
            })?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("failed to serialize griffon config: {e}"))?;

        fs::write(&path, content).map_err(|e| {
            format!(
                "failed to write griffon config file {}: {e}",
                path.display()
            )
        })?;

        Ok(())
    }

    pub fn ensure_plugin_exists(plugin_uuid: [u8; 16]) -> Result<(), String> {
        if plugin_uuid == [0; 16] {
            return Ok(());
        }

        let mut config = Self::load();
        let uuid = format_uuid_bytes(&plugin_uuid);

        if config.plugins.contains_key(&uuid) {
            return Ok(());
        }

        config.plugins.insert(uuid, PluginSettings::default());
        config.save()
    }
}

fn griffon_config_path() -> PathBuf {
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
