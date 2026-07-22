use crate::common::logging::Logger;
use crate::config::ExecutorConfig;
use crate::executor::ObserverExecutor;
use crate::market::{Bar, BboTick};
use crate::{NuuError, Result};

/// Own temporary control Executors for one Bot cycle.
pub struct ControlBotCycle {
    log: Logger,
    executors: Vec<ObserverExecutor>,
    running: bool,
    stopped: bool,
}

impl ControlBotCycle {
    // Program flow

    /// Create and initialize every configured Executor.
    pub fn init(log: Logger, configs: &[ExecutorConfig]) -> Result<Self> {
        log.info("botcycle", "init")?;
        let executors = configs
            .iter()
            .cloned()
            .map(|config| ObserverExecutor::init(log.clone(), config))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            log,
            executors,
            running: false,
            stopped: false,
        })
    }

    /// Open this cycle for bounded work.
    pub fn start(&mut self) -> Result<()> {
        if self.running || self.stopped {
            return Err(NuuError::Lifecycle(
                "BotCycle cannot start from current state".into(),
            ));
        }
        self.log.info("botcycle", "start")?;
        self.running = true;
        Ok(())
    }

    /// Advance every active Executor once.
    pub fn mainloop(&mut self, now_ms: u64) -> Result<bool> {
        if !self.running {
            return Err(NuuError::Lifecycle("BotCycle is not running".into()));
        }
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.mainloop(now_ms);
            }
        }
        Ok(self.executors.iter().all(ObserverExecutor::terminal))
    }

    /// Stop every Executor in reverse order once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.running = false;
        self.stopped = true;

        // Preserve first failure.
        let mut first_error = self.log.info("botcycle", "stop").err();
        for executor in self.executors.iter_mut().rev() {
            if let Err(error) = executor.stop() {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    // Domain inputs

    /// Deliver one trusted BBO to active Executors.
    pub fn on_bbo(&mut self, bbo: BboTick) {
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.on_bbo(bbo);
            }
        }
    }

    /// Deliver one trusted Bar to active Executors.
    pub fn on_bar(&mut self, bar: Bar) {
        for executor in &mut self.executors {
            if !executor.terminal() {
                executor.on_bar(bar);
            }
        }
    }
}
