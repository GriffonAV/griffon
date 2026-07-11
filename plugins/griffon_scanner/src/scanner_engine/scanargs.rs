use crate::scanner_engine::yara_engine::threat_category::ThreatCategory;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct ScanArgs {
    // scan archive file: bool, default false
    #[arg(short, long, default_value_t = false)]
    pub archives: bool,

    // recursive scan: bool, default true
    #[arg(short, long, default_value_t = false)]
    pub recursive: bool,

    #[arg(value_name = "PATH", required = true)]
    pub paths: Vec<PathBuf>,

    // yara only
    #[arg(long, default_value_t = true)]
    pub yara_only: bool,

    // thread settings
    //--threads 4 — explicit count
    //--threads auto — let rayon decide (default, uses all cores)
    //--threads conservative — use max(1, cores / 2) to leave room for the rest of the system
    //--threads off — disables parallelism, runs sequentially
    #[arg(long, default_value = "auto")]
    pub threads: String,

    #[arg(long, default_value_t = 1)]
    pub nb_threads: u32,

    #[arg(short, long, value_delimiter = ',')]
    pub include: Vec<ThreatCategory>,

    /// Exclude these categories (comma separated).
    #[arg(short, long, value_delimiter = ',')]
    pub exclude: Vec<ThreatCategory>,
}

impl Default for ScanArgs {
    fn default() -> Self {
        ScanArgs {
            archives: true,
            recursive: true,
            paths: vec![],
            yara_only: false,
            threads: "auto".to_string(),
            nb_threads: 0,
            include: vec![],
            exclude: vec![],
        }
    }
}

impl ScanArgs {
    pub fn get_active_categories(&self) -> Vec<ThreatCategory> {
        let all = ThreatCategory::all();

        if self.include.contains(&ThreatCategory::All) {
            return all.to_vec();
        }

        if !self.include.is_empty() {
            return self.include.clone();
        }

        all.iter()
            .filter(|&cat| !self.exclude.contains(cat))
            .cloned()
            .collect()
    }
}

#[derive(Parser, Debug, Clone, Default)]
#[command(version, about, long_about = None)]
pub struct PrepArgs {
    // path to hash db: string, optional
    #[arg(short = 'H', long)]
    pub hash_db: Option<String>,
    // path to yara rules: string, optional
    #[arg(short, long)]
    pub yara_rules: Option<String>,

    // yara only
    #[arg(long)]
    pub yara_only: bool,

    // thread settings
    //--threads 4 — explicit count
    //--threads auto — let rayon decide (default, uses all cores)
    //--threads conservative — use max(1, cores / 2) to leave room for the rest of the system
    #[arg(long, default_value = "auto")]
    pub threads: String,
}
