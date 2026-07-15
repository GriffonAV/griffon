use clap::{Parser, Subcommand};
use serde::Serialize;
use std::{
    fs, io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::{self, ExitCode},
};
use uuid::Uuid;

const PLUGIN_DIRECTORY: &str = "/usr/lib/griffon/plugins";
const MAX_PLUGIN_FILE_SIZE: u64 = 256 * 1024 * 1024;

#[derive(Debug, Parser)]
#[command(name = "griffon-plugin-installer")]
#[command(about = "Privileged plugin installer for Griffon")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Install a Griffon plugin from a ZIP archive.
    Install {
        #[arg(long)]
        archive: PathBuf,
    },

    /// Remove an installed Griffon plugin using its UUID.
    Remove {
        #[arg(long)]
        uuid: String,
    },
}

#[derive(Debug, Serialize)]
struct InstalledPlugin {
    toml_path: String,
    so_path: String,
    plugin_dir: String,
}

fn main() -> ExitCode {
    match run() {
        Ok(Some(installed_plugin)) => match serde_json::to_string(&installed_plugin) {
            Ok(json) => {
                println!("{json}");
                ExitCode::SUCCESS
            }

            Err(error) => {
                eprintln!("Failed to serialize installer response: {error}");
                ExitCode::FAILURE
            }
        },

        Ok(None) => ExitCode::SUCCESS,

        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<Option<InstalledPlugin>, String> {
    let cli = Cli::parse();

    ensure_root()?;

    match cli.command {
        Commands::Install { archive } => {
            let installed_plugin = install_plugin_archive(&archive)?;
            Ok(Some(installed_plugin))
        }

        Commands::Remove { uuid } => {
            remove_plugin_by_uuid(&uuid)?;
            Ok(None)
        }
    }
}

fn ensure_root() -> Result<(), String> {
    if unsafe { libc::geteuid() } != 0 {
        return Err(
            "griffon-plugin-installer must be executed with administrator privileges".to_string(),
        );
    }

    Ok(())
}

/*
 * --------------------------------------------------------------------------
 * Installation
 * --------------------------------------------------------------------------
 */

fn install_plugin_archive(archive_path: &Path) -> Result<InstalledPlugin, String> {
    let archive_path = archive_path.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve archive {}: {error}",
            archive_path.display()
        )
    })?;

    if !archive_path.is_file() {
        return Err(format!(
            "Archive does not exist: {}",
            archive_path.display()
        ));
    }

    ensure_extension(&archive_path, "zip")?;

    let zip_file = fs::File::open(&archive_path)
        .map_err(|error| format!("Unable to open archive {}: {error}", archive_path.display()))?;

    let mut archive = zip::ZipArchive::new(zip_file)
        .map_err(|error| format!("Unable to read ZIP archive: {error}"))?;

    let temporary_directory = tempfile::tempdir()
        .map_err(|error| format!("Unable to create temporary directory: {error}"))?;

    let mut extracted_toml: Option<PathBuf> = None;
    let mut extracted_so: Option<PathBuf> = None;

    for index in 0..archive.len() {
        let mut archive_file = archive
            .by_index(index)
            .map_err(|error| format!("Unable to access ZIP entry {index}: {error}"))?;

        let enclosed_path = archive_file
            .enclosed_name()
            .ok_or_else(|| "Unsafe path found in ZIP archive".to_string())?
            .to_owned();

        let extension = enclosed_path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        if extension != "toml" && extension != "so" {
            continue;
        }

        if archive_file.size() > MAX_PLUGIN_FILE_SIZE {
            return Err(format!(
                "Plugin file is too large: {}",
                enclosed_path.display()
            ));
        }

        match extension.as_str() {
            "toml" if extracted_toml.is_some() => {
                return Err("Archive contains multiple TOML manifests".to_string());
            }

            "so" if extracted_so.is_some() => {
                return Err("Archive contains multiple shared libraries".to_string());
            }

            _ => {}
        }

        let file_name = enclosed_path
            .file_name()
            .ok_or_else(|| format!("Invalid file name in archive: {}", enclosed_path.display()))?;

        let extracted_path = temporary_directory.path().join(file_name);

        let mut extracted_file = fs::File::create(&extracted_path).map_err(|error| {
            format!(
                "Unable to create temporary file {}: {error}",
                extracted_path.display()
            )
        })?;

        io::copy(&mut archive_file, &mut extracted_file)
            .map_err(|error| format!("Unable to extract {}: {error}", enclosed_path.display()))?;

        match extension.as_str() {
            "toml" => extracted_toml = Some(extracted_path),
            "so" => extracted_so = Some(extracted_path),
            _ => {}
        }
    }

    let extracted_toml =
        extracted_toml.ok_or_else(|| "Archive does not contain a TOML manifest".to_string())?;

    let extracted_so =
        extracted_so.ok_or_else(|| "Archive does not contain a shared library".to_string())?;

    let plugin_directory = Path::new(PLUGIN_DIRECTORY);

    fs::create_dir_all(plugin_directory).map_err(|error| {
        format!(
            "Unable to create plugin directory {}: {error}",
            plugin_directory.display()
        )
    })?;

    let toml_destination =
        plugin_directory.join(extracted_toml.file_name().ok_or("Invalid TOML file name")?);

    let so_destination = plugin_directory.join(
        extracted_so
            .file_name()
            .ok_or("Invalid shared library file name")?,
    );

    atomic_install(&extracted_toml, &toml_destination)?;
    atomic_install(&extracted_so, &so_destination)?;

    Ok(InstalledPlugin {
        toml_path: toml_destination.to_string_lossy().into_owned(),

        so_path: so_destination.to_string_lossy().into_owned(),

        plugin_dir: plugin_directory.to_string_lossy().into_owned(),
    })
}

fn ensure_extension(path: &Path, expected: &str) -> Result<(), String> {
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();

    if extension.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(format!("Expected a .{expected} file: {}", path.display()))
    }
}

fn atomic_install(source: &Path, destination: &Path) -> Result<(), String> {
    let destination_name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("Invalid destination path: {}", destination.display()))?;

    let temporary_destination =
        destination.with_file_name(format!(".{destination_name}.{}.tmp", process::id()));

    fs::copy(source, &temporary_destination).map_err(|error| {
        format!(
            "Unable to copy {} to {}: {error}",
            source.display(),
            temporary_destination.display()
        )
    })?;

    fs::set_permissions(&temporary_destination, fs::Permissions::from_mode(0o644)).map_err(
        |error| {
            let _ = fs::remove_file(&temporary_destination);

            format!(
                "Unable to set permissions on {}: {error}",
                temporary_destination.display()
            )
        },
    )?;

    fs::rename(&temporary_destination, destination).map_err(|error| {
        let _ = fs::remove_file(&temporary_destination);

        format!("Unable to install {}: {error}", destination.display())
    })?;

    Ok(())
}

/*
 * --------------------------------------------------------------------------
 * Suppression
 * --------------------------------------------------------------------------
 */

fn remove_plugin_by_uuid(plugin_uuid: &str) -> Result<(), String> {
    let plugin_uuid = validate_plugin_uuid(plugin_uuid)?;

    let plugin_directory = Path::new(PLUGIN_DIRECTORY);

    if !plugin_directory.is_dir() {
        return Err(format!(
            "Plugin directory does not exist: {}",
            plugin_directory.display()
        ));
    }

    let canonical_plugin_directory = plugin_directory.canonicalize().map_err(|error| {
        format!(
            "Unable to resolve plugin directory {}: {error}",
            plugin_directory.display()
        )
    })?;

    let (manifest_path, library_path) =
        find_plugin_files_by_uuid(&canonical_plugin_directory, &plugin_uuid)?;

    /*
     * La bibliothèque est supprimée en premier.
     * Le manifeste reste donc disponible si sa suppression échoue ensuite.
     */
    remove_file_inside_plugin_directory(&library_path, &canonical_plugin_directory, false)?;

    remove_file_inside_plugin_directory(&manifest_path, &canonical_plugin_directory, true)?;

    Ok(())
}

fn validate_plugin_uuid(plugin_uuid: &str) -> Result<String, String> {
    let plugin_uuid = plugin_uuid.trim();

    Uuid::parse_str(plugin_uuid)
        .map(|uuid| uuid.to_string())
        .map_err(|error| format!("Invalid plugin UUID \"{plugin_uuid}\": {error}"))
}

fn find_plugin_files_by_uuid(
    plugin_directory: &Path,
    plugin_uuid: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let entries = fs::read_dir(plugin_directory).map_err(|error| {
        format!(
            "Unable to read plugin directory {}: {error}",
            plugin_directory.display()
        )
    })?;

    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Unable to read plugin directory entry: {error}"))?;

        let path = entry.path();

        let is_toml = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"));

        if !is_toml {
            continue;
        }

        let canonical_manifest = path
            .canonicalize()
            .map_err(|error| format!("Unable to resolve manifest {}: {error}", path.display()))?;

        if !canonical_manifest.starts_with(plugin_directory) {
            return Err(format!(
                "Manifest is outside the Griffon plugin directory: {}",
                canonical_manifest.display()
            ));
        }

        let manifest_content = fs::read_to_string(&canonical_manifest).map_err(|error| {
            format!(
                "Unable to read manifest {}: {error}",
                canonical_manifest.display()
            )
        })?;

        let manifest: toml::Value = match toml::from_str(&manifest_content) {
            Ok(manifest) => manifest,

            Err(error) => {
                eprintln!(
                    "Ignoring invalid manifest {}: {error}",
                    canonical_manifest.display()
                );
                continue;
            }
        };

        let Some(manifest_uuid) = find_uuid_in_toml(&manifest) else {
            continue;
        };

        let normalized_manifest_uuid = match Uuid::parse_str(manifest_uuid) {
            Ok(uuid) => uuid.to_string(),
            Err(_) => continue,
        };

        if normalized_manifest_uuid != plugin_uuid {
            continue;
        }

        let library_path = canonical_manifest.with_extension("so");

        return Ok((canonical_manifest, library_path));
    }

    Err(format!("No installed plugin found for UUID {plugin_uuid}"))
}

fn find_uuid_in_toml(value: &toml::Value) -> Option<&str> {
    match value {
        toml::Value::Table(table) => {
            if let Some(uuid) = table.get("uuid").and_then(toml::Value::as_str) {
                return Some(uuid);
            }

            table.values().find_map(find_uuid_in_toml)
        }

        toml::Value::Array(values) => values.iter().find_map(find_uuid_in_toml),

        _ => None,
    }
}

fn remove_file_inside_plugin_directory(
    path: &Path,
    plugin_directory: &Path,
    required: bool,
) -> Result<(), String> {
    if !path.exists() {
        if required {
            return Err(format!(
                "Required plugin file does not exist: {}",
                path.display()
            ));
        }

        return Ok(());
    }

    let canonical_path = path
        .canonicalize()
        .map_err(|error| format!("Unable to resolve plugin file {}: {error}", path.display()))?;

    if !canonical_path.starts_with(plugin_directory) {
        return Err(format!(
            "Refusing to delete a file outside the Griffon plugin directory: {}",
            canonical_path.display()
        ));
    }

    fs::remove_file(&canonical_path)
        .map_err(|error| format!("Unable to delete {}: {error}", canonical_path.display()))
}
