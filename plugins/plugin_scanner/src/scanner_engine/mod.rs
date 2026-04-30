use crate::scanner_engine::archive::{ArchiveKind, detect_archive};
#[allow(dead_code)]
#[allow(unused_variables)]
use crate::scanner_engine::data_type::ScanResult;
use clap::Parser;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

mod archive;
pub mod data_type;
pub mod hash_scanner;
pub mod yara_engine;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ScanArgs {
    // scan archive file: bool, default false
    #[arg(short, long)]
    pub scan_archives: bool,

    // recursive scan: bool, default true
    #[arg(short, long)]
    pub recursive: bool,

    // path to scan: string, no flag just the path
    pub path: PathBuf,

    // path to hash db: string, optional
    #[arg(short = 'H', long)]
    pub hash_db: Option<String>,
    // path to yara rules: string, optional
    #[arg(short, long)]
    pub yara_rules: Option<String>,

    // yara only
    #[arg(long)]
    pub yara_only: bool,

    // parallel scan: bool, default true
    #[arg(short, long)]
    pub parallel: bool,
}

impl Default for ScanArgs {
    fn default() -> Self {
        ScanArgs {
            scan_archives: false,
            recursive: true,
            path: PathBuf::new(),
            hash_db: None,
            yara_rules: None,
            yara_only: false,
            parallel: true,
        }
    }
}

#[derive(Default)]
pub struct ScanEngine {
    pub hash_db: Option<hash_scanner::SignatureDb>,
    pub yara_rules: Option<yara_engine::YaraEngine>,
    scan_args: ScanArgs,
}

impl ScanEngine {
    pub fn new() -> Self {
        env_logger::init();
        Self::default()
    }

    pub fn load_hash_db(&mut self, args: &ScanArgs) -> anyhow::Result<()> {
        log::warn!(
            "Experimental feature: Hash-based scanning is still in early development, use only yara rules for better results"
        );
        let path = Path::new(args.hash_db.as_deref().unwrap_or("signatures/hashes.txt"));
        let db = hash_scanner::SignatureDb::load(path)?;
        self.hash_db = Some(db);
        Ok(())
    }

    pub fn load_yara_rules(&mut self, args: &ScanArgs) -> anyhow::Result<()> {
        let path = Path::new(args.yara_rules.as_deref().unwrap_or("rules/"));
        let rules = yara_engine::YaraEngine::load_from_dir(path.to_str().unwrap_or(""))?;
        self.yara_rules = Some(rules);
        Ok(())
    }

    pub fn scan_file(&self, path: &Path) -> ScanResult {
        let mut threats = Vec::new();
        let mut error = None;

        if !self.scan_args.yara_only
            && let Some(db) = &self.hash_db
        {
            log::info!(
                "Scanning with hash database containing {} signatures",
                db.count()
            );
            // log::info!(
            //     "Scanning with hash database containing {} signatures",
            //     db.count()
            // );
            // let result = hash_scanner::hash::scan_file(path, db);
            // match result {
            //     Ok(r) => threats.extend(r.threats),
            //     Err(e) => error = Some(format!("Hash scan error: {}", e)),
            // }
        }

        let bytes = std::fs::read(path);
        let kind = detect_archive(&bytes.unwrap_or_default());
        if kind != ArchiveKind::Unknown {
            log::warn!(
                "Archive scanning is not implemented yet, skipping {}",
                path.display()
            );
            todo!();
        } else {
            return ScanResult::default();
        }

        if let Some(rules) = &self.yara_rules {
            log::info!("Scanning with {} YARA rules", rules.rule_count());
            match rules.scan_bytes(path, &std::fs::read(path).unwrap_or_default()) {
                Ok(mut t) => threats.append(&mut t),
                Err(e) => error = Some(format!("YARA scan error: {}", e)),
            }
        }

        ScanResult {
            number_file_scanned: 1,
            path: path.to_path_buf(),
            threats,
            error,
        }
    }

    pub fn scan_dir_parallel(&self, root: &Path) -> Vec<ScanResult> {
        let mut results = Vec::new();
        log::info!("Scanning directory {} in parallel", root.display());
        if self.scan_args.recursive {
            results = WalkDir::new(root)
                .follow_links(false) // don't follow symlinks
                .into_iter()
                .filter_map(|entry| entry.ok()) // skip permission errors
                .filter(|entry| entry.file_type().is_file()) // skip directories
                .par_bridge() // parallelize the iterator
                .map(|entry| self.scan_file(entry.path()))
                .collect();
        } else {
            results = std::fs::read_dir(root)
                .unwrap_or_else(|_| panic!("Failed to read directory: {}", root.display()))
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .par_bridge() // parallelize the iterator
                .map(|entry| self.scan_file(&entry.path()))
                .collect();
        }
        results
    }

    pub fn scan_directory(&self, root: &Path) -> Vec<ScanResult> {
        let mut results = Vec::new();
        if self.scan_args.parallel {
            return self.scan_dir_parallel(root);
        }

        if self.scan_args.recursive {
            results = WalkDir::new(root)
                .follow_links(false) // don't follow symlinks
                .into_iter()
                .filter_map(|entry| entry.ok()) // skip permission errors
                .filter(|entry| entry.file_type().is_file()) // skip directories
                .map(|entry| self.scan_file(entry.path()))
                .collect();
        } else {
            results = std::fs::read_dir(root)
                .unwrap_or_else(|_| panic!("Failed to read directory: {}", root.display()))
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.file_type().map(|ft| ft.is_file()).unwrap_or(false))
                .map(|entry| self.scan_file(&entry.path()))
                .collect();
        }
        results
    }

    pub fn scan(&mut self, path: &Path, args: &ScanArgs) -> ScanResult {
        self.scan_args = args.clone();
        let scan = ScanResult {
            path: path.to_path_buf(),
            ..Default::default()
        };

        //check if its a file or directory / or archive if scan_archives is true
        if path.is_file() {
            return self.scan_file(path);
        } else if path.is_dir() {
            return self.scan_directory(path).into_iter().fold(
                ScanResult::default(),
                |mut acc, r| {
                    acc.number_file_scanned += r.number_file_scanned;
                    acc.threats.extend(r.threats);
                    if r.error.is_some() {
                        acc.error = r.error;
                    }
                    acc
                },
            );
        } else {
            todo!();
            if args.scan_archives {
                log::warn!(
                    "Archive scanning is not implemented yet, skipping {}",
                    path.display()
                );
                todo!();
            }
        }

        return scan;
    }
}
