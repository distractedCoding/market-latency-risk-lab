#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PaperFill {
    pub fill_px: f64,
    pub qty: f64,
    pub notional: f64,
    pub fee: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaperExecError {
    InvalidPrice,
    InvalidQuantity,
    InvalidSlippageBps,
    InvalidFeeBps,
    SellFillPriceNonPositive,
    FillPriceOutOfBounds,
}

pub fn paper_fill_buy(
    best_ask: f64,
    qty: f64,
    slippage_bps: f64,
    fee_bps: f64,
) -> Result<PaperFill, PaperExecError> {
    validate_inputs(best_ask, qty, slippage_bps, fee_bps)?;

    let slippage_rate = bps_to_rate(slippage_bps);
    let fee_rate = bps_to_rate(fee_bps);
    let fill_px = best_ask * (1.0 + slippage_rate);
    validate_fill_price(fill_px)?;
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
) -> Result<PaperFill, PaperExecError> {
    validate_inputs(best_bid, qty, slippage_bps, fee_bps)?;

    let slippage_rate = bps_to_rate(slippage_bps);
    let fee_rate = bps_to_rate(fee_bps);
    let fill_px = best_bid * (1.0 - slippage_rate);
    if fill_px <= 0.0 {
        return Err(PaperExecError::SellFillPriceNonPositive);
    }
    validate_fill_price(fill_px)?;
    let notional = fill_px * qty;
    let fee = notional * fee_rate;

    Ok(PaperFill {
        fill_px,
        qty,
        notional,
        fee,
    })
}

fn validate_inputs(
    price: f64,
    qty: f64,
    slippage_bps: f64,
    fee_bps: f64,
) -> Result<(), PaperExecError> {
    if !price.is_finite() || !(0.0..=1.0).contains(&price) {
        return Err(PaperExecError::InvalidPrice);
    }
    if !qty.is_finite() || qty <= 0.0 {
        return Err(PaperExecError::InvalidQuantity);
    }
    if !slippage_bps.is_finite() || slippage_bps < 0.0 {
        return Err(PaperExecError::InvalidSlippageBps);
    }
    if !fee_bps.is_finite() || fee_bps < 0.0 {
        return Err(PaperExecError::InvalidFeeBps);
    }

    Ok(())
}

fn validate_fill_price(fill_px: f64) -> Result<(), PaperExecError> {
    if !fill_px.is_finite() || !(0.0..=1.0).contains(&fill_px) {
        return Err(PaperExecError::FillPriceOutOfBounds);
    }

    Ok(())
}

fn bps_to_rate(bps: f64) -> f64 {
    bps / 10_000.0
}

#[cfg(test)]
mod tests {
    use super::{paper_fill_buy, paper_fill_sell, PaperExecError};

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
        assert_eq!(
            paper_fill_buy(-0.1, 1.0, 1.0, 1.0),
            Err(PaperExecError::InvalidPrice)
        );
        assert_eq!(
            paper_fill_buy(0.5, 0.0, 1.0, 1.0),
            Err(PaperExecError::InvalidQuantity)
        );
        assert_eq!(
            paper_fill_buy(0.5, 1.0, -1.0, 1.0),
            Err(PaperExecError::InvalidSlippageBps)
        );
        assert_eq!(
            paper_fill_sell(0.5, 1.0, 10_000.0, 1.0),
            Err(PaperExecError::SellFillPriceNonPositive)
        );
    }

    #[test]
    fn accepts_zero_quote_price_input() {
        let fill = paper_fill_buy(0.0, 1.0, 0.0, 0.0).unwrap();
        assert_eq!(fill.fill_px, 0.0);
    }

    #[test]
    fn rejects_buy_fill_price_above_one_due_to_slippage() {
        assert_eq!(
            paper_fill_buy(0.9999, 1.0, 2.0, 0.0),
            Err(PaperExecError::FillPriceOutOfBounds)
        );
    }
}
