#[allow(dead_code)]
#[allow(unused_variables)]
use crate::scanner_engine::data_type::ScanReport;
use crate::scanner_engine::scanargs::{PrepArgs, ScanArgs};
use std::path::Path;

mod archive;
pub mod data_type;
pub mod hash_scanner;
pub mod load;
pub mod scan_dir;
pub mod scan_file;
pub mod scanargs;
pub mod yara_engine;

use std::sync::OnceLock;
static THREAD_POOL_INIT: OnceLock<()> = OnceLock::new();

#[derive(Default)]
pub struct ScanEngine {
    pub hash_db: Option<hash_scanner::SignatureDb>,
    pub yara_rules: Option<yara_engine::YaraEngine>,
    prep_args: PrepArgs,
    scan_args: ScanArgs,
}

impl ScanEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn prepare(&mut self, args: &PrepArgs) -> Result<(i32, i32), String> {
        self.prep_args = args.clone();

        // Self::init_thread_pool(&args.threads)?;

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

        let _ = Self::init_thread_pool(&args.threads);

        log::info!("Rayon active threads: {}", rayon::current_num_threads());
        if path.is_file() {
            let results = self.scan_file(path);
            report.add(results);
        } else if path.is_dir() {
            let results = self.scan_dir(path);
            report.add(results);
        }

        report
    }

    fn init_thread_pool(threads: &str) -> Result<(), String> {
        let num_threads = match threads.trim().to_lowercase().as_str() {
            "auto" => None,
            "conservative" => {
                let cores = std::thread::available_parallelism()
                    .map(|n| n.get())
                    .unwrap_or(2);
                Some((cores / 2).max(1))
            }
            n => Some(n.parse::<usize>().map_err(|_| {
                format!(
                    "Invalid --threads value: '{}'. Use a number, 'auto', or 'conservative'",
                    n
                )
            })?),
        };

        THREAD_POOL_INIT.get_or_init(|| {
            if let Some(n) = num_threads {
                rayon::ThreadPoolBuilder::new()
                    .num_threads(n)
                    .build_global()
                    .unwrap_or_else(|e| log::warn!("Thread pool init failed: {}", e));
                log::info!("Thread pool initialized with {} threads", n);
            } else {
                log::info!("Thread pool using auto (all available cores)");
            }
        });

        Ok(())
    }
}
