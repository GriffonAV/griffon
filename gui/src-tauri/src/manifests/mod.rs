use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct PluginManifest {
    pub plugin: Plugin,
    pub ui: UI,
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub tabs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UI {
    pub sections: Vec<Section>,
}

#[derive(Debug, Deserialize)]
pub struct Section {
    pub id: String,
    pub section: String,
}

pub fn load_plugin_manifest(path: &str) -> Result<PluginManifest, Box<dyn std::error::Error>> {
    let toml_content = fs::read_to_string(path)?;
    let manifest: PluginManifest = toml::from_str(&toml_content)?;
    Ok(manifest)
}
