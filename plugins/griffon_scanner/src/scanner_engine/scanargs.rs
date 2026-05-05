use crate::scanner_engine::yara_engine::threat_category::ThreatCategory;
use clap::Parser;
use std::path::PathBuf;

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

    // thread settings
    //--threads 4 — explicit count
    //--threads auto — let rayon decide (default, uses all cores)
    //--threads conservative — use max(1, cores / 2) to leave room for the rest of the system
    #[arg(long, default_value = "auto")]
    pub threads: String,

    #[arg(short, long, value_delimiter = ',')]
    pub include: Vec<ThreatCategory>,

    /// Exclude these categories (comma separated).
    #[arg(short, long, value_delimiter = ',')]
    pub exclude: Vec<ThreatCategory>,
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
            threads: "auto".to_string(),
            include: vec![],
            exclude: vec![],
        }
    }
}

impl ScanArgs {
    pub fn get_active_categories(&self) -> Vec<ThreatCategory> {
        let all = ThreatCategory::all();

        if !self.include.is_empty() {
            return self.include.clone();
        }

        all.iter()
            .filter(|&cat| !self.exclude.contains(cat))
            .cloned()
            .collect()
    }
}
