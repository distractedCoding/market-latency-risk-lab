use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LagSignal {
    pub market_id: String,
    pub poly_mid: f64,
    pub fair_yes_px: f64,
    pub divergence_pct: f64,
    pub triggered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LagError {
    InvalidMarketId,
    InvalidPolyMid,
    InvalidFairValue,
    InvalidThresholdPct,
}

pub fn detect_lag(
    market_id: &str,
    poly_mid: f64,
    fair_yes_px: f64,
    threshold_pct: f64,
) -> Result<LagSignal, LagError> {
    if market_id.trim().is_empty() {
        return Err(LagError::InvalidMarketId);
    }
    if !poly_mid.is_finite() || poly_mid <= 0.0 || poly_mid > 1.0 {
        return Err(LagError::InvalidPolyMid);
    }
    if !fair_yes_px.is_finite() || !(0.0..=1.0).contains(&fair_yes_px) {
        return Err(LagError::InvalidFairValue);
    }
    if !threshold_pct.is_finite() || threshold_pct <= 0.0 || threshold_pct > 100.0 {
        return Err(LagError::InvalidThresholdPct);
    }

    let divergence_pct = ((fair_yes_px - poly_mid) / poly_mid) * 100.0;
    let triggered = divergence_pct.abs() > threshold_pct;

    Ok(LagSignal {
        market_id: market_id.to_string(),
        poly_mid,
        fair_yes_px,
        divergence_pct,
        triggered,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_detector_triggers_above_threshold() {
        let signal = detect_lag("btc", 0.5000, 0.5020, 0.3).unwrap();

        assert!(signal.triggered);
    }

    #[test]
    fn lag_detector_does_not_trigger_at_or_below_threshold() {
        let signal = detect_lag("btc", 0.5000, 0.5015, 0.3).unwrap();

        assert!(!signal.triggered);
    }
}
