use serde::Deserialize;

// threading enum auto, max, custom

#[derive(Deserialize, Debug, Default)]
pub enum Threading {
    #[default]
    Auto,
    Max,
    Custom,
}

#[derive(Deserialize, Default)]
struct NoArgs {}

#[derive(Deserialize, Debug)]
pub struct ScanOptions {
    paths: Vec<String>,

    #[serde(default)]
    archive: bool,

    #[serde(default)]
    folder: bool,

    #[serde(default)]
    threading: Threading,

    #[serde(default)]
    threads: u32,

    #[serde(default)]
    threats: Vec<String>,
}

#[derive(Deserialize)]
pub struct PathTarget {
    path: String,
}

#[derive(Deserialize)]
struct QuarantineTarget {
    /// Name of the quarantined item, as returned by `q_list`.
    name: String,
}
