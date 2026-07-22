//! Balanced Risk implementation.

use crate::Result;
use crate::common::logging::Logger;
use crate::config::RiskConfig;

/// Provide the current no-exit control Risk.
pub struct BalancedRisk;

impl BalancedRisk {
    /// Initialize one Balanced Risk.
    pub fn init(log: Logger, _config: RiskConfig) -> Result<Self> {
        log.info("risk", "init")?;
        Ok(Self)
    }

    /// Assess the current control Risk state.
    pub fn assess(&self) -> bool {
        false
    }
}
