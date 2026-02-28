pub mod divergence;
pub mod sizing;

pub use divergence::{divergence, emit_signal, Signal};
pub use sizing::{regime_multiplier, size_for_signal, Regime, SizingConfig};

pub fn module_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::divergence::{emit_signal, Signal};

    #[test]
    fn emits_buy_signal_when_prediction_leads_market_above_threshold() {
        let signal = emit_signal(101.0, 100.0, 0.5);

        assert_eq!(signal, Signal::Buy);
    }

    #[test]
    fn emits_sell_signal_when_prediction_lags_market_below_negative_threshold() {
        let signal = emit_signal(99.0, 100.0, 0.5);

        assert_eq!(signal, Signal::Sell);
    }

    #[test]
    fn emits_hold_signal_when_divergence_is_within_threshold_band() {
        let signal = emit_signal(100.2, 100.0, 0.5);

        assert_eq!(signal, Signal::Hold);
    }
}
