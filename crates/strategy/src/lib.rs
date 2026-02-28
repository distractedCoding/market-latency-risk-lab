pub mod divergence;
pub mod risk;
pub mod sizing;

pub use divergence::{divergence, emit_signal, Signal, StrategyError};
pub use risk::RiskState;
pub use sizing::{regime_multiplier, size_for_signal, Regime, SizingConfig};

pub fn module_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::divergence::{emit_signal, Signal, StrategyError};
    use crate::risk::RiskState;
    use crate::sizing::{size_for_signal, Regime, SizingConfig};

    #[test]
    fn emits_buy_signal_when_prediction_leads_market_above_threshold() {
        let signal = emit_signal(101.0, 100.0, 0.5);

        assert_eq!(signal, Ok(Signal::Buy));
    }

    #[test]
    fn emits_sell_signal_when_prediction_lags_market_below_negative_threshold() {
        let signal = emit_signal(99.0, 100.0, 0.5);

        assert_eq!(signal, Ok(Signal::Sell));
    }

    #[test]
    fn emits_hold_signal_when_divergence_is_within_threshold_band() {
        let signal = emit_signal(100.2, 100.0, 0.5);

        assert_eq!(signal, Ok(Signal::Hold));
    }

    #[test]
    fn emits_hold_signal_at_threshold_boundaries() {
        assert_eq!(emit_signal(100.5, 100.0, 0.5), Ok(Signal::Hold));
        assert_eq!(emit_signal(99.5, 100.0, 0.5), Ok(Signal::Hold));
    }

    #[test]
    fn rejects_negative_threshold() {
        assert_eq!(
            emit_signal(101.0, 100.0, -0.1),
            Err(StrategyError::NegativeThreshold)
        );
    }

    #[test]
    fn rejects_non_finite_inputs_for_signal_generation() {
        assert_eq!(
            emit_signal(f64::NAN, 100.0, 0.1),
            Err(StrategyError::NonFiniteInput)
        );
        assert_eq!(
            emit_signal(101.0, f64::INFINITY, 0.1),
            Err(StrategyError::NonFiniteInput)
        );
        assert_eq!(
            emit_signal(101.0, 100.0, f64::NEG_INFINITY),
            Err(StrategyError::NonFiniteInput)
        );
    }

    #[test]
    fn rejects_non_positive_market_price() {
        assert_eq!(
            emit_signal(101.0, 0.0, 0.1),
            Err(StrategyError::NonPositiveMarketPrice)
        );
        assert_eq!(
            emit_signal(101.0, -1.0, 0.1),
            Err(StrategyError::NonPositiveMarketPrice)
        );
    }

    #[test]
    fn sizing_returns_zero_for_hold_signal() {
        let size = size_for_signal(Signal::Hold, Regime::Volatile, SizingConfig::default());

        assert_eq!(size, Ok(0.0));
    }

    #[test]
    fn sizing_applies_regime_scaling() {
        let config = SizingConfig::new(2.0).expect("valid sizing config");

        assert_eq!(
            size_for_signal(Signal::Buy, Regime::Normal, config),
            Ok(2.0)
        );
        assert_eq!(
            size_for_signal(Signal::Sell, Regime::Volatile, config),
            Ok(1.0)
        );
    }

    #[test]
    fn sizing_rejects_invalid_config_numeric_cases() {
        assert_eq!(
            SizingConfig::new(0.0),
            Err(StrategyError::InvalidBaseOrderSize)
        );
        assert_eq!(
            SizingConfig::new(-1.0),
            Err(StrategyError::InvalidBaseOrderSize)
        );
        assert_eq!(
            SizingConfig::new(f64::NAN),
            Err(StrategyError::InvalidBaseOrderSize)
        );
        assert_eq!(
            SizingConfig::new(f64::INFINITY),
            Err(StrategyError::InvalidBaseOrderSize)
        );
    }

    #[test]
    fn halts_when_daily_loss_cap_is_breached() {
        let mut risk = RiskState::new(100_000.0, 2.0).expect("valid risk state");

        risk.apply_realized_pnl(-2_001.0).expect("valid pnl update");

        assert!(risk.is_halted());
    }
}
