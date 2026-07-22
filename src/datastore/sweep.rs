//! Read-only Sweep datastore implementation.

use std::path::Path;

use chrono::NaiveDate;
use rusqlite::{Connection, OpenFlags, params};

use super::models::{BotSpec, StoredBotConfig};
use crate::{NuuError, Result};

/// Own one read-only connection to the Nuubot4 Sweep copy.
pub struct SweepStore {
    connection: Connection,
}

impl SweepStore {
    /// Open one existing Sweep database without write authority.
    pub fn open(path: &Path) -> Result<Self> {
        // Require owned copy.
        if !path.is_file() {
            return Err(NuuError::MissingPath(path.to_path_buf()));
        }
        let connection = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        Ok(Self { connection })
    }

    /// Load one Bot replay specification by exact identity.
    pub fn load_bot(&self, sweep_id: u64, bot_id: u64) -> Result<BotSpec> {
        // Read stored config.
        let json: String = self
            .connection
            .query_row(
                "SELECT config_json FROM bot WHERE sweep_id = ?1 AND bot_id = ?2",
                params![sweep_id, bot_id],
                |row| row.get(0),
            )
            .map_err(|error| {
                NuuError::Datastore(format!("Bot sweep_id={sweep_id} bot_id={bot_id}: {error}"))
            })?;
        let stored: StoredBotConfig = serde_json::from_str(&json).map_err(|error| {
            NuuError::Datastore(format!(
                "invalid Bot config sweep_id={sweep_id} bot_id={bot_id}: {error}"
            ))
        })?;

        // Validate Bot fields.
        let symbol = stored.general.symbol.trim().to_owned();
        if symbol.is_empty() || stored.data.ticks.as_os_str().is_empty() {
            return Err(NuuError::Datastore(
                "Bot symbol or tick path is empty".into(),
            ));
        }
        let start = NaiveDate::parse_from_str(&stored.date_range.start, "%Y-%m-%d")
            .map_err(|error| NuuError::Datastore(format!("invalid start date: {error}")))?;
        let end = NaiveDate::parse_from_str(&stored.date_range.end, "%Y-%m-%d")
            .map_err(|error| NuuError::Datastore(format!("invalid end date: {error}")))?;
        if start >= end {
            return Err(NuuError::Datastore(
                "Bot start date must precede end date".into(),
            ));
        }
        Ok(BotSpec {
            symbol,
            ticks_path: stored.data.ticks,
            start,
            end,
        })
    }
}
