/// Drive Runtime callbacks from admitted replay time.
pub struct TickClock {
    interval_ms: u64,
    next_ms: Option<u64>,
}

impl TickClock {
    /// Create one stopped replay Clock.
    pub fn new(interval_ms: u64) -> Self {
        Self {
            interval_ms,
            next_ms: None,
        }
    }

    /// Advance time and invoke one due Runtime pass.
    pub fn advance(&mut self, now_ms: u64) -> bool {
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
        due
    }
}
