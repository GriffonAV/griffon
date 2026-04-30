use std::path::Path;

use crate::scanner_engine::{ScanArgs, ScanEngine, hash_scanner, yara_engine};

impl ScanEngine {
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
}
