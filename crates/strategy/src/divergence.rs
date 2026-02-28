#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyError {
    NonFiniteInput,
    NonFiniteMarketExposure,
    NonPositiveMarketPrice,
    NegativeThreshold,
    InvalidBaseOrderSize,
    InvalidMarketId,
    InvalidMarketExposure,
    InvalidPositionSize,
    InvalidStartingEquity,
    InvalidDailyLossCapPct,
    MarketExposureCapExceeded,
    NonFinitePnl,
}

pub fn divergence(prediction_price: f64, market_price: f64) -> Result<f64, StrategyError> {
    if !prediction_price.is_finite() || !market_price.is_finite() {
        return Err(StrategyError::NonFiniteInput);
    }
    if market_price <= 0.0 {
        return Err(StrategyError::NonPositiveMarketPrice);
    }

    Ok(prediction_price - market_price)
}

pub fn normalized_divergence(
    prediction_price: f64,
    market_price: f64,
) -> Result<f64, StrategyError> {
    let raw_divergence = divergence(prediction_price, market_price)?;

    Ok(raw_divergence / market_price)
}

fn signal_from_thresholded_divergence(
    divergence: f64,
    threshold: f64,
) -> Result<Signal, StrategyError> {
    if !divergence.is_finite() || !threshold.is_finite() {
        return Err(StrategyError::NonFiniteInput);
    }
    if threshold < 0.0 {
        return Err(StrategyError::NegativeThreshold);
    }

    if divergence > threshold {
        Ok(Signal::Buy)
    } else if divergence < -threshold {
        Ok(Signal::Sell)
    } else {
        Ok(Signal::Hold)
    }
}

pub fn signal_from_raw_divergence(
    raw_divergence: f64,
    threshold: f64,
) -> Result<Signal, StrategyError> {
    signal_from_thresholded_divergence(raw_divergence, threshold)
}

pub fn signal_from_normalized_divergence(
    normalized_divergence: f64,
    threshold: f64,
) -> Result<Signal, StrategyError> {
    signal_from_thresholded_divergence(normalized_divergence, threshold)
}

pub fn emit_signal(
    prediction_price: f64,
    market_price: f64,
    threshold: f64,
) -> Result<Signal, StrategyError> {
    let raw_divergence = divergence(prediction_price, market_price)?;

    signal_from_raw_divergence(raw_divergence, threshold)
}

#[cfg(test)]
mod tests {
    use super::{
        signal_from_normalized_divergence, signal_from_raw_divergence, Signal, StrategyError,
    };

    #[test]
    fn raw_divergence_threshold_uses_absolute_price_delta_units() {
        assert_eq!(signal_from_raw_divergence(0.25, 0.1), Ok(Signal::Buy));
        assert_eq!(signal_from_raw_divergence(-0.25, 0.1), Ok(Signal::Sell));
    }

    #[test]
    fn normalized_divergence_threshold_uses_ratio_units() {
        assert_eq!(
            signal_from_normalized_divergence(0.004, 0.003),
            Ok(Signal::Buy)
        );
        assert_eq!(
            signal_from_normalized_divergence(-0.004, 0.003),
            Ok(Signal::Sell)
        );
    }

    #[test]
    fn normalized_divergence_rejects_negative_threshold() {
        assert_eq!(
            signal_from_normalized_divergence(0.004, -0.003),
            Err(StrategyError::NegativeThreshold)
        );
    }
}
