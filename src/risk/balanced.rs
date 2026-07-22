//! Balanced Risk implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::RiskConfig;

/// Provide the current no-exit control Risk.
struct RiskStats {
    assessments: u64,
    exits_requested: u64,
}

pub struct BalancedRisk {
    log: Logger,
    risk_no: usize,
    stats: RiskStats,
    stopped: bool,
}

impl BalancedRisk {
    /// Initialize one Balanced Risk.
    pub fn init(log: Logger, risk_no: usize, _config: RiskConfig) -> Result<Self> {
        log.info("risk", format!("init risk={risk_no}"));
        Ok(Self {
            log,
            risk_no,
            stats: RiskStats {
                assessments: 0,
                exits_requested: 0,
            },
            stopped: false,
        })
    }

    /// Assess the current control Risk state.
    pub fn assess(&mut self) -> bool {
        self.stats.assessments += 1;
        false
    }

    /// Log Risk statistics once.
    pub fn stop(&mut self) -> Result<()> {
        if self.stopped {
            return Ok(());
        }
        self.stopped = true;
        self.log.info(
            "risk",
            format!(
                "stop status=success risk={} assessments={} exits_requested={}",
                self.risk_no, self.stats.assessments, self.stats.exits_requested
            ),
        );
        Ok(())
    }
}
