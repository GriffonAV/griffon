use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

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
    file_path: Option<&'static str>,
}

impl Logger {
    pub const fn new(
        prefix: &'static str,
        min_level: LogLevel,
        file_path: Option<&'static str>,
    ) -> Self {
        Self {
            min_level,
            prefix,
            file_path,
        }
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
        let formatted = format!("[{}]({}) {}", self.prefix, level, msg);

        match level {
            LogLevel::Error => eprintln!("{}", formatted),
            _ => println!("{}", formatted),
        }

        if let Some(path) = self.file_path {
            if let Some(parent) = Path::new(path).parent() {
                let _ = fs::create_dir_all(parent);
            }

            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
                let _ = writeln!(file, "{}", formatted);
            }
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
