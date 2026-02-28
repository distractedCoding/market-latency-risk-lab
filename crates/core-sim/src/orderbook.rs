use crate::fills::{Fill, FillSummary};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PriceLevel {
    pub price: f64,
    pub qty: f64,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct OrderBook {
    asks: Vec<PriceLevel>,
}

impl OrderBook {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn default_with_liquidity() -> Self {
        Self {
            asks: vec![
                PriceLevel {
                    price: 100.0,
                    qty: 1.0,
                },
                PriceLevel {
                    price: 101.0,
                    qty: 2.0,
                },
                PriceLevel {
                    price: 102.0,
                    qty: 5.0,
                },
            ],
        }
    }

    pub fn execute_market_buy(&mut self, qty: f64) -> FillSummary {
        if qty <= 0.0 {
            return FillSummary::default();
        }

        let mut remaining = qty;
        let mut filled_qty = 0.0;
        let mut total_notional = 0.0;
        let mut fills = Vec::new();

        for level in &mut self.asks {
            if remaining <= 0.0 {
                break;
            }
            if level.qty <= 0.0 {
                continue;
            }

            let fill_qty = remaining.min(level.qty);
            level.qty -= fill_qty;
            remaining -= fill_qty;
            filled_qty += fill_qty;
            total_notional += fill_qty * level.price;
            fills.push(Fill {
                price: level.price,
                qty: fill_qty,
            });
        }

        self.asks.retain(|level| level.qty > 0.0);

        let avg_price = if filled_qty > 0.0 {
            total_notional / filled_qty
        } else {
            0.0
        };

        FillSummary {
            fills,
            filled_qty,
            avg_price,
            remaining_qty: remaining,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OrderBook;

    #[test]
    fn crossing_order_fills_at_best_level() {
        let mut book = OrderBook::default_with_liquidity();
        let fill = book.execute_market_buy(1.5);

        assert!(fill.filled_qty > 0.0);
        assert!(fill.avg_price >= 100.0 && fill.avg_price <= 101.0);
    }
}
