//! Observer Executor implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::ExecutorConfig;
use crate::market::{Bar, BboTick};

/// Count admitted events for one bounded control execution.
pub struct ObserverExecutor {
    log: Logger,
    config: ExecutorConfig,
    tick_count: u64,
    bar_count: u64,
    terminal: bool,
}

impl ObserverExecutor {
    /// Initialize one Observer Executor.
    pub fn init(log: Logger, config: ExecutorConfig) -> Result<Self> {
        log.info("executor", "init")?;
        Ok(Self {
            log,
            config,
            tick_count: 0,
            bar_count: 0,
            terminal: false,
        })
    }

    /// Count one trusted BBO.
    pub fn on_bbo(&mut self, _bbo: BboTick) {
        if !self.terminal {
            self.tick_count += 1;
        }
    }

    /// Count one trusted Bar.
    pub fn on_bar(&mut self, _bar: Bar) {
        if !self.terminal {
            self.bar_count += 1;
        }
    }

    /// Evaluate configured terminal limits.
    pub fn mainloop(&mut self, _now_ms: u64) -> bool {
        // Check event limits.
        let ticks_done = self
            .config
            .max_ticks
            .is_some_and(|limit| self.tick_count >= limit);
        let bars_done = self
            .config
            .max_bars
            .is_some_and(|limit| self.bar_count >= limit);
        self.terminal |= ticks_done || bars_done;
        self.terminal
    }

    /// Stop this Executor once.
    pub fn stop(&mut self) -> Result<()> {
        if !self.terminal {
            self.terminal = true;
            self.log.info("executor", "stop")?;
        }
        Ok(())
    }

    /// Report whether this Executor ended.
    pub fn terminal(&self) -> bool {
        self.terminal
    }
}
