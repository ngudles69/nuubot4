use crate::Result;
use crate::common::logging::Logger;

/// Track work owned by one TickClock.
struct TickClockStats {
    ticks_seen: u64,
    passes_due: u64,
}

/// Drive Runtime callbacks from admitted replay time.
pub struct TickClock {
    log: Logger,
    interval_ms: u64,
    next_ms: Option<u64>,
    stats: TickClockStats,
    stopped: bool,
}

impl TickClock {
    /// Initialize one replay Clock.
    pub fn init(log: Logger, interval_ms: u64) -> Self {
        log.info("tickclock", "init");
        Self {
            log,
            interval_ms,
            next_ms: None,
            stats: TickClockStats {
                ticks_seen: 0,
                passes_due: 0,
            },
            stopped: false,
        }
    }

    /// Advance time and invoke one due Runtime pass.
    pub fn advance(&mut self, now_ms: u64) -> bool {
        self.stats.ticks_seen += 1;

        // Calculate one callback.
        let due = match self.next_ms {
            None => {
                self.next_ms = Some(now_ms + self.interval_ms);
                true
            }
            Some(next) if now_ms >= next => {
                let intervals = ((now_ms - next) / self.interval_ms) + 1;
                self.next_ms = Some(next + intervals * self.interval_ms);
                true
            }
            Some(_) => false,
        };
        if due {
            self.stats.passes_due += 1;
        }
        due
    }

    /// Log TickClock stats once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.stopped = true;
        self.log.info(
            "tickclock",
            format!(
                "stop status=success ticks_seen={} passes_due={}",
                self.stats.ticks_seen, self.stats.passes_due
            ),
        );
        Ok(())
    }
}
