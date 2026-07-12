use serde::Serialize;
use std::{
    fs, io,
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

#[tauri::command]
pub fn install_plugin_zip(zip_path: String) -> Result<InstalledPlugin, String> {
    let zip_src = PathBuf::from(&zip_path);

    if !zip_src.is_file() {
        return Err(format!("ZIP file does not exist: {}", zip_path));
    }

    ensure_extension(&zip_src, "zip")?;

    let plugin_dir = griffon_config_dir()?;

    fs::create_dir_all(&plugin_dir)
        .map_err(|err| format!("Failed to create plugin directory: {}", err))?;

    let file =
        fs::File::open(&zip_src).map_err(|err| format!("Failed to open zip file: {}", err))?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|err| format!("Failed to read zip archive: {}", err))?;

    let mut installed_toml = None;
    let mut installed_so = None;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|err| format!("Failed to access file in zip: {}", err))?;

        let outpath = match file.enclosed_name() {
            Some(path) => path.to_owned(),
            None => continue,
        };

        let extension = outpath
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default();

        if extension == "toml" || extension == "so" {
            let file_name = outpath
                .file_name()
                .ok_or_else(|| "Invalid file name in zip".to_string())?;
            let dest_path = plugin_dir.join(file_name);

            let mut outfile = fs::File::create(&dest_path)
                .map_err(|err| format!("Failed to create output file {:?}: {}", dest_path, err))?;

            io::copy(&mut file, &mut outfile)
                .map_err(|err| format!("Failed to write to {:?}: {}", dest_path, err))?;

            if extension == "toml" {
                installed_toml = Some(dest_path);
            } else if extension == "so" {
                installed_so = Some(dest_path);
            }
        }
    }

    let toml_path =
        installed_toml.ok_or_else(|| "No .toml file found in the zip archive".to_string())?;
    let so_path = installed_so.ok_or_else(|| "No .so file found in the zip archive".to_string())?;

    Ok(InstalledPlugin {
        toml_path: toml_path.to_string_lossy().to_string(),
        so_path: so_path.to_string_lossy().to_string(),
        plugin_dir: plugin_dir.to_string_lossy().to_string(),
    })
}
