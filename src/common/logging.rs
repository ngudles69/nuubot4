//! Shared logging support.

use std::fmt::Display;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
    TerminalMode, WriteLogger,
};

use crate::Result;
use crate::config::load_config;

static PROCESS_LOGGER: OnceLock<Logger> = OnceLock::new();

/// Write through the installed process logger.
#[derive(Clone, Debug)]
pub struct Logger(());

impl Logger {
    /// Install one owner log, the shared error log, and an optional console mirror.
    fn install(owner_path: Option<PathBuf>, log_dir: &Path, console: bool) -> Result<Self> {
        fs::create_dir_all(log_dir)
            .map_err(|error| format!("create log directory {}: {error}", log_dir.display()))?;

        let mut config = ConfigBuilder::new();
        config
            .set_target_level(LevelFilter::Error)
            .set_time_format_rfc3339();
        let _ = config.set_time_offset_to_local();
        let config = config.build();

        let mut outputs: Vec<Box<dyn SharedLogger>> = Vec::new();
        if let Some(path) = owner_path {
            let file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|error| format!("open log file {}: {error}", path.display()))?;
            outputs.push(WriteLogger::new(LevelFilter::Info, config.clone(), file));
        }
        let errors = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_dir.join("errors.log"))
            .map_err(|error| format!("open errors.log: {error}"))?;
        outputs.push(WriteLogger::new(LevelFilter::Error, config.clone(), errors));
        if console {
            outputs.push(TermLogger::new(
                LevelFilter::Info,
                config,
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ));
        }
        CombinedLogger::init(outputs).map_err(|error| format!("start logger: {error}"))?;
        let logger = Self(());
        PROCESS_LOGGER
            .set(logger.clone())
            .map_err(|_| "process logger already installed".to_owned())?;
        Ok(logger)
    }

    /// Write one informational lifecycle record.
    pub fn info(&self, component: &str, event: impl Display) {
        log::info!(target: component, "{event}");
    }

    /// Write one fatal lifecycle record.
    pub fn error(&self, component: &str, event: impl Display) {
        log::error!(target: component, "{event}");
    }
}

/// Create the one process logger from an optional file name.
pub fn logger(name: Option<&str>) -> Result<Logger> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (log_dir, console) = match load_config(&root.join("config.toml")) {
        Ok(config) => (root.join(config.paths.logs), config.logging.console),
        Err(_) => (root.join("workspace/logs"), true),
    };
    let owner_path = name.map(|name| log_dir.join(format!("{name}.log")));
    Logger::install(owner_path, &log_dir, console)
}

/// Build the shared Bot log filename.
pub fn bot_log_name(sweep_id: u64, bot_id: u64) -> String {
    format!("nuubot4-bot-{sweep_id}-{bot_id}")
}

/// Log the program's final error once.
pub fn log_error(program: &str, error: impl Display) {
    let error = error.to_string();
    if let Some(log) = PROCESS_LOGGER.get() {
        log.error(program, error);
        return;
    }

    match logger(None) {
        Ok(log) => log.error(program, error),
        Err(log_error) => eprintln!("logging failed: {log_error}; {error}"),
    }
}
