//! Shared logging support.

use std::fs::{self, OpenOptions};
use std::io::{IsTerminal, Write};
use std::path::{Path, PathBuf};

use chrono::Local;

use crate::Result;

/// Identify every object owned by one Bot run.
#[derive(Clone, Copy, Debug)]
pub struct BotIdentity {
    pub sweep_id: u64,
    pub bot_id: u64,
}

/// Append ordered records to one process-owned log.
#[derive(Clone, Debug)]
pub struct Logger {
    path: PathBuf,
}

impl Logger {
    /// Create one logger and its parent directory.
    pub fn new(path: PathBuf) -> Result<Self> {
        // Prepare log path.
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(Self { path })
    }

    /// Create the shared Bot identity logger.
    pub fn for_bot(log_dir: &Path, identity: BotIdentity) -> Result<Self> {
        Self::new(log_dir.join(format!(
            "nuubot4-bot-{}-{}.log",
            identity.sweep_id, identity.bot_id
        )))
    }

    /// Write one informational lifecycle record.
    pub fn info(&self, component: &str, event: &str) -> Result<()> {
        self.write(" INFO", component, event)
    }

    /// Write one fatal lifecycle record.
    pub fn error(&self, component: &str, event: &str) -> Result<()> {
        self.write("ERROR", component, event)
    }

    /// Append one formatted record.
    fn write(&self, level: &str, component: &str, event: &str) -> Result<()> {
        // Format one record.
        let line = format!(
            "{} [{level:>5}] component={} event={}\n",
            Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
            component,
            event
        );

        // Append process log.
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)?
            .write_all(line.as_bytes())?;
        if std::io::stderr().is_terminal() {
            eprint!("{line}");
        }
        Ok(())
    }
}
