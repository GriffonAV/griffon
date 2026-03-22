use crate::file_context::{self, FileType, ScanStage};
use log::error;
use regex::Regex;
use std::path::Path;

pub struct RuleMetadata {
    pub file_type: FileType,
    pub stage: ScanStage,
}

pub struct RuleParser {
    re_file_type: Regex,
    re_stage: Regex,
}

impl RuleParser {
    pub fn new() -> Self {
        Self {
            re_file_type: Regex::new(r#"file_type\s*=\s*"([^"]+)""#).unwrap(),
            re_stage: Regex::new(r#"stage\s*=\s*"([^"]+)""#).unwrap(),
        }
    }

    pub fn parse_metadata(&self, contents: &str, path: &Path) -> RuleMetadata {
        let file_type = self.extract_file_type(contents, path);
        let stage = self.extract_stage(contents, path);

        RuleMetadata { file_type, stage }
    }

    fn extract_file_type(&self, contents: &str, path: &Path) -> FileType {
        self.re_file_type
            .captures(contents)
            .and_then(|c| self.parse_file_type_value(&c[1], path))
            .unwrap_or(FileType::GenericBinary)
    }

    fn parse_file_type_value(&self, value: &str, path: &Path) -> Option<FileType> {
        match value.to_lowercase().as_str() {
            "elf" => Some(FileType::Executable(file_context::ExecutableType::Elf)),
            "script" => Some(FileType::Script(file_context::ScriptType::Other)),
            "archive" => Some(FileType::Archive(file_context::ArchiveType::Unknown)),
            "generic" | "genericbinary" => Some(FileType::GenericBinary),
            other => {
                error!(
                    "Unrecognized file_type metadata '{}' in {} - assigning Generic",
                    other,
                    path.display()
                );
                Some(FileType::GenericBinary)
            }
        }
    }

    fn extract_stage(&self, contents: &str, path: &Path) -> ScanStage {
        self.re_stage
            .captures(contents)
            .and_then(|c| self.parse_stage_value(&c[1], path))
            .unwrap_or(ScanStage::Pre)
    }

    fn parse_stage_value(&self, value: &str, path: &Path) -> Option<ScanStage> {
        match value.to_lowercase().as_str() {
            "pre" => Some(ScanStage::Pre),
            "post" => Some(ScanStage::Post),
            other => {
                error!(
                    "Unrecognized stage '{}' in {} - defaulting to Pre",
                    other,
                    path.display()
                );
                Some(ScanStage::Pre)
            }
        }
    }
}

impl Default for RuleParser {
    fn default() -> Self {
        Self::new()
    }
}
