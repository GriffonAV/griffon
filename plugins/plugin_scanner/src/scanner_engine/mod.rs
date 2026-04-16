use crate::scanner_engine::data_type::ScanResult;
use clap::Parser;
use std::path::{Path, PathBuf};

pub mod data_type;
pub mod hash_scanner;
pub mod yara_engine;

#[derive(Parser, Debug)]
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
}

#[derive(Default)]
pub struct ScanEngine {
    pub hash_db: Option<hash_scanner::SignatureDb>,
    pub yara_rules: Option<yara_engine::YaraEngine>,
}

impl ScanEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load_hash_db(&mut self, args: &ScanArgs) -> anyhow::Result<()> {
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

    pub fn scan(&self, path: &Path, args: &ScanArgs) -> ScanResult {
        let mut results = Vec::new();

        if !args.yara_only
            && let Some(db) = &self.hash_db
        {
            println!(
                "Scanning with hash database containing {} signatures",
                db.count()
            );
            results.push(hash_scanner::hash::scan_file(path, db));
        }

        if let Some(rules) = &self.yara_rules {
            println!("Scanning with {} YARA rules", rules.rule_count());
            results.push(rules.scan_bytes(path, &std::fs::read(path).unwrap_or_default()));
        }

        let mut combined_threats = Vec::new();
        let mut combined_error = None;

        for result in results {
            combined_threats.extend(result.threats);
            if result.error.is_some() {
                combined_error = result.error;
            }
        }

        ScanResult {
            path: path.to_path_buf(),
            threats: combined_threats,
            error: combined_error,
        }
    }
}
