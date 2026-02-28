use serde::{Deserialize, Serialize};

const DEFAULT_FRESHNESS_WINDOW_MS: u64 = 5_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictorSource {
    TradingView,
    CryptoQuant,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PredictorTick {
    pub source: PredictorSource,
    pub predicted_yes_px: f64,
    pub confidence: f64,
    pub ts_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FusedFairValue {
    pub fair_yes_px: f64,
    pub source_count: usize,
    pub freshness_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PredictorFusionError {
    InvalidPrice,
    InvalidConfidence,
    NoFreshSources,
}

pub fn fuse_predictors(
    ticks: &[PredictorTick],
    now_ms: u64,
) -> Result<FusedFairValue, PredictorFusionError> {
    let mut weighted_sum = 0.0;
    let mut confidence_sum = 0.0;
    let mut source_count = 0usize;
    let mut max_age_ms = 0u64;

    for tick in ticks {
        if !tick.predicted_yes_px.is_finite() || !(0.0..=1.0).contains(&tick.predicted_yes_px) {
            return Err(PredictorFusionError::InvalidPrice);
        }
        if !tick.confidence.is_finite() || tick.confidence <= 0.0 {
            return Err(PredictorFusionError::InvalidConfidence);
        }

        let age_ms = now_ms.saturating_sub(tick.ts_ms);
        if age_ms > DEFAULT_FRESHNESS_WINDOW_MS {
            continue;
        }

        weighted_sum += tick.predicted_yes_px * tick.confidence;
        confidence_sum += tick.confidence;
        source_count += 1;
        if age_ms > max_age_ms {
            max_age_ms = age_ms;
        }
    }

    if source_count == 0 || confidence_sum <= 0.0 {
        return Err(PredictorFusionError::NoFreshSources);
    }

    Ok(FusedFairValue {
        fair_yes_px: weighted_sum / confidence_sum,
        source_count,
        freshness_ms: max_age_ms,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fused_fair_value_averages_fresh_predictors() {
        let fused = fuse_predictors(&[tv_tick(), cq_tick()], 10_000).unwrap();

        assert!(fused.fair_yes_px > 0.0);
        assert_eq!(fused.source_count, 2);
    }

    #[test]
    fn fused_fair_value_excludes_stale_predictors() {
        let fused = fuse_predictors(&[stale_tv_tick(), cq_tick()], 10_000).unwrap();

        assert_eq!(fused.source_count, 1);
    }

    fn tv_tick() -> PredictorTick {
        PredictorTick {
            source: PredictorSource::TradingView,
            predicted_yes_px: 0.513,
            confidence: 0.9,
            ts_ms: 9_800,
        }
    }

    fn cq_tick() -> PredictorTick {
        PredictorTick {
            source: PredictorSource::CryptoQuant,
            predicted_yes_px: 0.509,
            confidence: 0.8,
            ts_ms: 9_900,
        }
    }

    fn stale_tv_tick() -> PredictorTick {
        PredictorTick {
            source: PredictorSource::TradingView,
            predicted_yes_px: 0.6,
            confidence: 0.9,
            ts_ms: 0,
        }
    }
}
