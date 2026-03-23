use anyhow::{Context, Result};
use log::{debug, error};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use yara_x::Compiler;

use super::rule_index::RuleIndex;
use super::rule_key::RuleKey;
use super::rule_parser::RuleParser;

pub struct RuleLoader {
    parser: RuleParser,
}

impl RuleLoader {
    pub fn new() -> Self {
        Self {
            parser: RuleParser::new(),
        }
    }

    pub fn load_from_directory<P: AsRef<Path>>(&self, dir: P) -> Result<RuleIndex> {
        debug!(
            "Loading rules from {} and extracting metadata...",
            dir.as_ref().display()
        );

        let buckets = self.collect_rules(dir.as_ref())?;
        self.compile_buckets(buckets)
    }

    fn collect_rules(&self, dir: &Path) -> Result<HashMap<RuleKey, Vec<String>>> {
        let mut buckets: HashMap<RuleKey, Vec<String>> = HashMap::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if !self.is_rule_file(path) {
                continue;
            }

            if let Some((key, contents)) = self.process_rule_file(path) {
                buckets.entry(key).or_insert_with(Vec::new).push(contents);
            }
        }

        Ok(buckets)
    }

    fn is_rule_file(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }

        path.extension()
            .map(|ext| ext == "yar" || ext == "yara")
            .unwrap_or(false)
    }

    fn process_rule_file(&self, path: &Path) -> Option<(RuleKey, String)> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("reading rule file {}", path.display()))
            .ok()?;

        let metadata = self.parser.parse_metadata(&contents, path);
        let key = RuleKey::new(metadata.file_type, metadata.stage);

        Some((key, contents))
    }

    fn compile_buckets(&self, buckets: HashMap<RuleKey, Vec<String>>) -> Result<RuleIndex> {
        let mut index = RuleIndex::new();

        for (key, sources) in buckets {
            if let Some(rules) = self.compile_bucket(key, sources) {
                index.insert(key, rules);
            }
        }

        Ok(index)
    }

    fn compile_bucket(&self, key: RuleKey, sources: Vec<String>) -> Option<yara_x::Rules> {
        let mut compiler = Compiler::new();
        let mut added = 0usize;

        for src in &sources {
            if compiler.add_source(src.as_str()).is_ok() {
                added += 1;
            } else {
                error!("Failed to add source to compiler for bucket {:?}", key);
            }
        }

        let rules = compiler.build();
        debug!("Compiled bucket {:?} with {} sources", key, added);
        Some(rules)
    }
}

impl Default for RuleLoader {
    fn default() -> Self {
        Self::new()
    }
}
