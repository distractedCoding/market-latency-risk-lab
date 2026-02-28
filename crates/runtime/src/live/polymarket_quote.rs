use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolymarketQuoteTick {
    pub market_slug: String,
    pub best_yes_bid: f64,
    pub best_yes_ask: f64,
    pub mid_yes: f64,
    pub ts: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NormalizePolymarketQuoteError {
    NonFinite,
    OutOfRange,
    CrossedBook,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPolymarketQuote {
    pub market_slug: String,
    pub best_yes_bid: f64,
    pub best_yes_ask: f64,
    pub ts: u64,
}

impl RawPolymarketQuote {
    pub fn normalize(self) -> Result<PolymarketQuoteTick, NormalizePolymarketQuoteError> {
        if !self.best_yes_bid.is_finite() || !self.best_yes_ask.is_finite() {
            return Err(NormalizePolymarketQuoteError::NonFinite);
        }
        if self.best_yes_bid < 0.0
            || self.best_yes_bid > 1.0
            || self.best_yes_ask < 0.0
            || self.best_yes_ask > 1.0
        {
            return Err(NormalizePolymarketQuoteError::OutOfRange);
        }
        if self.best_yes_bid > self.best_yes_ask {
            return Err(NormalizePolymarketQuoteError::CrossedBook);
        }

        let mid_yes = (self.best_yes_bid + self.best_yes_ask) / 2.0;

        Ok(PolymarketQuoteTick {
            market_slug: self.market_slug,
            best_yes_bid: self.best_yes_bid,
            best_yes_ask: self.best_yes_ask,
            mid_yes,
            ts: self.ts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{NormalizePolymarketQuoteError, RawPolymarketQuote};

    #[test]
    fn normalize_quote_computes_mid() {
        let quote = RawPolymarketQuote {
            market_slug: "btc-up-down".to_string(),
            best_yes_bid: 0.45,
            best_yes_ask: 0.55,
            ts: 1,
        };

        let out = quote.normalize().unwrap();
        assert_eq!(out.mid_yes, 0.5);
    }

    #[test]
    fn normalize_quote_rejects_non_finite_values() {
        let quote = RawPolymarketQuote {
            market_slug: "btc-up-down".to_string(),
            best_yes_bid: f64::NAN,
            best_yes_ask: 0.55,
            ts: 1,
        };

        let out = quote.normalize();
        assert_eq!(out, Err(NormalizePolymarketQuoteError::NonFinite));
    }

    #[test]
    fn normalize_quote_rejects_negative_bid() {
        let quote = RawPolymarketQuote {
            market_slug: "btc-up-down".to_string(),
            best_yes_bid: -0.01,
            best_yes_ask: 0.55,
            ts: 1,
        };

        let out = quote.normalize();
        assert_eq!(out, Err(NormalizePolymarketQuoteError::OutOfRange));
    }

    #[test]
    fn normalize_quote_rejects_ask_above_one() {
        let quote = RawPolymarketQuote {
            market_slug: "btc-up-down".to_string(),
            best_yes_bid: 0.45,
            best_yes_ask: 1.01,
            ts: 1,
        };

        let out = quote.normalize();
        assert_eq!(out, Err(NormalizePolymarketQuoteError::OutOfRange));
    }

    #[test]
    fn normalize_quote_rejects_crossed_book() {
        let quote = RawPolymarketQuote {
            market_slug: "btc-up-down".to_string(),
            best_yes_bid: 0.56,
            best_yes_ask: 0.55,
            ts: 1,
        };

        let out = quote.normalize();
        assert_eq!(out, Err(NormalizePolymarketQuoteError::CrossedBook));
    }
}
