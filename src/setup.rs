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
    let early_log = Logger::new(root.join("workspace/logs/nuubot4-setup.log"))?;
    early_log.info("setup", "load_config")?;
    let config = load_config(&root.join("config.toml"))?;

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

    // Create Bot logger.
    let identity = BotIdentity { sweep_id, bot_id };
    let log = Logger::for_bot(&rooted(root, &config.paths.logs), identity)?;
    log.info("setup", "ready")?;

    // Return ready context.
    Ok(SetupContext {
        config,
        bot: BotSpec { ticks_path, ..bot },
        log,
    })
}

/// Resolve the configured fatal logger or the pre-config bootstrap log.
pub fn failure_logger(identity: Option<BotIdentity>) -> Result<Logger> {
    // Resolve available config.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let log_dir = match load_config(&root.join("config.toml")) {
        Ok(config) => rooted(root, &config.paths.logs),
        Err(_) => root.join("workspace/logs"),
    };

    // Select identity log.
    match identity {
        Some(identity) => Logger::for_bot(&log_dir, identity),
        None => Logger::new(log_dir.join("nuubot4-failure.log")),
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
