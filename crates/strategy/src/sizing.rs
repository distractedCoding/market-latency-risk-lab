use crate::divergence::{Signal, StrategyError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Regime {
    Calm,
    Normal,
    Volatile,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizingConfig {
    base_order_size: f64,
}

impl SizingConfig {
    pub fn new(base_order_size: f64) -> Result<Self, StrategyError> {
        if !base_order_size.is_finite() || base_order_size <= 0.0 {
            return Err(StrategyError::InvalidBaseOrderSize);
        }

        Ok(Self { base_order_size })
    }

    pub fn base_order_size(&self) -> f64 {
        self.base_order_size
    }
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

pub fn size_for_signal(
    signal: Signal,
    regime: Regime,
    config: SizingConfig,
) -> Result<f64, StrategyError> {
    if !config.base_order_size.is_finite() || config.base_order_size <= 0.0 {
        return Err(StrategyError::InvalidBaseOrderSize);
    }

    let size = match signal {
        Signal::Hold => 0.0,
        Signal::Buy | Signal::Sell => config.base_order_size * regime_multiplier(regime),
    };

    if !size.is_finite() || size < 0.0 {
        return Err(StrategyError::InvalidPositionSize);
    }

    Ok(size)
}
