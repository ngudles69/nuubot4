//! Macross Signaler implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::{MaKind, SignalerConfig};
use crate::market::Bar;

/// Calculate moving-average cross Signals from admitted Bars.
pub struct MacrossSignaler {
    config: SignalerConfig,
    closes: Vec<f64>,
    previous_side: Option<i8>,
}

impl MacrossSignaler {
    /// Initialize one Macross Signaler.
    pub fn init(log: Logger, config: SignalerConfig) -> Result<Self> {
        log.info("signaler", "init")?;
        Ok(Self {
            config,
            closes: Vec::new(),
            previous_side: None,
        })
    }

    /// Ingest one trusted Bar and return one boundary cross.
    pub fn on_bar(&mut self, bar: Bar) -> Option<&'static str> {
        // Build slow window.
        self.closes.push(bar.close);
        if self.closes.len() < self.config.slow_ma {
            return None;
        }
        let remove = self.closes.len() - self.config.slow_ma;
        self.closes.drain(..remove);

        // Calculate both averages.
        let fast = moving_average(&self.closes, self.config.fast_ma, self.config.ma_kind);
        let slow = moving_average(&self.closes, self.config.slow_ma, self.config.ma_kind);
        let side = if fast > slow {
            1
        } else if fast < slow {
            -1
        } else {
            0
        };

        // Emit boundary change.
        let signal = match self.previous_side {
            Some(previous) if previous != side && side > 0 => Some("long"),
            Some(previous) if previous != side && side < 0 => Some("short"),
            _ => None,
        };
        self.previous_side = Some(side);
        signal
    }
}

/// Calculate one configured moving average.
fn moving_average(values: &[f64], period: usize, kind: MaKind) -> f64 {
    let window = &values[values.len() - period..];
    match kind {
        MaKind::Sma => window.iter().sum::<f64>() / period as f64,
        MaKind::Ema => {
            let alpha = 2.0 / (period as f64 + 1.0);
            window[1..].iter().fold(window[0], |average, value| {
                average + alpha * (value - average)
            })
        }
    }
}
