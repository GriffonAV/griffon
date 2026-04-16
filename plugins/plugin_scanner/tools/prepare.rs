// safe prepare script for proto-yara-x
// - clones a rules repo (YARA text rules)
// - extracts quoted strings from rules to use as "malicious patterns"
// - generates sample files, embedding patterns into some samples
// - optionally downloads EICAR test file (safe)

// Usage examples:
// cargo run --bin prepare -- --rules-repo https://github.com/Yara-Rules/rules --samples 200 --hit-rate 0.1 --fetch-eicar

//RUST_LOG=debug
use log::debug;
use rand::RngExt;

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use rand::seq::SliceRandom;
use regex::Regex;
use std::fs;
use std::io::Write;

fn main() -> Result<()> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    debug!("Args: {:?}", args);

    let force = arg_value(&args, "--force");
    let rules_repo = arg_value(&args, "--rules-repo")
        .unwrap_or_else(|| "https://github.com/Yara-Rules/rules".to_string());
    let rules_dir = arg_value(&args, "--rules-dir").unwrap_or_else(|| "rules".to_string());
    let samples_dir = arg_value(&args, "--samples-dir").unwrap_or_else(|| "samples".to_string());
    let samples_count: usize = arg_value(&args, "--samples")
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    let hit_rate: f64 = arg_value(&args, "--hit-rate")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.10);
    let fetch_eicar = args.iter().any(|a| a == "--fetch-eicar");

    debug!(
        "Prepare: rules_dir='{}', samples_dir='{}', samples_count={}, hit_rate={}",
        rules_dir, samples_dir, samples_count, hit_rate
    );

    ensure_git_available()?;
    if force.is_some() {
        debug!("--force specified: removing existing rules and samples directories.");
        let _ = fs::remove_dir_all(&rules_dir);
        let _ = fs::remove_dir_all(&samples_dir);
    }
    ensure_rules_cloned(&rules_repo, &rules_dir)?;

    let patterns = extract_patterns_from_rules(&rules_dir)?;
    debug!(
        "Extracted {} pattern strings from rules (will use up to 200 unique).",
        patterns.len()
    );

    generate_samples(&samples_dir, &patterns, samples_count, hit_rate)?;
    if fetch_eicar {
        download_eicar_files()?;
    }

    debug!(
        "Preparation done. rules OK ({}/), samples OK ({}/){}",
        rules_dir,
        samples_dir,
        if fetch_eicar { ", EICAR fetched" } else { "" }
    );
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

fn ensure_git_available() -> Result<()> {
    match Command::new("git").arg("--version").output() {
        Ok(o) if o.status.success() => Ok(()),
        _ => Err(anyhow::anyhow!(
            "`git` not found in PATH. Please install git to allow cloning rules."
        )),
    }
}

fn ensure_rules_cloned(repo: &str, dir: &str) -> Result<()> {
    let p = Path::new(dir);
    if p.exists() && p.read_dir()?.next().is_some() {
        debug!(
            "Rules directory '{}' already exists and is not empty — skipping clone.",
            dir
        );
        return Ok(());
    }

    debug!("Cloning rules repo from {} into {} ...", repo, dir);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", repo, dir])
        .status()
        .context("git clone failed")?;

    if !status.success() {
        return Err(anyhow::anyhow!("git clone returned non-zero status"));
    }

    debug!("Cleaning up cloned rules directory: keeping only 'malware/' and 'email/'");
    for entry in fs::read_dir(p)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(file_name) = path.file_name().and_then(|s| s.to_str())
            && file_name != "malware"
            && file_name != "email"
        {
            if path.is_dir() {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove directory: {}", path.display()))?;
                debug!("Removed directory: {}", path.display());
            } else {
                fs::remove_file(&path)
                    .with_context(|| format!("Failed to remove file: {}", path.display()))?;
                debug!("Removed file: {}", path.display());
            }
        }
    }
    Ok(())
}

fn extract_patterns_from_rules(rules_dir: &str) -> Result<Vec<String>> {
    let mut patterns = Vec::new();
    let q_re = Regex::new(r#""([^"]+)""#).unwrap();

    for entry in walk_files(rules_dir)? {
        if let Some(ext) = entry.extension().and_then(|s| s.to_str())
            && (ext.eq_ignore_ascii_case("yar") || ext.eq_ignore_ascii_case("yara"))
            && let Ok(text) = fs::read_to_string(&entry)
        {
            for cap in q_re.captures_iter(&text) {
                if let Some(m) = cap.get(1) {
                    let s = m.as_str().trim().to_string();
                    if (4..=200).contains(&s.len()) {
                        patterns.push(s);
                    }
                }
            }
        }
    }

    patterns.sort();
    patterns.dedup();
    let mut rng = rand::rng();
    patterns.shuffle(&mut rng);
    patterns.truncate(std::cmp::min(patterns.len(), 200));
    Ok(patterns)
}

fn walk_files(dir: &str) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    let p = Path::new(dir);
    if !p.exists() {
        return Ok(out);
    }

    fn recur(p: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
        for ent in fs::read_dir(p)? {
            let ent = ent?;
            let path = ent.path();
            if path.is_dir() {
                recur(&path, out)?;
            } else if path.is_file() {
                out.push(path);
            }
        }
        Ok(())
    }
    recur(p, &mut out).context("failed to walk files")?;
    Ok(out)
}

fn generate_samples(
    samples_dir: &str,
    patterns: &[String],
    count: usize,
    hit_rate: f64,
) -> Result<()> {
    fs::create_dir_all(samples_dir)?;

    let mut rng = rand::rng();

    for i in 0..count {
        let is_hit = rng.random_bool(hit_rate);
        let filename = format!("{}/sample_{:05}.bin", samples_dir, i);
        let mut file = fs::File::create(&filename)?;
        if is_hit && !patterns.is_empty() {
            hit_file(&mut file, patterns)?;
        } else {
            let size = rng.random_range(1024..10 * 1024); // 1KB to 10KB
            random_begnim_file(&mut file, size)?;
        }
    }

    debug!(
        "Generated {} samples in '{}'. (approx {} hits)",
        count,
        samples_dir,
        (count as f64 * hit_rate).round()
    );
    Ok(())
}

fn hit_file(file: &mut fs::File, patterns: &[String]) -> Result<()> {
    // pick 1-3 patterns and embed them
    let mut rng = rand::rng();
    let patterns_to_embed = rng.random_range(1..=3).min(patterns.len());
    let mut selected_patterns = Vec::new();
    for _ in 0..patterns_to_embed {
        let patern_index = rng.random_range(0..patterns.len());
        selected_patterns.push(&patterns[patern_index]);
    }
    file.write_all(b"---BEGIN MALICIOUS CONTENT---\n")?;
    for pattern in selected_patterns {
        file.write_all(pattern.as_bytes())?;
        file.write_all(b"\n")?;
    }
    file.write_all(b"--END OF MALICIOUS CONTENT---\n")?;
    Ok(())
}

fn random_begnim_file(file: &mut fs::File, size: usize) -> Result<()> {
    let mut rng = rand::rng();
    let mut buffer = vec![0u8; size];
    rng.fill(&mut buffer[..]);
    file.write_all(&buffer)?;
    Ok(())
}

fn download_eicar_files() -> Result<()> {
    let eicar_urls = [
        "https://secure.eicar.org/eicar.com",
        "https://secure.eicar.org/eicar.com.txt",
        "https://secure.eicar.org/eicar_com.zip",
        "https://secure.eicar.org/eicar_com2.zip",
    ];

    let eicar_dir = Path::new("eicar_files");
    fs::create_dir_all(eicar_dir)?;

    let client = reqwest::blocking::Client::new();

    for url in eicar_urls {
        let filename = url
            .split('/')
            .next_back()
            .expect("URL should have a filename");

        let dest_path = eicar_dir.join(filename);

        debug!("Downloading {} -> {:?}", url, dest_path);

        let mut response = client.get(url).send()?.error_for_status()?;
        let mut file = fs::File::create(dest_path)?;

        response.copy_to(&mut file)?;
    }

    Ok(())
}
