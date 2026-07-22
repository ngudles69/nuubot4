//! Macross Signaler implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::{MaKind, SignalerConfig};
use crate::market::Bar;

/// Track work owned by one MacrossSignaler.
struct SignalerStats {
    bars_received: u64,
    warmup_bars: u64,
    evaluations: u64,
    no_signal: u64,
    long_signals: u64,
    short_signals: u64,
}

/// Calculate moving-average cross Signals from admitted Bars.
pub struct MacrossSignaler {
    log: Logger,
    config: SignalerConfig,
    closes: Vec<f64>,
    previous_side: Option<i8>,
    stats: SignalerStats,
    stopped: bool,
}

impl MacrossSignaler {
    /// Initialize one Macross Signaler.
    pub fn init(log: Logger, config: SignalerConfig) -> Result<Self> {
        log.info("signaler", "init");
        Ok(Self {
            log,
            config,
            closes: Vec::new(),
            previous_side: None,
            stats: SignalerStats {
                bars_received: 0,
                warmup_bars: 0,
                evaluations: 0,
                no_signal: 0,
                long_signals: 0,
                short_signals: 0,
            },
            stopped: false,
        })
    }

    /// Ingest one trusted Bar and return one boundary cross.
    pub fn on_bar(&mut self, bar: Bar) -> Option<&'static str> {
        self.stats.bars_received += 1;

        // Build slow window.
        self.closes.push(bar.close);
        if self.closes.len() < self.config.slow_ma {
            self.stats.warmup_bars += 1;
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
        self.stats.evaluations += 1;
        match signal {
            Some("long") => self.stats.long_signals += 1,
            Some("short") => self.stats.short_signals += 1,
            _ => self.stats.no_signal += 1,
        }
        signal
    }

    /// Log Signaler stats once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.stopped = true;
        self.log.info(
            "signaler",
            format!(
                "stop status=success bars_received={} warmup_bars={} evaluations={} no_signal={} long_signals={} short_signals={}",
                self.stats.bars_received,
                self.stats.warmup_bars,
                self.stats.evaluations,
                self.stats.no_signal,
                self.stats.long_signals,
                self.stats.short_signals
            ),
        );
        Ok(())
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
