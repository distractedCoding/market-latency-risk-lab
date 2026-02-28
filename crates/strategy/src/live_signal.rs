use crate::divergence::{normalized_divergence, signal_from_divergence, Signal, StrategyError};

pub type Action = Signal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LiveSignal {
    pub action: Action,
    pub normalized_divergence: f64,
}

pub fn live_signal(
    btc_reference_price: f64,
    market_price: f64,
    threshold: f64,
) -> Result<LiveSignal, StrategyError> {
    let normalized_divergence = normalized_divergence(btc_reference_price, market_price)?;
    let action = signal_from_divergence(normalized_divergence, threshold)?;

    Ok(LiveSignal {
        action,
        normalized_divergence,
    })
}

#[cfg(test)]
mod tests {
    use super::{live_signal, Action};
    use crate::StrategyError;

    #[test]
    fn emits_buy_intent_when_btc_reference_exceeds_market_threshold() {
        let signal = live_signal(64_200.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Action::Buy);
    }

    #[test]
    fn emits_sell_intent_when_btc_reference_is_below_market_threshold() {
        let signal = live_signal(63_500.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Action::Sell);
    }

    #[test]
    fn emits_hold_intent_when_normalized_divergence_is_within_threshold_band() {
        let signal = live_signal(63_900.0, 63_800.0, 0.003).unwrap();
        assert_eq!(signal.action, Action::Hold);
    }

    #[test]
    fn rejects_non_positive_market_price_for_live_signal() {
        let error = live_signal(64_200.0, 0.0, 0.003).unwrap_err();
        assert_eq!(error, StrategyError::NonPositiveMarketPrice);
    }
}
