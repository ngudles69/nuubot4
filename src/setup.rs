//! Process setup.

use std::path::{Path, PathBuf};

use crate::common::logging::{BotIdentity, Logger};
use crate::config::{AppConfig, load_config};
use crate::datastore::{BotSpec, SweepStore};
use crate::{NuuError, Result};

/// Hold fully admitted infrastructure for one BtRunner.
pub struct SetupContext {
    pub config: AppConfig,
    pub bot: BotSpec,
    pub log: Logger,
}

/// Build and validate common infrastructure before runner composition.
pub fn nuubot_setup(sweep_id: u64, bot_id: u64) -> Result<SetupContext> {
    // Load root config.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = load_config(&root.join("config.toml"))?;

    // Create Bot logger.
    let identity = BotIdentity { sweep_id, bot_id };
    let log = Logger::for_bot(
        &rooted(root, &config.paths.logs),
        identity,
        config.logging.console,
    )?;
    log.info("setup", "load_config")?;

    // Resolve owned paths.
    let database = rooted(root, &config.paths.sweep_database);
    let store = SweepStore::open(&database)?;
    let bot = store.load_bot(sweep_id, bot_id)?;

    // Enforce data boundary.
    let shared_data = config.paths.shared_data.canonicalize()?;
    let ticks_path = bot.ticks_path.canonicalize()?;
    if !ticks_path.starts_with(&shared_data) {
        return Err(NuuError::Config(format!(
            "Bot ticks path is outside shared_data: {}",
            ticks_path.display()
        )));
    }

    log.info("setup", "ready")?;

    // Return ready context.
    Ok(SetupContext {
        config,
        bot: BotSpec { ticks_path, ..bot },
        log,
    })
}

/// Resolve the active process logger or initialize its fallback.
pub fn program_logger(identity: Option<BotIdentity>) -> Result<Logger> {
    if let Some(log) = Logger::current() {
        return Ok(log);
    }

    // Resolve available config.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let (log_dir, console) = match load_config(&root.join("config.toml")) {
        Ok(config) => (rooted(root, &config.paths.logs), config.logging.console),
        Err(_) => (root.join("workspace/logs"), true),
    };

    // Select identity log.
    match identity {
        Some(identity) => Logger::for_bot(&log_dir, identity, console),
        None => Logger::new(log_dir.join("nuubot4-failure.log"), console),
    }
}

/// Resolve one repository-relative configured path.
fn rooted(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
