//! Parquet replay decoder.

use std::fs::File;
use std::path::PathBuf;

use arrow_array::{Array, Float64Array, Int64Array, RecordBatch};
use parquet::arrow::ProjectionMask;
use parquet::arrow::arrow_reader::{ParquetRecordBatchReader, ParquetRecordBatchReaderBuilder};

use crate::market::BboTick;
use crate::{NuuError, Result};

use super::{admit_tick, validate_sequence};

pub(super) struct ParquetTickReader {
    files: Vec<PathBuf>,
    next_file: usize,
    reader: Option<ParquetRecordBatchReader>,
    batch: Option<RecordBatch>,
    row: usize,
    batch_size: usize,
    start_us: u64,
    end_us: u64,
    last_us: Option<u64>,
    failed: bool,
}

impl ParquetTickReader {
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
            last_us: None,
            failed: false,
        }
    }

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
                        .ok_or_else(|| NuuError::Replay("close_time_us must be Int64".into()))?;
                    let prices = batch
                        .column_by_name("close")
                        .and_then(|column| column.as_any().downcast_ref::<Float64Array>())
                        .ok_or_else(|| NuuError::Replay("close must be Float64".into()))?;
                    if times.is_null(row) || prices.is_null(row) {
                        return Err(NuuError::Replay("Parquet BBO contains null".into()));
                    }
                    let close_time_us = u64::try_from(times.value(row)).map_err(|_| {
                        NuuError::Replay("close_time_us must be non-negative".into())
                    })?;
                    if close_time_us < self.start_us || close_time_us >= self.end_us {
                        continue;
                    }
                    validate_sequence(&mut self.last_us, close_time_us)?;
                    return Ok(Some(admit_tick(close_time_us, prices.value(row))?));
                }
                self.batch = None;
            }

            // Read next batch.
            if self.reader.is_none() && !self.open_next()? {
                return Ok(None);
            }
            match self.reader.as_mut().expect("opened Parquet").next() {
                Some(batch) => {
                    self.batch = Some(batch.map_err(|error| NuuError::Replay(error.to_string()))?);
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
        let builder = ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?;
        let schema = builder.schema();
        let time = schema
            .index_of("close_time_us")
            .map_err(|error| NuuError::Replay(error.to_string()))?;
        let close = schema
            .index_of("close")
            .map_err(|error| NuuError::Replay(error.to_string()))?;
        let projection = ProjectionMask::roots(builder.parquet_schema(), [time, close]);
        self.reader = Some(
            builder
                .with_projection(projection)
                .with_batch_size(self.batch_size)
                .build()?,
        );
        Ok(true)
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
