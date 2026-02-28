use crate::divergence::{
    normalized_divergence, signal_from_normalized_divergence, Signal, StrategyError,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveSignal {
    pub action: Signal,
    pub normalized_divergence: f64,
}

pub fn live_signal(
    prediction_price: f64,
    market_price: f64,
    threshold: f64,
) -> Result<LiveSignal, StrategyError> {
    let normalized_divergence = normalized_divergence(prediction_price, market_price)?;
    let action = signal_from_normalized_divergence(normalized_divergence, threshold)?;

    Ok(LiveSignal {
        action,
        normalized_divergence,
    })
}

#[cfg(test)]
mod tests {
    use super::live_signal;
    use crate::Signal;
    use crate::StrategyError;

    #[test]
    fn emits_buy_signal_when_prediction_exceeds_market_threshold() {
        let signal = live_signal(64_200.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Signal::Buy);
    }

    #[test]
    fn emits_sell_signal_when_prediction_is_below_market_threshold() {
        let signal = live_signal(63_500.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Signal::Sell);
    }

    #[test]
    fn emits_hold_signal_when_normalized_divergence_is_within_threshold_band() {
        let signal = live_signal(63_900.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Signal::Hold);
    }

    #[test]
    fn rejects_non_positive_market_price_for_live_signal() {
        let error = live_signal(64_200.0, 0.0, 0.003).unwrap_err();
        assert_eq!(error, StrategyError::NonPositiveMarketPrice);
    }
}
