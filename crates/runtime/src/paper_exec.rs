#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperFill {
    pub fill_px: f64,
    pub qty: f64,
    pub notional: f64,
    pub fee: f64,
}

pub fn paper_fill_buy(
    best_ask: f64,
    qty: f64,
    slippage_bps: f64,
    fee_bps: f64,
) -> Result<PaperFill, String> {
    validate_inputs(best_ask, qty, slippage_bps, fee_bps)?;

    let slippage_rate = bps_to_rate(slippage_bps);
    let fee_rate = bps_to_rate(fee_bps);
    let fill_px = best_ask * (1.0 + slippage_rate);
    let notional = fill_px * qty;
    let fee = notional * fee_rate;

    Ok(PaperFill {
        fill_px,
        qty,
        notional,
        fee,
    })
}

pub fn paper_fill_sell(
    best_bid: f64,
    qty: f64,
    slippage_bps: f64,
    fee_bps: f64,
) -> Result<PaperFill, String> {
    validate_inputs(best_bid, qty, slippage_bps, fee_bps)?;

    let slippage_rate = bps_to_rate(slippage_bps);
    let fee_rate = bps_to_rate(fee_bps);
    let fill_px = best_bid * (1.0 - slippage_rate);
    if fill_px <= 0.0 {
        return Err("fill price must remain positive".to_string());
    }
    let notional = fill_px * qty;
    let fee = notional * fee_rate;

    Ok(PaperFill {
        fill_px,
        qty,
        notional,
        fee,
    })
}

fn validate_inputs(price: f64, qty: f64, slippage_bps: f64, fee_bps: f64) -> Result<(), String> {
    if !price.is_finite() || price <= 0.0 {
        return Err("price must be finite and positive".to_string());
    }
    if !qty.is_finite() || qty <= 0.0 {
        return Err("quantity must be finite and positive".to_string());
    }
    if !slippage_bps.is_finite() || slippage_bps < 0.0 {
        return Err("slippage_bps must be finite and non-negative".to_string());
    }
    if !fee_bps.is_finite() || fee_bps < 0.0 {
        return Err("fee_bps must be finite and non-negative".to_string());
    }

    Ok(())
}

fn bps_to_rate(bps: f64) -> f64 {
    bps / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::{paper_fill_buy, paper_fill_sell};

    #[test]
    fn buy_fill_uses_ask_plus_slippage_and_fee() {
        let fill = paper_fill_buy(0.62, 5.0, 10.0, 2.0).unwrap();
        assert!(fill.fill_px > 0.62);
    }

    #[test]
    fn sell_fill_uses_bid_minus_slippage_and_fee() {
        let fill = paper_fill_sell(0.62, 5.0, 10.0, 2.0).unwrap();

        assert!(fill.fill_px < 0.62);
        assert!(fill.fee > 0.0);
    }

    #[test]
    fn rejects_invalid_inputs() {
        assert!(paper_fill_buy(0.0, 1.0, 1.0, 1.0).is_err());
        assert!(paper_fill_buy(0.5, 0.0, 1.0, 1.0).is_err());
        assert!(paper_fill_buy(0.5, 1.0, -1.0, 1.0).is_err());
        assert!(paper_fill_sell(0.5, 1.0, 10_000.0, 1.0).is_err());
    }
}
