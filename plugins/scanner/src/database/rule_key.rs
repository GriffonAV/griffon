use crate::file_context::{FileType, ScanStage};

#[derive(Hash, Eq, PartialEq, Debug, Clone, Copy)]
pub struct RuleKey {
    pub file_type: FileType,
    pub stage: ScanStage,
    //pub os: Option<OS>, // future expansion
    //pub arch: Option<Arch>,
    //pub scan_context: Option<ScanContext>,
}

impl RuleKey {
    pub fn new(file_type: FileType, stage: ScanStage) -> Self {
        Self { file_type, stage }
    }

    pub fn generic(stage: ScanStage) -> Self {
        Self {
            file_type: FileType::GenericBinary,
            stage,
        }
    }
}
