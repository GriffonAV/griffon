use std::path::{Path, PathBuf};

use crate::scanner_engine::{ScanArgs, ScanEngine, hash_scanner, yara_engine};

pub const SDB: &str = "rules/";

fn default_rules_dir() -> PathBuf {
    if cfg!(debug_assertions) {
        PathBuf::from(SDB)
    } else {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("griffon_scanner")
            .join("rules")
    }
}

impl ScanEngine {
    pub fn load_hash_db(&mut self, args: &ScanArgs) -> anyhow::Result<()> {
        log::warn!(
            "Experimental feature: Hash-based scanning is still in early development, use only yara rules for better results"
        );
        let base_path = args
            .hash_db
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(default_rules_dir);
        let path = base_path.join("hashes.txt");
        let db = hash_scanner::SignatureDb::load(&path)?;
        self.hash_db = Some(db);
        Ok(())
    }

    pub fn load_yara_rules(&mut self, args: &ScanArgs) -> anyhow::Result<()> {
        let path = args
            .yara_rules
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(default_rules_dir);
        log::info!("Loading YARA rules from {}", path.display());
        let rules = yara_engine::YaraEngine::load_from_dir(path.to_str().unwrap_or(""))?;
        self.yara_rules = Some(rules);
        Ok(())
    }
}
