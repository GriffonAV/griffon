use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[cfg(debug_assertions)]
use std::{fs, io};

#[cfg(not(debug_assertions))]
use std::process::Command;

#[cfg(not(debug_assertions))]
const PLUGIN_INSTALLER_PATH: &str =
    "/usr/libexec/griffon-plugin-installer";

#[derive(Debug, Serialize, Deserialize)]
pub struct InstalledPlugin {
    toml_path: String,
    so_path: String,
    plugin_dir: String,
}

#[cfg(debug_assertions)]
fn griffon_config_dir() -> Result<PathBuf, String> {
    let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let project_root = src_tauri_dir
        .parent()
        .and_then(|gui_dir| gui_dir.parent())
        .ok_or_else(|| {
            "Unable to resolve Griffon project root".to_string()
        })?;

    Ok(project_root.join(".config").join("griffon"))
}

fn ensure_extension(
    path: &Path,
    expected: &str,
) -> Result<(), String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    if extension.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "Invalid file extension for {}. Expected .{}",
            path.display(),
            expected
        ))
    }
}

/*
 * --------------------------------------------------------------------------
 * Plugin installation
 * --------------------------------------------------------------------------
 */

#[tauri::command]
pub async fn install_plugin_zip(
    zip_path: String,
) -> Result<InstalledPlugin, String> {
    tauri::async_runtime::spawn_blocking(move || {
        install_plugin_zip_inner(zip_path)
    })
        .await
        .map_err(|error| {
            format!("Plugin installation task failed: {error}")
        })?
}

fn install_plugin_zip_inner(
    zip_path: String,
) -> Result<InstalledPlugin, String> {
    let zip_source = PathBuf::from(&zip_path);

    if !zip_source.is_file() {
        return Err(format!(
            "ZIP file does not exist: {}",
            zip_source.display()
        ));
    }

    ensure_extension(&zip_source, "zip")?;

    #[cfg(debug_assertions)]
    {
        install_plugin_zip_dev(&zip_source)
    }

    #[cfg(not(debug_assertions))]
    {
        install_plugin_zip_release(&zip_source)
    }
}

#[cfg(debug_assertions)]
fn install_plugin_zip_dev(
    zip_source: &Path,
) -> Result<InstalledPlugin, String> {
    let plugin_dir = griffon_config_dir()?;

    fs::create_dir_all(&plugin_dir).map_err(|error| {
        format!(
            "Failed to create plugin directory {}: {error}",
            plugin_dir.display()
        )
    })?;

    let zip_file = fs::File::open(zip_source).map_err(|error| {
        format!(
            "Failed to open ZIP file {}: {error}",
            zip_source.display()
        )
    })?;

    let mut archive =
        zip::ZipArchive::new(zip_file).map_err(|error| {
            format!("Failed to read ZIP archive: {error}")
        })?;

    let mut installed_toml: Option<PathBuf> = None;
    let mut installed_so: Option<PathBuf> = None;

    for index in 0..archive.len() {
        let mut archive_file =
            archive.by_index(index).map_err(|error| {
                format!(
                    "Failed to access file in ZIP archive: {error}"
                )
            })?;

        let enclosed_path = archive_file
            .enclosed_name()
            .ok_or_else(|| {
                format!(
                    "Unsafe path found in ZIP archive at index {index}"
                )
            })?
            .to_owned();

        let extension = enclosed_path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if extension != "toml" && extension != "so" {
            continue;
        }

        if extension == "toml" && installed_toml.is_some() {
            return Err(
                "Multiple TOML manifests found in archive".to_string(),
            );
        }

        if extension == "so" && installed_so.is_some() {
            return Err(
                "Multiple shared libraries found in archive".to_string(),
            );
        }

        let file_name = enclosed_path
            .file_name()
            .ok_or_else(|| {
                "Invalid file name in ZIP archive".to_string()
            })?;

        let destination = plugin_dir.join(file_name);

        let mut output_file =
            fs::File::create(&destination).map_err(|error| {
                format!(
                    "Failed to create {}: {error}",
                    destination.display()
                )
            })?;

        io::copy(&mut archive_file, &mut output_file).map_err(
            |error| {
                format!(
                    "Failed to write {}: {error}",
                    destination.display()
                )
            },
        )?;

        match extension.as_str() {
            "toml" => installed_toml = Some(destination),
            "so" => installed_so = Some(destination),
            _ => {}
        }
    }

    let toml_path = installed_toml.ok_or_else(|| {
        "No .toml file found in the ZIP archive".to_string()
    })?;

    let so_path = installed_so.ok_or_else(|| {
        "No .so file found in the ZIP archive".to_string()
    })?;

    Ok(InstalledPlugin {
        toml_path: toml_path.to_string_lossy().into_owned(),
        so_path: so_path.to_string_lossy().into_owned(),
        plugin_dir: plugin_dir.to_string_lossy().into_owned(),
    })
}

#[cfg(not(debug_assertions))]
fn install_plugin_zip_release(
    zip_source: &Path,
) -> Result<InstalledPlugin, String> {
    let canonical_archive =
        zip_source.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve ZIP archive {}: {error}",
                zip_source.display()
            )
        })?;

    let output = Command::new("pkexec")
        .arg(PLUGIN_INSTALLER_PATH)
        .arg("install")
        .arg("--archive")
        .arg(&canonical_archive)
        .output()
        .map_err(|error| {
            format!(
                "Failed to start administrator authentication: {error}"
            )
        })?;

    if !output.status.success() {
        return Err(format_helper_error(
            output.status.code(),
            &output.stderr,
            "Plugin installation",
        ));
    }

    serde_json::from_slice::<InstalledPlugin>(&output.stdout)
        .map_err(|error| {
            let stdout =
                String::from_utf8_lossy(&output.stdout);

            format!(
                "Invalid response from plugin installer: {error}. Response: {}",
                stdout.trim()
            )
        })
}

/*
 * --------------------------------------------------------------------------
 * Plugin deletion
 * --------------------------------------------------------------------------
 */

#[tauri::command]
pub async fn delete_plugin(
    plugin_uuid: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        delete_plugin_inner(&plugin_uuid)
    })
        .await
        .map_err(|error| {
            format!("Plugin deletion task failed: {error}")
        })?
}

fn delete_plugin_inner(
    plugin_uuid: &str,
) -> Result<(), String> {
    let normalized_uuid = validate_plugin_uuid(plugin_uuid)?;

    #[cfg(debug_assertions)]
    {
        delete_plugin_dev(&normalized_uuid)
    }

    #[cfg(not(debug_assertions))]
    {
        delete_plugin_release(&normalized_uuid)
    }
}

fn validate_plugin_uuid(
    plugin_uuid: &str,
) -> Result<String, String> {
    let plugin_uuid = plugin_uuid.trim();

    Uuid::parse_str(plugin_uuid)
        .map(|uuid| uuid.to_string())
        .map_err(|error| {
            format!(
                "Invalid plugin UUID \"{plugin_uuid}\": {error}"
            )
        })
}

#[cfg(debug_assertions)]
fn delete_plugin_dev(
    plugin_uuid: &str,
) -> Result<(), String> {
    let plugin_dir = griffon_config_dir()?;

    let (manifest_path, library_path) =
        find_plugin_files_by_uuid(&plugin_dir, plugin_uuid)?;

    let mut deleted_files = 0;

    // Delete the library first so that the manifest still identifies
    // the plugin if deleting the library fails.
    for path in [&library_path, &manifest_path] {
        if !path.exists() {
            continue;
        }

        fs::remove_file(path).map_err(|error| {
            format!(
                "Failed to delete {}: {error}",
                path.display()
            )
        })?;

        deleted_files += 1;
    }

    if deleted_files == 0 {
        return Err(format!(
            "No files were deleted for plugin UUID {plugin_uuid}"
        ));
    }

    Ok(())
}

#[cfg(debug_assertions)]
fn find_plugin_files_by_uuid(
    plugin_dir: &Path,
    plugin_uuid: &str,
) -> Result<(PathBuf, PathBuf), String> {
    if !plugin_dir.is_dir() {
        return Err(format!(
            "Plugin directory does not exist: {}",
            plugin_dir.display()
        ));
    }

    let canonical_plugin_dir =
        plugin_dir.canonicalize().map_err(|error| {
            format!(
                "Failed to resolve plugin directory {}: {error}",
                plugin_dir.display()
            )
        })?;

    let entries =
        fs::read_dir(&canonical_plugin_dir).map_err(|error| {
            format!(
                "Failed to read plugin directory {}: {error}",
                canonical_plugin_dir.display()
            )
        })?;

    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "Failed to read plugin directory entry: {error}"
            )
        })?;

        let manifest_path = entry.path();

        let is_toml = manifest_path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                extension.eq_ignore_ascii_case("toml")
            });

        if !is_toml {
            continue;
        }

        let canonical_manifest =
            manifest_path.canonicalize().map_err(|error| {
                format!(
                    "Failed to resolve manifest {}: {error}",
                    manifest_path.display()
                )
            })?;

        if !canonical_manifest.starts_with(&canonical_plugin_dir) {
            return Err(format!(
                "Plugin manifest is outside the plugin directory: {}",
                canonical_manifest.display()
            ));
        }

        let manifest_content =
            fs::read_to_string(&canonical_manifest).map_err(
                |error| {
                    format!(
                        "Failed to read manifest {}: {error}",
                        canonical_manifest.display()
                    )
                },
            )?;

        let manifest: toml::Value =
            toml::from_str(&manifest_content).map_err(
                |error| {
                    format!(
                        "Failed to parse manifest {}: {error}",
                        canonical_manifest.display()
                    )
                },
            )?;

        let manifest_uuid = find_uuid_in_toml(&manifest);

        if manifest_uuid.is_some_and(|uuid| {
            uuid.eq_ignore_ascii_case(plugin_uuid)
        }) {
            let library_path =
                canonical_manifest.with_extension("so");

            return Ok((canonical_manifest, library_path));
        }
    }

    Err(format!(
        "No plugin manifest found for UUID {plugin_uuid}"
    ))
}

#[cfg(debug_assertions)]
fn find_uuid_in_toml(
    value: &toml::Value,
) -> Option<&str> {
    match value {
        toml::Value::Table(table) => {
            if let Some(uuid) = table
                .get("uuid")
                .and_then(toml::Value::as_str)
            {
                return Some(uuid);
            }

            table.values().find_map(find_uuid_in_toml)
        }

        toml::Value::Array(values) => {
            values.iter().find_map(find_uuid_in_toml)
        }

        _ => None,
    }
}

#[cfg(not(debug_assertions))]
fn delete_plugin_release(
    plugin_uuid: &str,
) -> Result<(), String> {
    let output = Command::new("pkexec")
        .arg(PLUGIN_INSTALLER_PATH)
        .arg("remove")
        .arg("--uuid")
        .arg(plugin_uuid)
        .output()
        .map_err(|error| {
            format!(
                "Failed to start administrator authentication: {error}"
            )
        })?;

    if !output.status.success() {
        return Err(format_helper_error(
            output.status.code(),
            &output.stderr,
            "Plugin deletion",
        ));
    }

    Ok(())
}

/*
 * --------------------------------------------------------------------------
 * Release helper errors
 * --------------------------------------------------------------------------
 */

#[cfg(not(debug_assertions))]
fn format_helper_error(
    status_code: Option<i32>,
    stderr: &[u8],
    operation: &str,
) -> String {
    let stderr = String::from_utf8_lossy(stderr)
        .trim()
        .to_string();

    match status_code {
        Some(126) => {
            format!("{operation} was cancelled by the user")
        }

        Some(127) => {
            format!(
                "{operation} failed because administrator authorization was denied"
            )
        }

        _ if !stderr.is_empty() => stderr,

        Some(code) => {
            format!("{operation} failed with exit code {code}")
        }

        None => {
            format!("{operation} was terminated unexpectedly")
        }
    }
}