use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Serialize)]
pub struct InstalledPlugin {
    toml_path: String,
    so_path: String,
    plugin_dir: String,
}

fn griffon_config_dir() -> Result<PathBuf, String> {
    let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let project_root = src_tauri_dir
        .parent()
        .and_then(|gui_dir| gui_dir.parent())
        .ok_or_else(|| "Unable to resolve Griffon project root".to_string())?;

    Ok(project_root.join(".config").join("griffon"))
}

fn ensure_extension(path: &Path, expected: &str) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();

    if ext.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!(
            "Invalid file extension for {:?}. Expected .{}",
            path, expected
        ))
    }
}

fn copy_file_to_dir(src: &Path, dest_dir: &Path) -> Result<PathBuf, String> {
    let file_name = src
        .file_name()
        .ok_or_else(|| "Invalid file name".to_string())?;

    let dest = dest_dir.join(file_name);

    fs::copy(src, &dest)
        .map_err(|err| format!("Failed to copy {:?} to {:?}: {}", src, dest, err))?;

    Ok(dest)
}

#[tauri::command]
pub fn install_plugin_files(toml_path: String, so_path: String) -> Result<InstalledPlugin, String> {
    let toml_src = PathBuf::from(&toml_path);
    let so_src = PathBuf::from(&so_path);

    if !toml_src.is_file() {
        return Err(format!("TOML file does not exist: {}", toml_path));
    }

    if !so_src.is_file() {
        return Err(format!("SO file does not exist: {}", so_path));
    }

    ensure_extension(&toml_src, "toml")?;
    ensure_extension(&so_src, "so")?;

    let plugin_dir = griffon_config_dir()?;

    fs::create_dir_all(&plugin_dir)
        .map_err(|err| format!("Failed to create plugin directory: {}", err))?;

    let installed_toml = copy_file_to_dir(&toml_src, &plugin_dir)?;
    let installed_so = copy_file_to_dir(&so_src, &plugin_dir)?;

    Ok(InstalledPlugin {
        toml_path: installed_toml.to_string_lossy().to_string(),
        so_path: installed_so.to_string_lossy().to_string(),
        plugin_dir: plugin_dir.to_string_lossy().to_string(),
    })
}
