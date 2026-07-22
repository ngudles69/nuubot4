//! Parquet replay decoder.

use std::fs::File;
use std::path::PathBuf;

use arrow_array::{Array, Float64Array, Int64Array, RecordBatch};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};

use crate::Result;
use crate::market::BboTick;

use super::admit_tick;

pub(super) struct ParquetTickReader {
    files: Vec<PathBuf>,
    next_file: usize,
    reader: Option<ParquetRecordBatchReader>,
    batch: Option<RecordBatch>,
    row: usize,
    batch_size: usize,
    start_us: u64,
    end_us: u64,
    last_ms: Option<u64>,
    failed: bool,
}

impl ParquetTickReader {
    // Program flow

    pub(super) fn new(files: Vec<PathBuf>, start_us: u64, end_us: u64, batch_size: usize) -> Self {
        Self {
            files,
            next_file: 0,
            reader: None,
            batch: None,
            row: 0,
            batch_size,
            start_us,
            end_us,
            last_ms: None,
            failed: false,
        }
    }
}

impl Iterator for ParquetTickReader {
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

impl ParquetTickReader {
    // Domain decoding

    /// Decode the next admitted Parquet row.
    fn next_tick(&mut self) -> Result<Option<BboTick>> {
        loop {
            // Read current batch.
            if let Some(batch) = &self.batch {
                if self.row < batch.num_rows() {
                    let row = self.row;
                    self.row += 1;
                    let times = batch
                        .column_by_name("close_time_us")
                        .and_then(|column| column.as_any().downcast_ref::<Int64Array>())
                        .ok_or_else(|| "close_time_us must be Int64".to_owned())?;
                    let prices = batch
                        .column_by_name("close")
                        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
                        .ok_or_else(|| "close must be Float64".to_owned())?;
                    if times.is_null(row) || prices.is_null(row) {
                        return Err("Parquet BBO contains null".into());
                    }
                    let close_time_us = u64::try_from(times.value(row))
                        .map_err(|_| "close_time_us must be non-negative".to_owned())?;
                    if close_time_us < self.start_us || close_time_us >= self.end_us {
                        continue;
                    }
                    return Ok(Some(admit_tick(
                        &mut self.last_ms,
                        close_time_us,
                        prices.value(row),
                    )?));
                }
                self.batch = None;
            }

            // Read next batch.
            if self.reader.is_none() && !self.open_next()? {
                return Ok(None);
            }
            match self.reader.as_mut().expect("opened Parquet").next() {
                Some(batch) => {
                    self.batch =
                        Some(batch.map_err(|error| format!("read Parquet batch: {error}"))?);
                    self.row = 0;
                }
                None => self.reader = None,
            }
        }
    }

    /// Open and project the next Parquet file.
    fn open_next(&mut self) -> Result<bool> {
        let Some(path) = self.files.get(self.next_file) else {
            return Ok(false);
        };
        self.next_file += 1;
        let file = File::open(path)
            .map_err(|error| format!("open Parquet {}: {error}", path.display()))?;
        let builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|error| format!("read Parquet metadata {}: {error}", path.display()))?;
        let schema = builder.schema();
        let time = schema
            .index_of("close_time_us")
            .map_err(|error| format!("find close_time_us in {}: {error}", path.display()))?;
        let close = schema
            .index_of("close")
            .map_err(|error| format!("find close in {}: {error}", path.display()))?;
        let projection = ProjectionMask::roots(builder.parquet_schema(), [time, close]);
        self.reader = Some(
            builder
                .with_projection(projection)
                .with_batch_size(self.batch_size)
                .build()
                .map_err(|error| format!("open Parquet reader {}: {error}", path.display()))?,
        );
        Ok(true)
    }
}
