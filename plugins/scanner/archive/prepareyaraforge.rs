// Simplified prepare script for GriffonAV testing
// Downloads EICAR test files and YARA Forge rules

use anyhow::{Context, Result};
use log::{debug, info};
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

const EICAR_URLS: &[&str] = &[
    "https://secure.eicar.org/eicar.com",
    "https://secure.eicar.org/eicar.com.txt",
    "https://secure.eicar.org/eicar_com.zip",
];

const YARA_FORGE_BASE: &str = "https://github.com/YARAHQ/yara-forge/releases/latest/download/";

#[derive(Debug, Clone, Copy)]
enum RuleSet {
    Core,
    Extended,
    Full,
}

impl RuleSet {
    fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "core" => Some(Self::Core),
            "extended" => Some(Self::Extended),
            "full" => Some(Self::Full),
            _ => None,
        }
    }

    fn filename(&self) -> &'static str {
        match self {
            Self::Core => "yara-forge-rules-core.zip",
            Self::Extended => "yara-forge-rules-extended.zip",
            Self::Full => "yara-forge-rules-full.zip",
        }
    }

    fn url(&self) -> String {
        format!("{}{}", YARA_FORGE_BASE, self.filename())
    }
}

struct Config {
    eicar_dir: String,
    rules_dir: String,
    ruleset: RuleSet,
    skip_eicar: bool,
    skip_rules: bool,
    force: bool,
}

impl Config {
    fn from_args(args: &[String]) -> Self {
        Self {
            eicar_dir: arg_value(args, "--eicar-dir").unwrap_or_else(|| "eicar_files".to_string()),
            rules_dir: arg_value(args, "--rules-dir").unwrap_or_else(|| "rules".to_string()),
            ruleset: arg_value(args, "--ruleset")
                .and_then(|s| RuleSet::from_str(&s))
                .unwrap_or(RuleSet::Core),
            skip_eicar: args.iter().any(|a| a == "--skip-eicar"),
            skip_rules: args.iter().any(|a| a == "--skip-rules"),
            force: args.iter().any(|a| a == "--force"),
        }
    }
}

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return Ok(());
    }

    let config = Config::from_args(&args);

    info!("GriffonAV Test Data Preparation");
    info!("EICAR directory: {}", config.eicar_dir);
    info!("Rules directory: {}", config.rules_dir);
    info!("Ruleset: {:?}", config.ruleset);

    if config.force {
        info!("Force mode enabled - removing existing directories");
        let _ = fs::remove_dir_all(&config.eicar_dir);
        let _ = fs::remove_dir_all(&config.rules_dir);
    }

    if !config.skip_eicar {
        download_eicar_files(&config.eicar_dir)?;
    } else {
        info!("Skipping EICAR download (--skip-eicar)");
    }

    if !config.skip_rules {
        download_yara_forge_rules(&config.rules_dir, config.ruleset)?;
    } else {
        info!("Skipping YARA rules download (--skip-rules)");
    }

    info!("Preparation complete!");
    Ok(())
}

fn download_eicar_files(eicar_dir: &str) -> Result<()> {
    let dir = Path::new(eicar_dir);

    if dir.exists() && dir.read_dir()?.next().is_some() {
        info!("EICAR directory already exists and is not empty - skipping download");
        return Ok(());
    }

    info!("Creating EICAR directory: {}", eicar_dir);
    fs::create_dir_all(dir)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;

    for url in EICAR_URLS {
        let filename = url.split('/').last().unwrap();
        let dest_path = dir.join(filename);

        info!("Downloading {} -> {:?}", url, dest_path);

        match download_file(&client, url, &dest_path) {
            Ok(_) => info!("✓ Downloaded {}", filename),
            Err(e) => {
                log::warn!("✗ Failed to download {}: {}", filename, e);
            }
        }
    }

    Ok(())
}

fn download_yara_forge_rules(rules_dir: &str, ruleset: RuleSet) -> Result<()> {
    let dir = Path::new(rules_dir);

    if dir.exists() && dir.read_dir()?.next().is_some() {
        info!("Rules directory already exists and is not empty - skipping download");
        return Ok(());
    }

    info!("Creating rules directory: {}", rules_dir);
    fs::create_dir_all(dir)?;

    let url = ruleset.url();
    let zip_filename = ruleset.filename();
    let zip_path = dir.join(zip_filename);

    info!("Downloading YARA Forge {:?} ruleset", ruleset);
    info!("URL: {}", url);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?;

    download_file(&client, &url, &zip_path).context("Failed to download YARA Forge rules")?;

    info!("✓ Downloaded {}", zip_filename);
    info!("Extracting rules...");

    extract_zip(&zip_path, dir)?;

    info!("✓ Extracted rules to {}", rules_dir);

    // Optionally remove the zip file after extraction
    if let Err(e) = fs::remove_file(&zip_path) {
        debug!("Could not remove zip file: {}", e);
    }

    Ok(())
}

fn download_file(client: &reqwest::blocking::Client, url: &str, dest: &Path) -> Result<()> {
    let mut response = client
        .get(url)
        .send()
        .context("Failed to send request")?
        .error_for_status()
        .context("Server returned error status")?;

    let mut file = fs::File::create(dest).context(format!("Failed to create file: {:?}", dest))?;

    response
        .copy_to(&mut file)
        .context("Failed to write response to file")?;

    Ok(())
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    let file =
        fs::File::open(zip_path).context(format!("Failed to open zip file: {:?}", zip_path))?;

    let mut archive = zip::ZipArchive::new(file).context("Failed to read zip archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            debug!("Creating directory: {:?}", outpath);
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            debug!("Extracting: {:?}", outpath);
            let mut outfile = fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2).find_map(|w| {
        if w[0] == name {
            Some(w[1].clone())
        } else {
            None
        }
    })
}

fn print_help() {
    debug!("GriffonAV Test Data Preparation Tool");
    debug!();
    debug!("USAGE:");
    debug!("    cargo run --bin prepare [OPTIONS]");
    debug!();
    debug!("OPTIONS:");
    debug!("    --eicar-dir <DIR>      Directory for EICAR files (default: eicar_files)");
    debug!("    --rules-dir <DIR>      Directory for YARA rules (default: rules)");
    debug!(
        "    --ruleset <SET>        YARA Forge ruleset: core, extended, or full (default: core)"
    );
    debug!("    --skip-eicar           Skip downloading EICAR files");
    debug!("    --skip-rules           Skip downloading YARA rules");
    debug!("    --force                Remove existing directories before downloading");
    debug!("    --help, -h             Show this help message");
    debug!();
    debug!("EXAMPLES:");
    debug!("    # Download everything with defaults");
    debug!("    cargo run --bin prepare");
    debug!();
    debug!("    # Download extended ruleset");
    debug!("    cargo run --bin prepare --ruleset extended");
    debug!();
    debug!("    # Only download EICAR files");
    debug!("    cargo run --bin prepare --skip-rules");
    debug!();
    debug!("    # Force re-download everything");
    debug!("    cargo run --bin prepare --force");
    debug!();
    debug!("RULESETS:");
    debug!("    core      - Core detection rules (recommended for testing)");
    debug!("    extended  - Core + additional rules");
    debug!("    full      - Complete ruleset (large download)");
}
