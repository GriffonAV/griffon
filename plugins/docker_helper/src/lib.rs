use std::process::Command;

use plugin_sdk::params::NoParams;
use plugin_sdk::{PluginRouter, export_plugin};
use serde::Serialize;

fn build_router() -> PluginRouter {
    PluginRouter::new().method("get_containers", get_containers)
}

export_plugin! {
    meta: PluginMeta {
        name: "Docker Helper",
        uuid: "3e9761ae-6061-4d87-ac45-81ae06f65740",
        author: "Alexis",
        description: "This is the Docker Helper plugin",
    },
    router: build_router,
}
#[derive(Serialize)] // Removed Deserialize, we will parse manually safely
struct Container {
    id: String,
    name: String,
    image: String,
    state: String,
    status: String,
}

#[derive(Serialize)]
struct ContainerList {
    containers: Vec<Container>,
}

fn get_containers(_args: NoParams) -> Result<ContainerList, String> {
    // 1. Add logging so you can see when the function starts
    log::info!("[LIBDOCKERH] Starting docker ps command...");

    let output = Command::new("docker")
        .arg("ps")
        .arg("-a")
        .arg("--format")
        .arg("{{json .}}")
        .output()
        .map_err(|e| {
            let msg = format!("Failed to execute docker command: {}", e);
            log::error!("[LIBDOCKERH] {}", msg);
            msg
        })?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        log::error!("[LIBDOCKERH] Docker CLI error: {}", err);
        return Err(format!("Docker CLI error: {}", err));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut containers = Vec::new();

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // 2. Safely parse just like your old code
        let v: serde_json::Value = match serde_json::from_str(line) {
            Ok(val) => val,
            Err(e) => {
                log::warn!(
                    "[LIBDOCKERH] Skipping unparseable line: {}. Err: {}",
                    line,
                    e
                );
                continue; // Skip lines that fail to parse without killing the whole plugin
            }
        };

        containers.push(Container {
            id: v["ID"].as_str().unwrap_or("Unknown").to_string(),
            name: v["Names"].as_str().unwrap_or("Unknown").to_string(),
            image: v["Image"].as_str().unwrap_or("Unknown").to_string(),
            state: v["State"].as_str().unwrap_or("Unknown").to_string(),
            status: v["Status"].as_str().unwrap_or("Unknown").to_string(),
        });
    }

    log::info!(
        "[LIBDOCKERH] Successfully returning {} containers",
        containers.len()
    );
    Ok(ContainerList { containers })
}
