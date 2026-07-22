use crate::{NuuError, Result};

/// Carry one validated best-bid-offer replay event.
#[derive(Clone, Copy, Debug)]
pub struct BboTick {
    ts_ms: u64,
    price: f64,
}

impl BboTick {
    /// Admit one external quote into trusted internal state.
    pub fn admit(ts_ms: u64, price: f64) -> Result<Self> {
        // Validate boundary once.
        if ts_ms == 0 || !price.is_finite() || price <= 0.0 {
            return Err(NuuError::Replay(format!(
                "invalid BBO timestamp={ts_ms} price={price}"
            )));
        }
        Ok(Self { ts_ms, price })
    }

    /// Return the trusted quote timestamp.
    pub fn ts_ms(self) -> u64 {
        self.ts_ms
    }

    /// Return the trusted quote price.
    pub fn price(self) -> f64 {
        self.price
    }
}

/// Carry one admitted bar for Signaler calculations.
#[derive(Clone, Copy, Debug)]
pub struct Bar {
    pub close: f64,
}
