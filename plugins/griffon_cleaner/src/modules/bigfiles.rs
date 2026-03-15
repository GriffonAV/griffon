use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};

pub struct BigfilesScanner;

impl BigfilesScanner {
    pub fn new() -> Self {
        Self
    }
}

impl CleanerModule for BigfilesScanner {
    fn id(&self) -> &'static str {
        " bigfiles"
    }

    fn description(&self) -> &'static str {
        "Scan and clean large unused files."
    }

    fn run(&self, _ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        Ok(ModuleReport::empty(self.id()))
    }
}
