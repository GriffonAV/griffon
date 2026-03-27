use serde::{Deserialize, Serialize};
use toml::Value;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: Plugin,
    pub ui: UI,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub tabs: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UI {
    pub sections: Vec<Section>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub tab: String,
    pub contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub r#type: String,
    pub id: String,
    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

pub fn load_plugin_manifest(path: &str) -> Result<PluginManifest, Box<dyn std::error::Error>> {
    println!("{}", path);
    let toml_content = fs::read_to_string(path)?;
    let manifest: PluginManifest = toml::from_str(&toml_content)?;
    Ok(manifest)
}
