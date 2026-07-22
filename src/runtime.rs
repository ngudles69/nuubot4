use crate::botcycle::ControlBotCycle;
use crate::common::logging::Logger;
use crate::config::RuntimeConfig;
use crate::market::{Bar, BboTick};
use crate::risk::BalancedRisk;
use crate::signaler::MacrossSignaler;
use crate::{NuuError, Result};

/// Expose one bounded Runtime result to BtRunner.
#[derive(Clone, Copy, Debug)]
pub struct RuntimeOutcome {
    pub stop_requested: bool,
    pub stop_reason: Option<&'static str>,
    pub completed_cycles: u64,
}

/// Own and sequence one Bot's control components.
pub struct Runtime {
    log: Logger,
    config: RuntimeConfig,
    signaler: MacrossSignaler,
    risks: Vec<BalancedRisk>,
    botcycle: Option<ControlBotCycle>,
    mainloop_count: u64,
    completed_cycles: u64,
    stop_reason: Option<&'static str>,
    started: bool,
    stopped: bool,
}

impl Runtime {
    /// Create and initialize the Runtime-owned subtree.
    pub fn init(log: Logger, config: RuntimeConfig) -> Result<Self> {
        log.info("runtime", "init")?;

        // Initialize direct children.
        let signaler = MacrossSignaler::init(log.clone(), config.signaler.clone())?;
        let risks = config
            .risks
            .iter()
            .cloned()
            .map(|config| BalancedRisk::init(log.clone(), config))
            .collect::<Result<Vec<_>>>()?;
        let botcycle = ControlBotCycle::init(log.clone(), &config.executors)?;
        Ok(Self {
            log,
            config,
            signaler,
            risks,
            botcycle: Some(botcycle),
            mainloop_count: 0,
            completed_cycles: 0,
            stop_reason: None,
            started: false,
            stopped: false,
        })
    }

    /// Open Runtime admission after its subtree is ready.
    pub fn start(&mut self) -> Result<()> {
        if self.started || self.stopped {
            return Err(NuuError::Lifecycle(
                "Runtime cannot start from current state".into(),
            ));
        }
        self.log.info("runtime", "start")?;

        // Start active cycle.
        self.botcycle
            .as_mut()
            .ok_or_else(|| NuuError::Lifecycle("Runtime has no BotCycle".into()))?
            .start()?;
        self.started = true;
        Ok(())
    }

    /// Deliver one trusted BBO to the active cycle.
    pub fn ingest_bbo(&mut self, bbo: BboTick) -> Result<()> {
        if !self.started || self.stopped || self.stop_reason.is_some() {
            return Err(NuuError::Lifecycle(
                "Runtime cannot ingest BBO from current state".into(),
            ));
        }
        self.botcycle
            .as_mut()
            .ok_or_else(|| NuuError::Lifecycle("Runtime has no BotCycle".into()))?
            .on_bbo(bbo);
        Ok(())
    }

    /// Deliver trusted Bars through Signaler then BotCycle.
    pub fn ingest_bars(&mut self, bars: &[Bar]) -> Result<()> {
        if !self.started || self.stopped || self.stop_reason.is_some() {
            return Err(NuuError::Lifecycle(
                "Runtime cannot ingest Bars from current state".into(),
            ));
        }
        for bar in bars {
            self.signaler.on_bar(*bar);
            self.botcycle
                .as_mut()
                .expect("initialized BotCycle")
                .on_bar(*bar);
        }
        Ok(())
    }

    /// Perform one bounded Bot decision pass.
    pub fn mainloop(&mut self, now_ms: u64) -> Result<RuntimeOutcome> {
        if !self.started || self.stopped {
            return Err(NuuError::Lifecycle("Runtime is not running".into()));
        }
        self.mainloop_count += 1;

        // Resolve Risk first.
        if self.risks.iter().any(BalancedRisk::assess) {
            self.request_stop("risk");
        }
        if self.stop_reason.is_some() {
            self.close_cycle()?;
            return Ok(self.outcome());
        }

        // Advance active cycle.
        let completed = self
            .botcycle
            .as_mut()
            .ok_or_else(|| NuuError::Lifecycle("Runtime has no BotCycle".into()))?
            .mainloop(now_ms)?;
        if !completed {
            return Ok(self.outcome());
        }

        // Replace completed cycle.
        self.close_cycle()?;
        self.completed_cycles += 1;
        if self.completed_cycles >= self.config.max_cycles {
            self.request_stop("max_cycles");
            return Ok(self.outcome());
        }
        let mut next = ControlBotCycle::init(self.log.clone(), &self.config.executors)?;
        next.start()?;
        self.botcycle = Some(next);
        Ok(self.outcome())
    }

    /// Stop admission and unwind direct children once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        // Close child ownership.
        let log_error = self.log.info("runtime", "stop").err();
        self.request_stop("parent_stop");
        self.started = false;
        self.stopped = true;
        let child_result = self.close_cycle();
        match log_error {
            Some(error) => Err(error),
            None => child_result,
        }
    }

    /// Return current Runtime evidence.
    pub fn outcome(&self) -> RuntimeOutcome {
        RuntimeOutcome {
            stop_requested: self.stop_reason.is_some(),
            stop_reason: self.stop_reason,
            completed_cycles: self.completed_cycles,
        }
    }

    /// Return the completed callback count.
    pub fn mainloop_count(&self) -> u64 {
        self.mainloop_count
    }

    /// Latch the first stop reason.
    fn request_stop(&mut self, reason: &'static str) {
        if self.stop_reason.is_none() {
            self.stop_reason = Some(reason);
        }
    }

    /// Stop and release the active cycle.
    fn close_cycle(&mut self) -> Result<()> {
        if let Some(mut botcycle) = self.botcycle.take() {
            botcycle.stop()?;
        }
        Ok(())
    }
}
