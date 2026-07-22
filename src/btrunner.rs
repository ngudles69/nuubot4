use std::time::Instant;

use crate::Result;
use crate::clock::TickClock;
use crate::common::logging::Logger;
use crate::config::{BtRunnerConfig, LoaderKind};
use crate::datastore::BotSpec;
use crate::replay::{LoadingWindows, TickReader};
use crate::runtime::Runtime;
use crate::setup::{SetupContext, nuubot_setup};

/// Track work owned by one BtRunner.
struct BtRunnerStats {
    loader: &'static str,
    windows_started: u64,
    windows_completed: u64,
    ticks_expected: u64,
    ticks_served: u64,
    passes_expected: u64,
    passes_triggered: u64,
    first_ts_ms: Option<u64>,
    last_ts_ms: Option<u64>,
    expected_first_ts_ms: u64,
    expected_last_ts_ms: u64,
    replay_completed: bool,
    elapsed_seconds: f64,
}

impl BtRunnerStats {
    /// Initialize the exact work assigned to one BtRunner.
    fn init(bot: &BotSpec, config: &BtRunnerConfig) -> Result<Self> {
        let start_ts_ms = u64::try_from(
            bot.start
                .and_hms_opt(0, 0, 0)
                .expect("valid midnight")
                .and_utc()
                .timestamp_millis(),
        )
        .map_err(|_| "replay start precedes Unix epoch".to_owned())?;
        let expected_last_ts_ms = u64::try_from(
            bot.end
                .and_hms_opt(0, 0, 0)
                .expect("valid midnight")
                .and_utc()
                .timestamp_millis(),
        )
        .map_err(|_| "replay end precedes Unix epoch".to_owned())?;
        let duration_ms = expected_last_ts_ms
            .checked_sub(start_ts_ms)
            .ok_or_else(|| "replay end must follow start".to_owned())?;

        Ok(Self {
            loader: match config.loader {
                LoaderKind::Csv => "csv",
                LoaderKind::Parquet => "parquet",
            },
            windows_started: 0,
            windows_completed: 0,
            ticks_expected: duration_ms / 1000,
            ticks_served: 0,
            passes_expected: duration_ms.div_ceil(config.timer_interval_ms),
            passes_triggered: 0,
            first_ts_ms: None,
            last_ts_ms: None,
            expected_first_ts_ms: start_ts_ms + 1000,
            expected_last_ts_ms,
            replay_completed: false,
            elapsed_seconds: 0.0,
        })
    }

    /// Verify that this BtRunner completed its assigned replay.
    fn verify(&mut self) -> Result<()> {
        if self.ticks_served != self.ticks_expected
            || self.passes_triggered != self.passes_expected
            || self.first_ts_ms != Some(self.expected_first_ts_ms)
            || self.last_ts_ms != Some(self.expected_last_ts_ms)
        {
            return Err(format!(
                "replay ticks={}/{} passes={}/{} range={:?}..{:?}/{}..{}",
                self.ticks_served,
                self.ticks_expected,
                self.passes_triggered,
                self.passes_expected,
                self.first_ts_ms,
                self.last_ts_ms,
                self.expected_first_ts_ms,
                self.expected_last_ts_ms
            ));
        }
        self.replay_completed = true;
        Ok(())
    }
}

/// Own and supervise one complete backtest program lifecycle.
pub struct BtRunner {
    ctx: SetupContext,
    log: Logger,
    clock: TickClock,
    ticks: TickReader,
    runtime: Runtime,
    stats: BtRunnerStats,
    started: bool,
    stopped: bool,
}

impl BtRunner {
    /// Set up infrastructure and initialize direct children.
    pub fn init(log: Logger, sweep_id: u64, bot_id: u64) -> Result<Self> {
        // Init context.
        let ctx = nuubot_setup(&log, sweep_id, bot_id)?;
        log.info("btrunner", "init");

        // Init TickClock.
        let clock = TickClock::init(log.clone(), ctx.config.btrunner.timer_interval_ms);

        // Init TickReader.
        let ticks = TickReader::init(log.clone(), &ctx.bot, &ctx.config.btrunner)?;

        // Init Runtime.
        let runtime = Runtime::init(log.clone(), ctx.config.runtime.clone())?;

        // Init BtRunner stats.
        let stats = BtRunnerStats::init(&ctx.bot, &ctx.config.btrunner)?;

        // Create BtRunner.
        let runner = Self {
            ctx,
            log,
            clock,
            ticks,
            runtime,
            stats,
            started: false,
            stopped: false,
        };

        Ok(runner)
    }

    /// Start the initialized Bot Runtime.
    pub fn start(&mut self) -> Result<()> {
        if self.started || self.stopped {
            return Err("BtRunner cannot start from current state".into());
        }
        self.log.info("btrunner", "start");
        self.runtime.start()?;
        self.started = true;
        Ok(())
    }

    /// Replay every admitted tick until end or Bot stop.
    pub fn run(&mut self) -> Result<()> {
        if !self.started || self.stopped {
            return Err("BtRunner is not running".into());
        }

        // Start replay.
        self.log.info("btrunner", "run");
        let started_at = Instant::now();
        let result = self.replay();
        self.stats.elapsed_seconds = started_at.elapsed().as_secs_f64();
        result
    }

    /// Serve admitted ticks and trigger due Runtime passes.
    fn replay(&mut self) -> Result<()> {
        // Replay windows.
        'windows: for window in LoadingWindows::new(&self.ctx.bot)? {
            self.stats.windows_started += 1;
            self.log.info(
                "btrunner",
                format!("replay_window={}..{}", window.start, window.end),
            );

            // Replay ticks.
            for tick in self.ticks.load(window)? {
                // Serve one tick.
                self.runtime.ingest_bbo(tick)?;
                self.stats.first_ts_ms.get_or_insert(tick.ts_ms());
                self.stats.last_ts_ms = Some(tick.ts_ms());
                self.stats.ticks_served += 1;

                // Trigger one due Runtime pass.
                if self.clock.advance(tick.ts_ms()) {
                    self.stats.passes_triggered += 1;
                    if self.runtime.mainloop(tick.ts_ms())? {
                        break 'windows;
                    }
                }
            }
            self.stats.windows_completed += 1;

            // Pause between windows.
            if self.ctx.config.btrunner.window_pause_ms > 0 {
                std::thread::sleep(std::time::Duration::from_millis(
                    self.ctx.config.btrunner.window_pause_ms,
                ));
            }
        }

        // Verify replay.
        self.stats.verify()
    }

    /// Stop direct children and log BtRunner stats once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        if !self.started {
            return Err("BtRunner is not running".into());
        }
        self.started = false;
        self.stopped = true;

        // Stop direct children in reverse initialization order.
        let mut first_error = None;
        if let Err(error) = self.runtime.stop() {
            first_error = Some(error);
        }
        if let Err(error) = self.ticks.stop() {
            first_error.get_or_insert(error);
        }
        if let Err(error) = self.clock.stop() {
            first_error.get_or_insert(error);
        }

        // Log only BtRunner-owned stats.
        let status = if self.stats.replay_completed && first_error.is_none() {
            "success"
        } else {
            "failed"
        };
        self.log.info(
            "btrunner",
            format!(
                "stop status={status} loader={} windows={}/{} ticks={}/{} passes={}/{} first_ts_ms={:?} last_ts_ms={:?} replay_completed={} elapsed_seconds={:.3}",
                self.stats.loader,
                self.stats.windows_completed,
                self.stats.windows_started,
                self.stats.ticks_served,
                self.stats.ticks_expected,
                self.stats.passes_triggered,
                self.stats.passes_expected,
                self.stats.first_ts_ms,
                self.stats.last_ts_ms,
                self.stats.replay_completed,
                self.stats.elapsed_seconds
            ),
        );

        if let Some(error) = first_error {
            return Err(error);
        }
        if !self.stats.replay_completed {
            return Err("BtRunner replay did not complete".into());
        }
        Ok(())
    }
}
