#[allow(dead_code)]
#[allow(unused_variables)]
use crate::scanner_engine::data_type::ScanReport;
use clap::Parser;
use std::path::{Path, PathBuf};

mod archive;
pub mod data_type;
pub mod hash_scanner;
pub mod load;
pub mod scan_dir;
pub mod scan_file;
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

    pub fn prepare(&mut self, args: &ScanArgs) -> Result<(i32, i32), String> {
        self.scan_args = args.clone();
        if !args.yara_only {
            self.load_hash_db(args)
                .expect("Failed to load signature DB");
        }
        self.load_yara_rules(args)
            .expect("Failed to load YARA rules");

        Ok((
            self.hash_db.as_ref().map_or(0, |db| db.count() as i32),
            self.yara_rules
                .as_ref()
                .map_or(0, |r| r.rule_count() as i32),
        ))
    }

    pub fn scan(&mut self, path: &Path, args: &ScanArgs) -> ScanReport {
        self.scan_args = args.clone();
        let mut report = ScanReport::default();

        if path.is_file() {
            let results = self.scan_file(path);
            report.add(results);
        } else if path.is_dir() {
            let results = self.scan_dir(path);
            report.add(results);
        }

        report
    }
}
