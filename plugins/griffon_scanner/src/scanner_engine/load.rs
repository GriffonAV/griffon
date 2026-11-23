use std::path::PathBuf;

use crate::paths::default_rules_dir;
use crate::scanner_engine::{ScanEngine, hash_scanner, scanargs::PrepArgs, yara_engine};

impl ScanEngine {
    pub fn load_hash_db(&mut self, args: &PrepArgs) -> anyhow::Result<()> {
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

    pub fn load_yara_rules(&mut self, args: &PrepArgs) -> anyhow::Result<()> {
        let path = args
            .yara_rules
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_else(default_rules_dir);
        log::info!("Loading YARA rules from {}", path.display());
        let path_str = path
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("rules path is not valid UTF-8: {}", path.display()))?;
        let rules = yara_engine::YaraEngine::load_from_dir(path_str)?;
        // An engine without rules would report every file as clean.
        anyhow::ensure!(
            rules.rule_count() > 0,
            "no YARA rules found in {}; update the signature database",
            path.display()
        );
        self.yara_rules = Some(rules);
        Ok(())
    }
}
