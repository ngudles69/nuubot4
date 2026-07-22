use std::path::PathBuf;

use chrono::{Datelike, NaiveDate};

use crate::config::{BtRunnerConfig, LoaderKind};
use crate::datastore::BotSpec;
use crate::market::BboTick;
use crate::{NuuError, Result};

mod csv;
mod parquet;

use csv::CsvTickReader;
use parquet::ParquetTickReader;

/// Stream trusted BBO values from one selected external encoding.
pub struct TickReader {
    source: TickReaderKind,
    pending: Option<BboTick>,
}

enum TickReaderKind {
    Csv(CsvTickReader),
    Parquet(ParquetTickReader),
}

/// Describe the exact replay evidence expected from one Bot range.
#[derive(Clone, Copy, Debug)]
pub struct ReplayExpectation {
    pub ticks: u64,
    pub callbacks: u64,
    pub first_ts_ms: u64,
    pub last_ts_ms: u64,
}

/// Carry one visible calendar-week replay boundary.
#[derive(Clone, Copy, Debug)]
pub struct ReplayWindow {
    pub start: NaiveDate,
    pub end: NaiveDate,
    end_ms: u64,
    final_window: bool,
}

/// Report each calendar-week boundary once during streaming replay.
pub struct ReplayWindows {
    windows: std::vec::IntoIter<ReplayWindow>,
}

/// Validate replay files and return one streaming iterator.
pub fn load_ticks(bot: &BotSpec, config: &BtRunnerConfig) -> Result<TickReader> {
    // Build exact file list.
    let files = replay_files(bot, config.loader)?;
    for path in &files {
        if !path.is_file() {
            return Err(NuuError::MissingPath(path.clone()));
        }
    }
    let start_us = date_us(bot.start)?;
    let end_us = date_us(bot.end)?;

    // Create selected reader.
    Ok(TickReader {
        source: match config.loader {
            LoaderKind::Csv => TickReaderKind::Csv(CsvTickReader::new(files, start_us, end_us)),
            LoaderKind::Parquet => TickReaderKind::Parquet(ParquetTickReader::new(
                files,
                start_us,
                end_us,
                config.parquet_batch_size,
            )),
        },
        pending: None,
    })
}

/// Calculate exact one-second replay evidence for a configured range.
pub fn replay_expectation(bot: &BotSpec, timer_interval_ms: u64) -> Result<ReplayExpectation> {
    // Calculate time range.
    let start_ms = date_us(bot.start)? / 1000;
    let end_ms = date_us(bot.end)? / 1000;
    let duration_ms = end_ms - start_ms;
    if duration_ms % 1000 != 0 {
        return Err(NuuError::Replay("replay range is not whole seconds".into()));
    }

    // Return exact counts.
    Ok(ReplayExpectation {
        ticks: duration_ms / 1000,
        callbacks: duration_ms.div_ceil(timer_interval_ms),
        first_ts_ms: start_ms + 1000,
        last_ts_ms: end_ms,
    })
}

impl ReplayWindows {
    /// Split one admitted Bot range at Monday boundaries.
    pub fn new(bot: &BotSpec) -> Result<Self> {
        let mut start = bot.start;
        let mut windows = Vec::new();
        while start < bot.end {
            let days = 7 - i64::from(start.weekday().num_days_from_monday());
            let monday = start + chrono::Duration::days(days);
            let end = monday.min(bot.end);
            windows.push(ReplayWindow {
                start,
                end,
                end_ms: date_us(end)? / 1000,
                final_window: end == bot.end,
            });
            start = end;
        }
        Ok(Self {
            windows: windows.into_iter(),
        })
    }
}

impl Iterator for ReplayWindows {
    type Item = ReplayWindow;

    fn next(&mut self) -> Option<Self::Item> {
        self.windows.next()
    }
}

impl TickReader {
    /// Load one owned calendar-week replay window.
    pub fn load_window(&mut self, window: ReplayWindow) -> Result<Vec<BboTick>> {
        let mut ticks = Vec::new();
        if let Some(tick) = self.pending.take() {
            ticks.push(tick);
        }
        loop {
            let next = match &mut self.source {
                TickReaderKind::Csv(reader) => reader.next(),
                TickReaderKind::Parquet(reader) => reader.next(),
            };
            let Some(tick) = next.transpose()? else {
                break;
            };
            if tick.ts_ms() > window.end_ms
                || (!window.final_window && tick.ts_ms() == window.end_ms)
            {
                self.pending = Some(tick);
                break;
            }
            ticks.push(tick);
        }
        Ok(ticks)
    }
}

/// Convert one external row into trusted internal time.
pub(super) fn admit_tick(
    last_ms: &mut Option<u64>,
    close_time_us: u64,
    price: f64,
) -> Result<BboTick> {
    // Validate close boundary.
    let fraction_us = close_time_us % 1_000_000;
    if !(999_000..=999_999).contains(&fraction_us) {
        return Err(NuuError::Replay(format!(
            "1s close_time_us must end in 999000..=999999, received {close_time_us}"
        )));
    }

    // Normalize close time.
    let ts_ms = close_time_us
        .checked_div(1_000_000)
        .and_then(|second| second.checked_add(1))
        .and_then(|second| second.checked_mul(1000))
        .ok_or_else(|| NuuError::Replay("close_time_us normalization overflow".into()))?;

    // Enforce one-second order.
    if let Some(last) = *last_ms {
        let expected = last
            .checked_add(1000)
            .ok_or_else(|| NuuError::Replay("1s sequence overflow".into()))?;
        if ts_ms != expected {
            return Err(NuuError::Replay(format!(
                "1s sequence expected {expected}, received {ts_ms}"
            )));
        }
    }

    // Admit trusted tick.
    let tick = BboTick::admit(ts_ms, price)?;
    *last_ms = Some(ts_ms);
    Ok(tick)
}

/// Build one exact calendar-month file list.
fn replay_files(bot: &BotSpec, loader: LoaderKind) -> Result<Vec<PathBuf>> {
    let market = bot
        .ticks_path
        .parent()
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .ok_or_else(|| NuuError::Replay("tick path has no market parent".into()))?;
    let extension = match loader {
        LoaderKind::Csv => "csv",
        LoaderKind::Parquet => "parquet",
    };
    let mut month = bot.start.with_day(1).expect("valid first day");
    let mut files = Vec::new();
    while month < bot.end {
        files.push(bot.ticks_path.join(format!(
            "{market}-1s-{:04}-{:02}.{extension}",
            month.year(),
            month.month()
        )));
        month = if month.month() == 12 {
            NaiveDate::from_ymd_opt(month.year() + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(month.year(), month.month() + 1, 1)
        }
        .expect("valid next month");
    }
    Ok(files)
}

/// Convert an admitted date boundary to Unix microseconds.
fn date_us(date: NaiveDate) -> Result<u64> {
    u64::try_from(
        date.and_hms_opt(0, 0, 0)
            .expect("valid midnight")
            .and_utc()
            .timestamp_micros(),
    )
    .map_err(|_| NuuError::Replay("date precedes Unix epoch".into()))
}

#[cfg(test)]
mod tests {
    use super::admit_tick;

    #[test]
    fn admits_mixed_precision_across_year_boundary() {
        let mut last_ms = None;

        let old = admit_tick(&mut last_ms, 1_735_689_599_999_000, 1.0).unwrap();
        let new = admit_tick(&mut last_ms, 1_735_689_600_999_999, 1.0).unwrap();

        assert_eq!(old.ts_ms(), 1_735_689_600_000);
        assert_eq!(new.ts_ms(), 1_735_689_601_000);
    }

    #[test]
    fn admits_ordinary_consecutive_ticks() {
        let mut last_ms = None;

        admit_tick(&mut last_ms, 1_000_999_999, 1.0).unwrap();
        let tick = admit_tick(&mut last_ms, 1_001_999_999, 1.0).unwrap();

        assert_eq!(tick.ts_ms(), 1_002_000);
    }

    #[test]
    fn rejects_duplicate_and_gap() {
        let mut last_ms = None;

        admit_tick(&mut last_ms, 1_000_999_999, 1.0).unwrap();
        assert!(admit_tick(&mut last_ms, 1_000_999_000, 1.0).is_err());
        assert!(admit_tick(&mut last_ms, 1_002_999_999, 1.0).is_err());
    }

    #[test]
    fn rejects_invalid_fraction() {
        assert!(admit_tick(&mut None, 1_000_998_999, 1.0).is_err());
    }

    #[test]
    fn rejects_sequence_overflow() {
        assert!(admit_tick(&mut Some(u64::MAX), 1_000_999_999, 1.0).is_err());
    }
}
