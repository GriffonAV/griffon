use super::rule_key::RuleKey;
use crate::file_context::{FileType, ScanStage};
use std::collections::HashMap;
use yara_x::Rules;

#[derive(Debug)]
pub struct RuleIndex {
    pub rules: HashMap<RuleKey, Rules>,
}

impl RuleIndex {
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: RuleKey, rules: Rules) {
        self.rules.insert(key, rules);
    }

    pub fn select_rules(&self, ft: FileType, stage: ScanStage) -> Option<&Rules> {
        let key = RuleKey::new(ft, stage);

        self.rules
            .get(&key)
            .or_else(|| self.rules.get(&RuleKey::generic(stage)))
    }
}

impl Default for RuleIndex {
    fn default() -> Self {
        Self::new()
    }
}
