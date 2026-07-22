//! CSV replay decoder.

use std::fs::File;
use std::path::PathBuf;

use crate::Result;
use crate::market::BboTick;

use super::admit_tick;

pub(super) struct CsvTickReader {
    files: Vec<PathBuf>,
    next_file: usize,
    rows: Option<csv::StringRecordsIntoIter<File>>,
    start_us: u64,
    end_us: u64,
    last_ms: Option<u64>,
    failed: bool,
}

impl CsvTickReader {
    // Program flow

    pub(super) fn new(files: Vec<PathBuf>, start_us: u64, end_us: u64) -> Self {
        Self {
            files,
            next_file: 0,
            rows: None,
            start_us,
            end_us,
            last_ms: None,
            failed: false,
        }
    }
}

impl Iterator for CsvTickReader {
    type Item = Result<BboTick>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.failed {
            return None;
        }
        match self.next_tick() {
            Ok(tick) => tick.map(Ok),
            Err(error) => {
                self.failed = true;
                Some(Err(error))
            }
        }
    }
}

impl CsvTickReader {
    // Domain decoding

    /// Decode the next admitted CSV row.
    fn next_tick(&mut self) -> Result<Option<BboTick>> {
        loop {
            // Open next file.
            if self.rows.is_none() && !self.open_next()? {
                return Ok(None);
            }

            // Decode next row.
            let Some(row) = self.rows.as_mut().expect("opened CSV").next() else {
                self.rows = None;
                continue;
            };
            let row = row.map_err(|error| format!("read CSV row: {error}"))?;
            if row.len() != 2 {
                return Err("CSV row must contain exactly close_time_us,close".into());
            }
            let close_time_us = row[0]
                .parse::<u64>()
                .map_err(|error| format!("invalid close_time_us: {error}"))?;
            let price = row[1]
                .parse::<f64>()
                .map_err(|error| format!("invalid close: {error}"))?;
            if close_time_us < self.start_us || close_time_us >= self.end_us {
                continue;
            }
            return Ok(Some(admit_tick(&mut self.last_ms, close_time_us, price)?));
        }
    }

    /// Open and validate the next CSV file.
    fn open_next(&mut self) -> Result<bool> {
        let Some(path) = self.files.get(self.next_file) else {
            return Ok(false);
        };
        self.next_file += 1;
        let mut reader = csv::ReaderBuilder::new()
            .from_path(path)
            .map_err(|error| format!("open CSV {}: {error}", path.display()))?;
        let headers = reader
            .headers()
            .map_err(|error| format!("read CSV header {}: {error}", path.display()))?;
        if headers.iter().collect::<Vec<_>>() != ["close_time_us", "close"] {
            return Err(format!("unexpected CSV header: {}", path.display()));
        }
        self.rows = Some(reader.into_records());
        Ok(true)
    }
}
