use serde::Serialize;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

const HISTORY_DIRS: &[&str] = &[
    "/tmp/griffon/plugin-history",
    "/var/log/griffon/plugin-history",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginHistoryEntry {
    pub timestamp: i64,
    pub level: String,
    pub plugin_name: String,
    pub plugin_uuid: String,
    pub event: Option<String>,
    pub pid: Option<String>,
    pub path: Option<String>,
    pub message: String,
    pub source_file: String,
}

#[tauri::command]
pub fn get_plugin_history() -> Result<Vec<PluginHistoryEntry>, String> {
    let mut entries = Vec::new();

    for dir in HISTORY_DIRS {
        let dir_path = Path::new(dir);

        if !dir_path.exists() || !dir_path.is_dir() {
            continue;
        }

        let files = fs::read_dir(dir_path)
            .map_err(|e| format!("Cannot read history directory {}: {}", dir, e))?;

        for file in files {
            let file = match file {
                Ok(f) => f,
                Err(_) => continue,
            };

            let path = file.path();

            if !path.is_file() {
                continue;
            }

            let file_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            if !file_name.starts_with("history_") {
                continue;
            }

            let fallback_uuid = file_name.trim_start_matches("history_").to_string();

            let opened_file = match File::open(&path) {
                Ok(f) => f,
                Err(_) => continue,
            };

            let reader = BufReader::new(opened_file);

            for line in reader.lines().map_while(Result::ok) {
                if line.trim().is_empty() {
                    continue;
                }

                if let Some(entry) =
                    parse_history_line(&line, &fallback_uuid, &path.to_string_lossy())
                {
                    entries.push(entry);
                }
            }
        }
    }

    entries.sort_by_key(|entry| std::cmp::Reverse(entry.timestamp));

    Ok(entries)
}

fn parse_history_line(
    line: &str,
    fallback_uuid: &str,
    source_file: &str,
) -> Option<PluginHistoryEntry> {
    let mut rest = line;
    let mut groups = Vec::new();

    while let Some(stripped) = rest.strip_prefix('[') {
        let (value, after) = stripped.split_once(']')?;
        groups.push(value.to_string());
        rest = after;

        if groups.len() == 4 {
            break;
        }
    }

    if groups.len() < 4 {
        return None;
    }

    let timestamp = groups[0].parse::<i64>().unwrap_or(0);
    let level = groups[1].clone();
    let plugin_name = groups[2].clone();
    let plugin_uuid = if groups[3].is_empty() {
        fallback_uuid.to_string()
    } else {
        groups[3].clone()
    };

    let message = rest.trim().to_string();

    Some(PluginHistoryEntry {
        timestamp,
        level,
        plugin_name,
        plugin_uuid,
        event: extract_value(&message, "event"),
        pid: extract_value(&message, "pid"),
        path: extract_value(&message, "path"),
        message,
        source_file: source_file.to_string(),
    })
}

fn extract_value(message: &str, key: &str) -> Option<String> {
    let prefix = format!("{}=", key);
    let start = message.find(&prefix)?;
    let value = &message[start + prefix.len()..];

    if let Some(stripped) = value.strip_prefix('"') {
        let end = stripped.find('"')?;
        Some(stripped[..end].to_string())
    } else {
        let end = value
            .find(|c: char| c.is_whitespace())
            .unwrap_or(value.len());

        Some(value[..end].to_string())
    }
}
