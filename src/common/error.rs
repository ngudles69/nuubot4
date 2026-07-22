//! Shared error types.

use std::path::PathBuf;

/// Represent one admitted Nuubot4 failure.
#[derive(Debug, thiserror::Error)]
pub enum NuuError {
    #[error("config error: {0}")]
    Config(String),
    #[error("datastore error: {0}")]
    Datastore(String),
    #[error("lifecycle error: {0}")]
    Lifecycle(String),
    #[error("replay error: {0}")]
    Replay(String),
    #[error("CLOID error: {0}")]
    Cloid(String),
    #[error("required path not found: {0}")]
    MissingPath(PathBuf),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Csv(#[from] csv::Error),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Parquet(#[from] parquet::errors::ParquetError),
}

pub type Result<T> = std::result::Result<T, NuuError>;
