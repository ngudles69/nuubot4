use std::time::Instant;

use crate::clock::TickClock;
use crate::replay::{ReplayExpectation, ReplayWindows, TickReader, load_ticks, replay_expectation};
use crate::runtime::Runtime;
use crate::setup::{SetupContext, nuubot_setup};
use crate::{NuuError, Result};

/// Report one completed standalone replay.
#[derive(Clone, Debug)]
pub struct BtRunSummary {
    pub loader: &'static str,
    pub ticks: u64,
    pub callbacks: u64,
    pub first_ts_ms: u64,
    pub last_ts_ms: u64,
    pub completed_cycles: u64,
    pub stop_reason: Option<&'static str>,
    pub elapsed_seconds: f64,
}

/// Own and supervise one complete backtest program lifecycle.
pub struct BtRunner {
    setup: SetupContext,
    clock: TickClock,
    runtime: Runtime,
    ticks: TickReader,
    windows: ReplayWindows,
    expected: ReplayExpectation,
    started: bool,
    stopped: bool,
}

impl BtRunner {
    /// Set up infrastructure and initialize direct children.
    pub fn init(sweep_id: u64, bot_id: u64) -> Result<Self> {
        // Build shared setup.
        let setup = nuubot_setup(sweep_id, bot_id)?;
        setup.log.info("btrunner", "init")?;

        // Create Tick Clock.
        let clock = TickClock::new(setup.config.btrunner.timer_interval_ms);

        // Load replay ticks.
        let ticks = load_ticks(&setup.bot, &setup.config.btrunner)?;
        let windows = ReplayWindows::new(&setup.bot)?;
        let expected = replay_expectation(&setup.bot, setup.config.btrunner.timer_interval_ms)?;

        // Create Bot Runtime.
        let runtime = Runtime::init(setup.log.clone(), setup.config.runtime.clone())?;
        Ok(Self {
            setup,
            clock,
            runtime,
            ticks,
            windows,
            expected,
            started: false,
            stopped: false,
        })
    }

    /// Start the initialized Bot Runtime.
    pub fn start(&mut self) -> Result<()> {
        if self.started || self.stopped {
            return Err(NuuError::Lifecycle(
                "BtRunner cannot start from current state".into(),
            ));
        }
        self.setup.log.info("btrunner", "start")?;
        self.runtime.start()?;
        self.started = true;
        Ok(())
    }

    /// Replay every admitted tick until end or Bot stop.
    pub fn run(&mut self) -> Result<BtRunSummary> {
        if !self.started || self.stopped {
            return Err(NuuError::Lifecycle("BtRunner is not running".into()));
        }
        self.setup.log.info("btrunner", "run")?;
        let started_at = Instant::now();
        let mut ticks = 0_u64;
        let mut first_ts_ms = None;
        let mut last_ts_ms = None;
        let mut replay_ended = true;

        // Drive ordered replay.
        'windows: for window in &mut self.windows {
            self.setup.log.info(
                "btrunner",
                &format!("replay_window={}..{}", window.start, window.end),
            )?;
            for tick in self.ticks.load_window(window)? {
                first_ts_ms.get_or_insert(tick.ts_ms());
                last_ts_ms = Some(tick.ts_ms());
                ticks += 1;
                self.runtime.ingest_bbo(tick)?;
                if self.clock.advance(tick.ts_ms()) {
                    self.runtime.mainloop(tick.ts_ms())?;
                }
                if self.runtime.outcome().stop_requested {
                    replay_ended = false;
                    break 'windows;
                }
            }
            if self.setup.config.btrunner.window_pause_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(
                    self.setup.config.btrunner.window_pause_ms,
                ));
            }
        }

        // Verify replay evidence.
        let first_ts_ms = first_ts_ms.ok_or_else(|| NuuError::Replay("empty replay".into()))?;
        let last_ts_ms = last_ts_ms.expect("non-empty replay");
        let callbacks = self.runtime.mainloop_count();
        if replay_ended
            && (ticks != self.expected.ticks
                || callbacks != self.expected.callbacks
                || first_ts_ms != self.expected.first_ts_ms
                || last_ts_ms != self.expected.last_ts_ms)
        {
            return Err(NuuError::Replay(format!(
                "evidence ticks={ticks}/{} callbacks={callbacks}/{} range={first_ts_ms}..{last_ts_ms}/{}..{}",
                self.expected.ticks,
                self.expected.callbacks,
                self.expected.first_ts_ms,
                self.expected.last_ts_ms
            )));
        }
        let outcome = self.runtime.outcome();
        Ok(BtRunSummary {
            loader: match self.setup.config.btrunner.loader {
                crate::config::LoaderKind::Csv => "csv",
                crate::config::LoaderKind::Parquet => "parquet",
            },
            ticks,
            callbacks,
            first_ts_ms,
            last_ts_ms,
            completed_cycles: outcome.completed_cycles,
            stop_reason: outcome.stop_reason,
            elapsed_seconds: started_at.elapsed().as_secs_f64(),
        })
    }

    /// Stop Runtime and close admission once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        let log_error = self.setup.log.info("btrunner", "stop").err();
        self.started = false;
        self.stopped = true;
        let runtime_result = self.runtime.stop();
        match log_error {
            Some(error) => Err(error),
            None => runtime_result,
        }
    }
}
