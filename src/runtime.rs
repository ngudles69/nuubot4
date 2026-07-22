use crate::Result;
use crate::botcycle::ControlBotCycle;
use crate::common::logging::Logger;
use crate::config::RuntimeConfig;
use crate::market::{Bar, BboTick};
use crate::risk::BalancedRisk;
use crate::signaler::MacrossSignaler;

/// Track work owned by one Runtime.
struct RuntimeStats {
    ticks_accepted: u64,
    bars_accepted: u64,
    passes_started: u64,
    passes_completed: u64,
    passes_failed: u64,
    cycles_started: u64,
    completed_cycles: u64,
    failed: bool,
}

/// Own and sequence one Bot's control components.
pub struct Runtime {
    log: Logger,
    config: RuntimeConfig,
    signaler: MacrossSignaler,
    risks: Vec<BalancedRisk>,
    botcycle: Option<ControlBotCycle>,
    stats: RuntimeStats,
    stop_reason: Option<&'static str>,
    started: bool,
    stopped: bool,
}

impl Runtime {
    // Program flow

    /// Create and initialize the Runtime-owned subtree.
    pub fn init(log: Logger, config: RuntimeConfig) -> Result<Self> {
        log.info("runtime", "init");

        // Initialize direct children.
        let signaler = MacrossSignaler::init(log.clone(), config.signaler.clone())?;
        let risks = config
            .risks
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, config)| BalancedRisk::init(log.clone(), index + 1, config))
            .collect::<Result<Vec<_>>>()?;
        let botcycle = ControlBotCycle::init(log.clone(), 1, &config.executors)?;
        Ok(Self {
            log,
            config,
            signaler,
            risks,
            botcycle: Some(botcycle),
            stats: RuntimeStats {
                ticks_accepted: 0,
                bars_accepted: 0,
                passes_started: 0,
                passes_completed: 0,
                passes_failed: 0,
                cycles_started: 0,
                completed_cycles: 0,
                failed: false,
            },
            stop_reason: None,
            started: false,
            stopped: false,
        })
    }

    /// Open Runtime admission after its subtree is ready.
    pub fn start(&mut self) -> Result<()> {
        if self.started || self.stopped {
            return Err("Runtime cannot start from current state".into());
        }
        self.log.info("runtime", "start");

        // Start active cycle.
        self.botcycle
            .as_mut()
            .ok_or_else(|| "Runtime has no BotCycle".to_owned())?
            .start()?;
        self.stats.cycles_started = 1;
        self.started = true;
        Ok(())
    }

    /// Perform one bounded Bot pass and report whether Runtime requested stop.
    pub fn mainloop(&mut self, now_ms: u64) -> Result<bool> {
        if !self.started || self.stopped {
            return Err("Runtime is not running".into());
        }
        self.stats.passes_started += 1;

        let result = self.run_pass(now_ms);
        match result {
            Ok(stop_requested) => {
                self.stats.passes_completed += 1;
                Ok(stop_requested)
            }
            Err(error) => {
                self.stats.passes_failed += 1;
                self.stats.failed = true;
                self.request_stop("runtime_error");
                Err(error)
            }
        }
    }

    /// Run the owned work for one Runtime pass.
    fn run_pass(&mut self, now_ms: u64) -> Result<bool> {
        // Resolve Risk first.
        if self.risks.iter_mut().any(BalancedRisk::assess) {
            self.request_stop("risk");
        }
        if self.stop_reason.is_some() {
            self.close_cycle()?;
            return Ok(true);
        }

        // Advance active cycle.
        let completed = self
            .botcycle
            .as_mut()
            .ok_or_else(|| "Runtime has no BotCycle".to_owned())?
            .mainloop(now_ms)?;
        if !completed {
            return Ok(false);
        }

        // Replace completed cycle.
        self.close_cycle()?;
        self.stats.completed_cycles += 1;
        if self.stats.completed_cycles >= self.config.max_cycles {
            self.request_stop("max_cycles");
            return Ok(true);
        }
        let cycle_no = self.stats.cycles_started + 1;
        let mut next = ControlBotCycle::init(self.log.clone(), cycle_no, &self.config.executors)?;
        next.start()?;
        self.botcycle = Some(next);
        self.stats.cycles_started += 1;
        Ok(false)
    }

    /// Stop direct children and log Runtime stats once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.request_stop("parent_stop");
        self.started = false;
        self.stopped = true;

        // Stop direct children in reverse initialization order.
        let mut first_error = None;
        if let Err(error) = self.close_cycle() {
            first_error = Some(error);
        }
        for risk in self.risks.iter_mut().rev() {
            if let Err(error) = risk.stop() {
                first_error.get_or_insert(error);
            }
        }
        if let Err(error) = self.signaler.stop() {
            first_error.get_or_insert(error);
        }

        // Log only Runtime-owned stats.
        let status = if first_error.is_none() && !self.stats.failed {
            "success"
        } else {
            "failed"
        };
        self.log.info(
            "runtime",
            format!(
                "stop status={status} ticks_accepted={} bars_accepted={} passes={}/{} failed_passes={} cycles={}/{} stop_reason={}",
                self.stats.ticks_accepted,
                self.stats.bars_accepted,
                self.stats.passes_completed,
                self.stats.passes_started,
                self.stats.passes_failed,
                self.stats.completed_cycles,
                self.stats.cycles_started,
                self.stop_reason.unwrap_or("unknown")
            ),
        );
        first_error.map_or(Ok(()), Err)
    }

    // Domain inputs and state

    /// Deliver one trusted BBO to the active cycle.
    pub fn ingest_bbo(&mut self, bbo: BboTick) -> Result<()> {
        if !self.started || self.stopped || self.stop_reason.is_some() {
            return Err("Runtime cannot ingest BBO from current state".into());
        }
        self.botcycle
            .as_mut()
            .ok_or_else(|| "Runtime has no BotCycle".to_owned())?
            .on_bbo(bbo);
        self.stats.ticks_accepted += 1;
        Ok(())
    }

    /// Deliver trusted Bars through Signaler then BotCycle.
    pub fn ingest_bars(&mut self, bars: &[Bar]) -> Result<()> {
        if !self.started || self.stopped || self.stop_reason.is_some() {
            return Err("Runtime cannot ingest Bars from current state".into());
        }
        for bar in bars {
            self.signaler.on_bar(*bar);
            self.botcycle
                .as_mut()
                .expect("initialized BotCycle")
                .on_bar(*bar);
            self.stats.bars_accepted += 1;
        }
        Ok(())
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
