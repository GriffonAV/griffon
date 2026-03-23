use anyhow::Result;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use yara_x::Rules;

use super::rule_index::RuleIndex;
use super::rule_loader::RuleLoader;
use crate::file_context::{FileType, ScanStage};

pub struct RulesEngineConfig {
    pub rules_dir: Option<PathBuf>,
}

pub struct RulesEngine {
    pub rule_index: Arc<RuleIndex>,
    pub config: RulesEngineConfig,
}

impl RulesEngine {
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let loader = RuleLoader::new();
        let idx = loader.load_from_directory(dir.as_ref())?;

        Ok(Self {
            rule_index: Arc::new(idx),
            config: RulesEngineConfig {
                rules_dir: Some(dir.as_ref().to_path_buf()),
            },
        })
    }

    pub fn select_rules(&self, ft: FileType, stage: ScanStage) -> Option<&Rules> {
        self.rule_index.select_rules(ft, stage)
    }
}
