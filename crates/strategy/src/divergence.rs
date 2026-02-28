#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signal {
    Buy,
    Sell,
    Hold,
}

pub fn divergence(prediction_price: f64, market_price: f64) -> f64 {
    prediction_price - market_price
}

pub fn emit_signal(prediction_price: f64, market_price: f64, threshold: f64) -> Signal {
    let divergence = divergence(prediction_price, market_price);
    let threshold = threshold.abs();

    if divergence > threshold {
        Signal::Buy
    } else if divergence < -threshold {
        Signal::Sell
    } else {
        Signal::Hold
    }
}
