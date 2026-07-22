//! Datastore boundary models.

use std::path::PathBuf;

use chrono::NaiveDate;
use serde::Deserialize;

/// Describe one validated Bot replay request.
#[derive(Clone, Debug)]
pub struct BotSpec {
    pub symbol: String,
    pub ticks_path: PathBuf,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[derive(Deserialize)]
pub(super) struct StoredBotConfig {
    pub(super) general: StoredGeneral,
    pub(super) data: StoredData,
    pub(super) date_range: StoredDateRange,
}

#[derive(Deserialize)]
pub(super) struct StoredGeneral {
    pub(super) symbol: String,
}

#[derive(Deserialize)]
pub(super) struct StoredData {
    pub(super) ticks: PathBuf,
}

#[derive(Deserialize)]
pub(super) struct StoredDateRange {
    pub(super) start: String,
    pub(super) end: String,
}
