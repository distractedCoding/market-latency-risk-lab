use runtime::live::{PredictorSource, PredictorTick};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParsePredictorError {
    InvalidJson,
    InvalidPrediction,
    InvalidConfidence,
}

#[derive(Debug, Deserialize)]
struct TradingViewPayload {
    yes_prediction: f64,
    confidence: f64,
}

#[derive(Debug, Deserialize)]
struct CryptoQuantPayload {
    prediction_yes: f64,
    confidence: f64,
}

pub fn parse_tradingview_payload(
    payload: &str,
    ts_ms: u64,
) -> Result<PredictorTick, ParsePredictorError> {
    let payload: TradingViewPayload =
        serde_json::from_str(payload).map_err(|_| ParsePredictorError::InvalidJson)?;
    normalize_predictor_tick(
        PredictorSource::TradingView,
        payload.yes_prediction,
        payload.confidence,
        ts_ms,
    )
}

pub fn parse_cryptoquant_payload(
    payload: &str,
    ts_ms: u64,
) -> Result<PredictorTick, ParsePredictorError> {
    let payload: CryptoQuantPayload =
        serde_json::from_str(payload).map_err(|_| ParsePredictorError::InvalidJson)?;
    normalize_predictor_tick(
        PredictorSource::CryptoQuant,
        payload.prediction_yes,
        payload.confidence,
        ts_ms,
    )
}

fn normalize_predictor_tick(
    source: PredictorSource,
    predicted_yes_px: f64,
    confidence: f64,
    ts_ms: u64,
) -> Result<PredictorTick, ParsePredictorError> {
    if !predicted_yes_px.is_finite() || !(0.0..=1.0).contains(&predicted_yes_px) {
        return Err(ParsePredictorError::InvalidPrediction);
    }
    if !confidence.is_finite() || confidence <= 0.0 {
        return Err(ParsePredictorError::InvalidConfidence);
    }

    Ok(PredictorTick {
        source,
        predicted_yes_px,
        confidence,
        ts_ms,
    })
}

#[cfg(test)]
mod tests {
    use runtime::live::PredictorSource;

    use super::*;

    #[test]
    fn parses_tradingview_payload_into_predictor_tick() {
        let payload = r#"{"yes_prediction":0.512,"confidence":0.82}"#;

        let tick = parse_tradingview_payload(payload, 100).unwrap();

        assert_eq!(tick.source, PredictorSource::TradingView);
        assert_eq!(tick.predicted_yes_px, 0.512);
        assert_eq!(tick.confidence, 0.82);
        assert_eq!(tick.ts_ms, 100);
    }

    #[test]
    fn parses_cryptoquant_payload_into_predictor_tick() {
        let payload = r#"{"prediction_yes":0.507,"confidence":0.76}"#;

        let tick = parse_cryptoquant_payload(payload, 100).unwrap();

        assert_eq!(tick.source, PredictorSource::CryptoQuant);
        assert_eq!(tick.predicted_yes_px, 0.507);
        assert_eq!(tick.confidence, 0.76);
        assert_eq!(tick.ts_ms, 100);
    }
}
