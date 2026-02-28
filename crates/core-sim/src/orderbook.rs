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

    pub fn from_asks(mut asks: Vec<PriceLevel>) -> Self {
        asks.retain(|level| {
            level.price.is_finite() && level.price > 0.0 && level.qty.is_finite() && level.qty > 0.0
        });
        asks.sort_by(|left, right| left.price.total_cmp(&right.price));

        Self { asks }
    }

    pub fn default_with_liquidity() -> Self {
        Self::from_asks(vec![
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
        ])
    }

    pub fn asks(&self) -> &[PriceLevel] {
        &self.asks
    }

    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    pub fn execute_market_buy(&mut self, qty: f64) -> FillSummary {
        if !qty.is_finite() || qty <= 0.0 {
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
    use crate::fills::Fill;

    use super::{OrderBook, PriceLevel};

    #[test]
    fn crossing_order_fills_at_best_level() {
        let mut book = OrderBook::default_with_liquidity();
        let fill = book.execute_market_buy(4.0);

        assert_eq!(
            fill.fills,
            vec![
                Fill {
                    price: 100.0,
                    qty: 1.0,
                },
                Fill {
                    price: 101.0,
                    qty: 2.0,
                },
                Fill {
                    price: 102.0,
                    qty: 1.0,
                },
            ]
        );
        assert_eq!(fill.filled_qty, 4.0);
        assert_eq!(fill.remaining_qty, 0.0);
        assert_eq!(fill.avg_price, 101.0);
    }

    #[test]
    fn invalid_market_buy_qty_is_no_op() {
        let mut book = OrderBook::default_with_liquidity();

        let zero_fill = book.execute_market_buy(0.0);
        let negative_fill = book.execute_market_buy(-1.0);
        let nan_fill = book.execute_market_buy(f64::NAN);
        let infinity_fill = book.execute_market_buy(f64::INFINITY);

        assert_eq!(zero_fill.filled_qty, 0.0);
        assert_eq!(negative_fill.filled_qty, 0.0);
        assert_eq!(nan_fill.filled_qty, 0.0);
        assert_eq!(infinity_fill.filled_qty, 0.0);
        assert_eq!(book, OrderBook::default_with_liquidity());
    }

    #[test]
    fn partial_fill_tracks_remaining_qty() {
        let mut book = OrderBook::default_with_liquidity();
        let fill = book.execute_market_buy(9.0);

        assert_eq!(fill.filled_qty, 8.0);
        assert_eq!(fill.remaining_qty, 1.0);
        assert_eq!(book.asks().len(), 0);
    }

    #[test]
    fn liquidity_exhaustion_clears_book() {
        let mut book = OrderBook::default_with_liquidity();
        let fill = book.execute_market_buy(100.0);

        assert_eq!(fill.filled_qty, 8.0);
        assert_eq!(fill.remaining_qty, 92.0);
        assert_eq!(fill.fills.len(), 3);
        assert_eq!(book.best_ask(), None);
    }

    #[test]
    fn sequential_market_buys_mutate_state_and_remove_levels() {
        let mut book = OrderBook::from_asks(vec![
            PriceLevel {
                price: 101.0,
                qty: 2.0,
            },
            PriceLevel {
                price: 100.0,
                qty: 1.0,
            },
            PriceLevel {
                price: 102.0,
                qty: 3.0,
            },
        ]);

        assert_eq!(book.best_ask().map(|level| level.price), Some(100.0));

        let first_fill = book.execute_market_buy(1.5);
        assert_eq!(first_fill.filled_qty, 1.5);
        assert_eq!(first_fill.remaining_qty, 0.0);
        assert_eq!(book.best_ask().map(|level| level.price), Some(101.0));
        assert_eq!(book.asks().len(), 2);

        let second_fill = book.execute_market_buy(2.5);
        assert_eq!(second_fill.filled_qty, 2.5);
        assert_eq!(second_fill.remaining_qty, 0.0);
        assert_eq!(book.asks().len(), 1);
        assert_eq!(book.best_ask().map(|level| level.price), Some(102.0));
        assert_eq!(book.asks()[0].qty, 2.0);
    }

    #[test]
    fn from_asks_filters_invalid_levels_and_sorts_by_price() {
        let book = OrderBook::from_asks(vec![
            PriceLevel {
                price: 103.0,
                qty: 1.0,
            },
            PriceLevel {
                price: f64::NAN,
                qty: 3.0,
            },
            PriceLevel {
                price: 100.0,
                qty: 2.0,
            },
            PriceLevel {
                price: 101.0,
                qty: 0.0,
            },
            PriceLevel {
                price: 102.0,
                qty: f64::INFINITY,
            },
        ]);

        assert_eq!(
            book.asks(),
            &[
                PriceLevel {
                    price: 100.0,
                    qty: 2.0,
                },
                PriceLevel {
                    price: 103.0,
                    qty: 1.0,
                },
            ]
        );
    }
}
