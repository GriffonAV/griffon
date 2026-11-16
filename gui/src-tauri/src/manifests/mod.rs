use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use toml::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: Plugin,
    pub ui: UI,

    #[serde(default)]
    pub store: HashMap<String, Value>,

    #[serde(default)]
    pub interactions: Vec<Interaction>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Plugin {
    pub name: String,
    pub id: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub uuid: String,

    #[serde(default)]
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

    #[serde(default)]
    pub title: Option<String>,

    #[serde(default)]
    pub description: Option<String>,

    pub contents: Vec<Content>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Content {
    pub r#type: String,
    pub id: String,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub on: String,

    #[serde(default)]
    pub steps: Vec<InteractionStep>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InteractionStep {
    pub r#type: String,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(default)]
    pub value: Option<Value>,

    #[serde(default)]
    pub from: Option<String>,

    #[serde(default)]
    pub amount: Option<f64>,

    #[serde(flatten)]
    pub unknown: HashMap<String, Value>,
}

pub fn load_plugin_manifest(path: &str) -> Result<PluginManifest, Box<dyn std::error::Error>> {
    // println!("{}", path);
    let toml_content = fs::read_to_string(path)?;
    let manifest: PluginManifest = toml::from_str(&toml_content)?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_metadata_and_declarative_component_items_are_preserved() {
        let manifest: PluginManifest = toml::from_str(
            r#"
                [plugin]
                name = "Fixture"
                id = "0f602a6e-76a9-4a8a-a482-3c3784fe60e3"
                version = "1.0.0"
                author = "Griffon"
                description = "Fixture"
                uuid = "0f602a6e-76a9-4a8a-a482-3c3784fe60e3"
                tabs = ["overview"]

                [store]

                [[ui.sections]]
                id = "overview"
                tab = "overview"
                title = "Security overview"
                description = "Summary"

                [[ui.sections.contents]]
                type = "metric_grid"
                id = "metrics"

                [[ui.sections.contents.items]]
                label = "Critical"
                from = "store.critical"
                tone = "danger"

                [[ui.sections.contents]]
                type = "table"
                id = "findings"
                from = "store.findings"
                selectable = false
                searchable = true
                filter_key = "severity"
                filter_keys = ["severity", "category"]
                layout = "cards"

                [[ui.sections.contents.columns]]
                key = "severity"
                label = "Severity"
                cell = "badge"
                tone_map = { High = "danger", Medium = "warning" }
            "#,
        )
        .expect("fixture manifest should deserialize");

        let section = &manifest.ui.sections[0];
        assert_eq!(section.title.as_deref(), Some("Security overview"));
        assert_eq!(section.description.as_deref(), Some("Summary"));
        assert!(section.contents[0].unknown["items"].is_array());
        assert_eq!(
            section.contents[1].unknown["selectable"].as_bool(),
            Some(false)
        );
        assert_eq!(
            section.contents[1].unknown["searchable"].as_bool(),
            Some(true)
        );
        assert_eq!(
            section.contents[1].unknown["filter_key"].as_str(),
            Some("severity")
        );
        assert_eq!(
            section.contents[1].unknown["layout"].as_str(),
            Some("cards")
        );
        assert_eq!(
            section.contents[1].unknown["filter_keys"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
        assert_eq!(
            section.contents[1].unknown["columns"][0]["tone_map"]["High"].as_str(),
            Some("danger")
        );
    }
}
