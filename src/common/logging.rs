//! Shared logging support.

use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

use simplelog::{
    ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, SharedLogger, TermLogger,
    TerminalMode, WriteLogger,
};

use crate::{NuuError, Result};

/// Identify every object owned by one Bot run.
#[derive(Clone, Copy, Debug)]
pub struct BotIdentity {
    pub sweep_id: u64,
    pub bot_id: u64,
}

/// Write through the installed process logger.
#[derive(Clone, Debug)]
pub struct Logger(());

impl Logger {
    /// Install one file logger with an optional console mirror.
    pub fn new(path: PathBuf, console: bool) -> Result<Self> {
        // Prepare log path.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        // Install outputs.
        let mut config = ConfigBuilder::new();
        config
            .set_target_level(LevelFilter::Error)
            .set_time_format_rfc3339();
        let _ = config.set_time_offset_to_local();
        let config = config.build();
        let mut outputs: Vec<Box<dyn SharedLogger>> =
            vec![WriteLogger::new(LevelFilter::Info, config.clone(), file)];
        if console {
            outputs.push(TermLogger::new(
                LevelFilter::Info,
                config,
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ));
        }
        CombinedLogger::init(outputs)
            .map_err(|error| NuuError::Config(format!("logging init failed: {error}")))?;
        Ok(Self(()))
    }

    /// Create the shared Bot identity logger.
    pub fn for_bot(log_dir: &Path, identity: BotIdentity, console: bool) -> Result<Self> {
        Self::new(
            log_dir.join(format!(
                "nuubot4-bot-{}-{}.log",
                identity.sweep_id, identity.bot_id
            )),
            console,
        )
    }

    /// Return the installed process logger.
    pub fn current() -> Option<Self> {
        log::log_enabled!(log::Level::Error).then_some(Self(()))
    }

    /// Write one informational lifecycle record.
    pub fn info(&self, component: &str, event: &str) -> Result<()> {
        log::info!(target: component, "{event}");
        Ok(())
    }

    /// Write one fatal lifecycle record.
    pub fn error(&self, component: &str, event: &str) -> Result<()> {
        log::error!(target: component, "{event}");
        Ok(())
    }
}
