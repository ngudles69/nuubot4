use crate::Result;
use crate::common::logging::Logger;
use crate::config::ExecutorConfig;
use crate::executor::ObserverExecutor;
use crate::market::{Bar, BboTick};

/// Track work owned by one BotCycle.
struct BotCycleStats {
    ticks_received: u64,
    bars_received: u64,
    passes: u64,
    completed: bool,
}

/// Own temporary control Executors for one Bot cycle.
pub struct ControlBotCycle {
    log: Logger,
    cycle_no: u64,
    executors: Vec<ObserverExecutor>,
    stats: BotCycleStats,
    running: bool,
    stopped: bool,
}

impl ControlBotCycle {
    // Program flow

    /// Create and initialize every configured Executor.
    pub fn init(log: Logger, cycle_no: u64, configs: &[ExecutorConfig]) -> Result<Self> {
        log.info("botcycle", format!("init cycle={cycle_no}"));
        let executors = configs
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, config)| ObserverExecutor::init(log.clone(), cycle_no, index + 1, config))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            log,
            cycle_no,
            executors,
            stats: BotCycleStats {
                ticks_received: 0,
                bars_received: 0,
                passes: 0,
                completed: false,
            },
            running: false,
            stopped: false,
        })
    }

    /// Open this cycle for bounded work.
    pub fn start(&mut self) -> Result<()> {
        if self.running || self.stopped {
            return Err("BotCycle cannot start from current state".into());
        }
        self.log
            .info("botcycle", format!("start cycle={}", self.cycle_no));
        self.running = true;
        Ok(())
    }

    /// Advance every active Executor once.
    pub fn mainloop(&mut self, now_ms: u64) -> Result<bool> {
        if !self.running {
            return Err("BotCycle is not running".into());
        }
        self.stats.passes += 1;
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.mainloop(now_ms);
            }
        }
        self.stats.completed = self.executors.iter().all(ObserverExecutor::terminal);
        Ok(self.stats.completed)
    }

    /// Stop every Executor in reverse order once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.running = false;
        self.stopped = true;

        // Stop direct children in reverse initialization order.
        let mut first_error = None;
        for executor in self.executors.iter_mut().rev() {
            if let Err(error) = executor.stop() {
                first_error.get_or_insert(error);
            }
        }

        // Log only BotCycle-owned stats.
        let status = if first_error.is_none() {
            "success"
        } else {
            "failed"
        };
        let stop_reason = if self.stats.completed {
            "completed"
        } else {
            "parent_stop"
        };
        self.log.info(
            "botcycle",
            format!(
                "stop status={status} cycle={} executors={} ticks_received={} bars_received={} passes={} stop_reason={stop_reason}",
                self.cycle_no,
                self.executors.len(),
                self.stats.ticks_received,
                self.stats.bars_received,
                self.stats.passes
            ),
        );
        first_error.map_or(Ok(()), Err)
    }

    // Domain inputs

    /// Deliver one trusted BBO to active Executors.
    pub fn on_bbo(&mut self, bbo: BboTick) {
        self.stats.ticks_received += 1;
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.on_bbo(bbo);
            }
        }
    }

    /// Deliver one trusted Bar to active Executors.
    pub fn on_bar(&mut self, bar: Bar) {
        self.stats.bars_received += 1;
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.on_bar(bar);
            }
        }
    }
}
