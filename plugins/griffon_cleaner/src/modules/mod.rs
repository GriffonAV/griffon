// src/modules/mod.rs

// Déclaration des sous-modules
pub mod bigfiles;
pub mod cache;
pub mod docker;
pub mod logs;
pub mod packages;

use crate::{CleanerResult, ExecutionContext, ModuleReport};

/// Trait commun à tous les sous-plugins du Cleaner.
pub trait CleanerModule: Send + Sync {
    fn id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, ctx: &ExecutionContext) -> CleanerResult<ModuleReport>;

    fn dry_run(&self, ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        let mut clone = ctx.clone();
        clone.dry_run = true;
        self.run(&clone)
    }
}
