use std::time::Instant;

use crate::Result;
use crate::clock::TickClock;
use crate::common::logging::Logger;
use crate::replay::{ReplayExpectation, ReplayWindows, TickReader, load_ticks, replay_expectation};
use crate::runtime::Runtime;
use crate::setup::{SetupContext, nuubot_setup};

/// Report one completed standalone replay.
#[derive(Clone, Debug)]
struct BtRunSummary {
    loader: &'static str,
    ticks: u64,
    callbacks: u64,
    first_ts_ms: u64,
    last_ts_ms: u64,
    completed_cycles: u64,
    stop_reason: Option<&'static str>,
    elapsed_seconds: f64,
}

/// Own and supervise one complete backtest program lifecycle.
pub struct BtRunner {
    setup: SetupContext,
    clock: TickClock,
    runtime: Runtime,
    ticks: TickReader,
    windows: ReplayWindows,
    expected: ReplayExpectation,
    summary: Option<BtRunSummary>,
    started: bool,
    stopped: bool,
}

impl BtRunner {
    /// Set up infrastructure and initialize direct children.
    pub fn init(log: Logger, sweep_id: u64, bot_id: u64) -> Result<Self> {
        // Build shared setup.
        let setup = nuubot_setup(log, sweep_id, bot_id)?;
        setup.log.info("btrunner", "init");

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
            summary: None,
            started: false,
            stopped: false,
        })
    }

    /// Start the initialized Bot Runtime.
    pub fn start(&mut self) -> Result<()> {
        if self.started || self.stopped {
            return Err("BtRunner cannot start from current state".into());
        }
        self.setup.log.info("btrunner", "start");
        self.runtime.start()?;
        self.started = true;
        Ok(())
    }

    /// Replay every admitted tick until end or Bot stop.
    pub fn run(&mut self) -> Result<()> {
        // lifecycle guard.
        if !self.started || self.stopped {
            return Err("BtRunner is not running".into());
        }

        // Start replay.
        self.setup.log.info("btrunner", "run");
        let started_at = Instant::now();
        let mut ticks = 0_u64;
        let mut first_ts_ms = None;
        let mut last_ts_ms = None;
        let mut replay_ended = true;

        // Replay windows.
        'windows: for window in &mut self.windows {
            self.setup.log.info(
                "btrunner",
                format!("replay_window={}..{}", window.start, window.end),
            );

            // Replay ticks.
            for tick in self.ticks.load_window(window)? {
                // Record replay evidence.
                first_ts_ms.get_or_insert(tick.ts_ms());
                last_ts_ms = Some(tick.ts_ms());
                ticks += 1;

                // Drive Runtime and Clock.
                self.runtime.ingest_bbo(tick)?;
                if self.clock.advance(tick.ts_ms()) {
                    self.runtime.mainloop(tick.ts_ms())?;
                }

                // Stop when Runtime requests it.
                if self.runtime.outcome().stop_requested {
                    replay_ended = false;
                    break 'windows;
                }
            }

            // Pause between windows.
            if self.setup.config.btrunner.window_pause_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(
                    self.setup.config.btrunner.window_pause_ms,
                ));
            }
        }

        // Verify replay evidence.
        let first_ts_ms = first_ts_ms.ok_or_else(|| "empty replay".to_owned())?;
        let last_ts_ms = last_ts_ms.expect("non-empty replay");
        let callbacks = self.runtime.mainloop_count();
        if replay_ended
            && (ticks != self.expected.ticks
                || callbacks != self.expected.callbacks
                || first_ts_ms != self.expected.first_ts_ms
                || last_ts_ms != self.expected.last_ts_ms)
        {
            return Err(format!(
                "evidence ticks={ticks}/{} callbacks={callbacks}/{} range={first_ts_ms}..{last_ts_ms}/{}..{}",
                self.expected.ticks,
                self.expected.callbacks,
                self.expected.first_ts_ms,
                self.expected.last_ts_ms
            ));
        }

        // Store replay summary.
        let outcome = self.runtime.outcome();
        self.summary = Some(BtRunSummary {
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
        });
        Ok(())
    }

    /// Stop Runtime and close admission once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.setup.log.info("btrunner", "stop");
        self.started = false;
        self.stopped = true;
        self.runtime.stop()?;
        if let Some(summary) = self.summary.take() {
            self.setup.log.info("btrunner", format_summary(&summary));
        }
        Ok(())
    }
}

/// Format one successful replay result.
fn format_summary(summary: &BtRunSummary) -> String {
    format!(
        "PASS loader={} ticks={} callbacks={} first_ts_ms={} last_ts_ms={} completed_cycles={} stop_reason={} elapsed_seconds={:.3}",
        summary.loader,
        summary.ticks,
        summary.callbacks,
        summary.first_ts_ms,
        summary.last_ts_ms,
        summary.completed_cycles,
        summary.stop_reason.unwrap_or("replay_end"),
        summary.elapsed_seconds
    )
}
