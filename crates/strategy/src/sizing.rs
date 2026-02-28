use crate::divergence::Signal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Calm,
    Normal,
    Volatile,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizingConfig {
    pub base_order_size: f64,
}

impl Default for SizingConfig {
    fn default() -> Self {
        Self {
            base_order_size: 1.0,
        }
    }
}

pub fn regime_multiplier(regime: Regime) -> f64 {
    match regime {
        Regime::Calm => 1.0,
        Regime::Normal => 1.0,
        Regime::Volatile => 0.5,
    }
}

pub fn size_for_signal(signal: Signal, regime: Regime, config: SizingConfig) -> f64 {
    match signal {
        Signal::Hold => 0.0,
        Signal::Buy | Signal::Sell => config.base_order_size * regime_multiplier(regime),
    }
}
