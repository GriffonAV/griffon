use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize)]
struct RuleEntry {
    filename: String,
}

#[derive(Deserialize)]
struct Manifest {
    version: String,
    release_date: String,
    rules: Vec<RuleEntry>,
}

fn default_rules_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from("rules/")
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("griffon_scanner")
            .join("rules")
    }
}

pub struct ScannerUpdater {
    pub rules_dir: PathBuf,
}

impl ScannerUpdater {
    pub fn new(dir: &PathBuf) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        log::info!("Updater directory: {}", dir.display());
        Ok(Self {
            rules_dir: dir.to_path_buf(),
        })
    }

    pub fn default() -> Self {
        let dir = default_rules_dir();
        std::fs::create_dir_all(&dir).expect("Failed to create rules directory");
        log::info!("Updater directory: {}", dir.display());
        Self { rules_dir: dir }
    }

    pub fn update(&self) -> Result<(), String> {
        let base_url = "https://raw.githubusercontent.com/GriffonAV/Griffon_scan_db/main";

        let manifest_url = format!("{}/manifest.toml", base_url);
        let manifest_content = reqwest::blocking::get(manifest_url)
            .map_err(|e| format!("Failed to download manifest: {}", e))?
            .text()
            .map_err(|e| format!("Failed to read manifest content: {}", e))?;
        let manifest: Manifest = toml::from_str(&manifest_content)
            .map_err(|e| format!("Failed to parse manifest: {}", e))?;

        let local_manifest_path = self.rules_dir.join("manifest.toml");
        if local_manifest_path.exists() {
            let local_manifest_content = std::fs::read_to_string(&local_manifest_path)
                .map_err(|e| format!("Failed to read local manifest: {}", e))?;
            let local_manifest: Manifest = toml::from_str(&local_manifest_content)
                .map_err(|e| format!("Failed to parse local manifest: {}", e))?;
            if local_manifest.version == manifest.version {
                log::info!(
                    "Rules are up to date (version {} released on {})",
                    manifest.version,
                    manifest.release_date
                );
                return Ok(());
            }
        }

        log::info!("Updating rules from {}", self.rules_dir.display());

        for rule in manifest.rules {
            let rule_url = format!("{}/{}", base_url, rule.filename);
            let rule_path = self.rules_dir.join(&rule.filename);
            if let Some(parent) = rule_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!("Failed to create directory {}: {}", parent.display(), e)
                })?;
            }
            log::info!("Downloading {} to {}", rule_url, rule_path.display());
            let response = reqwest::blocking::get(rule_url)
                .map_err(|e| format!("Failed to download {}: {}", rule.filename, e))?;
            if !response.status().is_success() {
                return Err(format!(
                    "Failed to download {}: HTTP {}",
                    rule.filename,
                    response.status()
                ));
            }
            let content = response
                .bytes()
                .map_err(|e| format!("Failed to read content of {}: {}", rule.filename, e))?;
            std::fs::write(&rule_path, &content)
                .map_err(|e| format!("Failed to save {}: {}", rule.filename, e))?;
        }

        Ok(())
    }
}
