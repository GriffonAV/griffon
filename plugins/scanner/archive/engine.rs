use log::{Level, debug, error, info, log_enabled};
use std::collections::HashMap;

use crate::file_context::{self, ArchiveType, ExecutableType, FileType, ScanStage, ScriptType};
use yara_x::Rules;

use anyhow::{Context, Result};
use regex::{Regex, bytes::Regex as ByteRegex};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use walkdir::WalkDir;
use yara_x::Compiler;

mod rule_key;
use rule_key::RuleKey;

mod rule_index;
use rule_index::RuleIndex;

mod rule_parser;

pub fn load_rule_index<P: AsRef<Path>>(dir: P) -> Result<RuleIndex> {
    let mut buckets: HashMap<RuleKey, Vec<String>> = HashMap::new();

    // ! important todo : split rules when multiple rules in one file, assign to multiple buckets
    let re_file_type = Regex::new(r#"file_type\s*=\s*\"([^\"]+)\""#).unwrap();
    let re_stage = Regex::new(r#"stage\s*=\s*\"([^\"]+)\""#).unwrap();

    debug!(
        "Get all rules in {} and extract metadata...",
        dir.as_ref().display()
    );

    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        if let Some(ext) = path.extension() {
            if ext != "yar" && ext != "yara" {
                continue;
            }
        } else {
            continue;
        }

        // ! for now, only the file type and stage of the first rule in the file is considered
        // ! also, those metadata are not always present, so we need to add new sorting metadata
        let contents = fs::read_to_string(path)
            .with_context(|| format!("reading rule file {}", path.display()))?;
        let ft = if let Some(c) = re_file_type.captures(&contents) {
            match &c[1].to_lowercase()[..] {
                "elf" => FileType::Executable(file_context::ExecutableType::Elf),
                "script" => FileType::Script(file_context::ScriptType::Other),
                "archive" => FileType::Archive(file_context::ArchiveType::Unknown),
                "generic" | "genericbinary" => FileType::GenericBinary,
                other => {
                    debug!(
                        "Unrecognized file_type metadata '{}' in {} - assigning Generic",
                        other,
                        path.display()
                    );
                    FileType::GenericBinary
                }
            }
        } else {
            FileType::GenericBinary
        };

        let stage = if let Some(c) = re_stage.captures(&contents) {
            match &c[1].to_lowercase()[..] {
                "pre" => ScanStage::Pre,
                "post" => ScanStage::Post,
                other => {
                    debug!(
                        "Unrecognized stage '{}' in {} - defaulting to Pre",
                        other,
                        path.display()
                    );
                    ScanStage::Pre
                }
            }
        } else {
            ScanStage::Pre
        };
        let key = RuleKey {
            file_type: ft,
            stage: stage,
        };
        buckets.entry(key).or_insert_with(Vec::new).push(contents);
    }

    // Compile per bucket
    let mut index = RuleIndex::new();
    for (key, sources) in buckets {
        let mut compiler = Compiler::new();
        let mut added = 0usize;
        for src in sources.iter() {
            if compiler.add_source(src.as_str()).is_ok() {
                added += 1;
            } else {
                debug!("Failed to add source to compiler for bucket {:?}", key);
            }
        }
        let rules = compiler.build();
        debug!("Compiled bucket {:?} with {} sources", key, added);
        index.rules.insert(key, rules);
    }
    Ok(index)
}

pub struct EngineConfig {
    pub rules_dir: Option<PathBuf>,
}

pub struct Engine {
    pub rule_index: Arc<RuleIndex>,
    pub config: EngineConfig,
}

impl Engine {
    pub fn from_dir<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let idx = load_rule_index(dir.as_ref())?;
        Ok(Self {
            rule_index: Arc::new(idx),
            config: EngineConfig {
                rules_dir: Some(dir.as_ref().to_path_buf()),
            },
        })
    }

    pub fn select_rules(&self, ft: FileType, stage: ScanStage) -> Option<&Rules> {
        self.rule_index.select_rules(ft, stage)
    }
}

// /// Convenience wrapper that keeps old behavior: compile all rules into a single `Rules`.
// pub fn load_yara_rules<P: AsRef<Path>>(dir: P) -> Rules {
//     let mut compiler = Compiler::new();

//     // Reuse existing logic: add all sources, ignoring failures
//     for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
//         let path = entry.path();
//         if path.is_file() {
//             if let Some(ext) = path.extension() {
//                 if ext == "yar" || ext == "yara" {
//                     if let Ok(contents) = fs::read_to_string(path) {
//                         let _ = compiler.add_source(contents.as_str());
//                     }
//                 }
//             }
//         }
//     }

//     compiler.build()
// }
