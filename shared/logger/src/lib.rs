use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "DEBUG"),
            LogLevel::Info => write!(f, "INFO"),
            LogLevel::Warn => write!(f, "WARN"),
            LogLevel::Error => write!(f, "ERROR"),
        }
    }
}

pub struct Logger {
    min_level: LogLevel,
    prefix: &'static str,
}

impl Logger {
    pub const fn new(prefix: &'static str, min_level: LogLevel) -> Self {
        Self { min_level, prefix }
    }

    pub fn set_level(&mut self, level: LogLevel) {
        self.min_level = level;
    }

    pub fn level(&self) -> LogLevel {
        self.min_level
    }

    pub fn enabled(&self, level: LogLevel) -> bool {
        level >= self.min_level
    }

    pub fn log(&self, level: LogLevel, msg: impl AsRef<str>) {
        if !self.enabled(level) {
            return;
        }

        let msg = msg.as_ref();

        match level {
            LogLevel::Error => eprintln!("[{}]({}) {}", self.prefix, level, msg),
            _ => println!("[{}]({}) {}", self.prefix, level, msg),
        }
    }

    pub fn debug(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Debug, msg);
    }

    pub fn info(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Info, msg);
    }

    pub fn warn(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Warn, msg);
    }

    pub fn error(&self, msg: impl AsRef<str>) {
        self.log(LogLevel::Error, msg);
    }
}