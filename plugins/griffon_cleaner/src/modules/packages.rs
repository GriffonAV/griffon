use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};

pub struct PackagesCleaner;

impl PackagesCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl CleanerModule for PackagesCleaner {
    fn id(&self) -> &'static str {
        "packages"
    }

    fn description(&self) -> &'static str {
        "Clean package manager caches and unused packages."
    }

    fn run(&self, _ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        Ok(ModuleReport::empty(self.id()))
    }
}
