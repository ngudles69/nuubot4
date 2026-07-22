//! Process setup.

use std::path::{Path, PathBuf};

use crate::Result;
use crate::common::logging::Logger;
use crate::config::{AppConfig, load_config};
use crate::datastore::{BotSpec, SweepStore};

/// Hold fully admitted infrastructure for one BtRunner.
pub struct SetupContext {
    pub config: AppConfig,
    pub bot: BotSpec,
}

/// Build and validate common infrastructure before runner composition.
pub fn nuubot_setup(log: &Logger, sweep_id: u64, bot_id: u64) -> Result<SetupContext> {
    // Load root config.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = load_config(&root.join("config.toml"))?;

    log.info("setup", "load_config");

    // Resolve owned paths.
    let database = rooted(root, &config.paths.sweep_database);
    let store = SweepStore::open(&database)?;
    let bot = store.load_bot(sweep_id, bot_id)?;

    // Enforce data boundary.
    let shared_data = config.paths.shared_data.canonicalize().map_err(|error| {
        format!(
            "resolve shared_data {}: {error}",
            config.paths.shared_data.display()
        )
    })?;
    let ticks_path = bot.ticks_path.canonicalize().map_err(|error| {
        format!(
            "resolve Bot ticks path {}: {error}",
            bot.ticks_path.display()
        )
    })?;
    if !ticks_path.starts_with(&shared_data) {
        return Err(format!(
            "Bot ticks path is outside shared_data: {}",
            ticks_path.display()
        ));
    }

    log.info("setup", "ready");

    // Return ready context.
    Ok(SetupContext {
        config,
        bot: BotSpec { ticks_path, ..bot },
    })
}

/// Resolve one repository-relative configured path.
fn rooted(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}
