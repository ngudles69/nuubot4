//! Observer Executor implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::ExecutorConfig;
use crate::market::{Bar, BboTick};

/// Track work owned by one ObserverExecutor.
struct ExecutorStats {
    ticks_received: u64,
    bars_received: u64,
    passes: u64,
    stop_reason: Option<&'static str>,
}

/// Count admitted events for one bounded control execution.
pub struct ObserverExecutor {
    log: Logger,
    cycle_no: u64,
    executor_no: usize,
    config: ExecutorConfig,
    stats: ExecutorStats,
    terminal: bool,
    stopped: bool,
}

impl ObserverExecutor {
    // Program flow

    /// Initialize one Observer Executor.
    pub fn init(
        log: Logger,
        cycle_no: u64,
        executor_no: usize,
        config: ExecutorConfig,
    ) -> Result<Self> {
        log.info(
            "executor",
            format!("init cycle={cycle_no} executor={executor_no}"),
        );
        Ok(Self {
            log,
            cycle_no,
            executor_no,
            config,
            stats: ExecutorStats {
                ticks_received: 0,
                bars_received: 0,
                passes: 0,
                stop_reason: None,
            },
            terminal: false,
            stopped: false,
        })
    }

    /// Evaluate configured terminal limits.
    pub fn mainloop(&mut self, _now_ms: u64) -> bool {
        self.stats.passes += 1;

        // Check event limits.
        let ticks_done = self
            .config
            .max_ticks
            .is_some_and(|limit| self.stats.ticks_received >= limit);
        let bars_done = self
            .config
            .max_bars
            .is_some_and(|limit| self.stats.bars_received >= limit);
        self.terminal |= ticks_done || bars_done;
        if self.stats.stop_reason.is_none() {
            self.stats.stop_reason = if ticks_done {
                Some("max_ticks")
            } else if bars_done {
                Some("max_bars")
            } else {
                None
            };
        }
        self.terminal
    }

    /// Stop and log Executor stats once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.terminal = true;
        self.stopped = true;
        self.stats.stop_reason.get_or_insert("parent_stop");
        self.log.info(
            "executor",
            format!(
                "stop status=success cycle={} executor={} ticks_received={} bars_received={} passes={} stop_reason={}",
                self.cycle_no,
                self.executor_no,
                self.stats.ticks_received,
                self.stats.bars_received,
                self.stats.passes,
                self.stats.stop_reason.unwrap_or("unknown")
            ),
        );
        Ok(())
    }

    // Domain inputs and state

    /// Count one trusted BBO.
    pub fn on_bbo(&mut self, _bbo: BboTick) {
        if !self.terminal {
            self.stats.ticks_received += 1;
        }
    }

    /// Count one trusted Bar.
    pub fn on_bar(&mut self, _bar: Bar) {
        if !self.terminal {
            self.stats.bars_received += 1;
        }
    }

    /// Report whether this Executor ended.
    pub fn terminal(&self) -> bool {
        self.terminal
    }
}
