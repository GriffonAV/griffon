use crate::{CleanerModule, CleanerResult, ExecutionContext, ModuleReport};

pub struct LogsCleaner;

impl LogsCleaner {
    pub fn new() -> Self {
        Self
    }
}

impl CleanerModule for LogsCleaner {
    fn id(&self) -> &'static str {
        "logs"
    }

    fn description(&self) -> &'static str {
        "Clean system and user logs directories."
    }

    fn run(&self, _ctx: &ExecutionContext) -> CleanerResult<ModuleReport> {
        Ok(ModuleReport::empty(self.id()))
    }
}
