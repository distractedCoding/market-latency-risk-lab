#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyError {
    NonFiniteInput,
    NonPositiveMarketPrice,
    NegativeThreshold,
    InvalidBaseOrderSize,
    InvalidPositionSize,
    InvalidStartingEquity,
    InvalidDailyLossCapPct,
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

pub fn emit_signal(
    prediction_price: f64,
    market_price: f64,
    threshold: f64,
) -> Result<Signal, StrategyError> {
    if !threshold.is_finite() {
        return Err(StrategyError::NonFiniteInput);
    }
    if threshold < 0.0 {
        return Err(StrategyError::NegativeThreshold);
    }

    let divergence = divergence(prediction_price, market_price)?;

    if divergence > threshold {
        Ok(Signal::Buy)
    } else if divergence < -threshold {
        Ok(Signal::Sell)
    } else {
        Ok(Signal::Hold)
    }
}
