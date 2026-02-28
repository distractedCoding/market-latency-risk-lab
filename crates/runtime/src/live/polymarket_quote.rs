use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolymarketQuoteTick {
    pub market_slug: String,
    pub best_yes_bid: f64,
    pub best_yes_ask: f64,
    pub mid_yes: f64,
    pub ts: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPolymarketQuote {
    pub market_slug: String,
    pub best_yes_bid: f64,
    pub best_yes_ask: f64,
    pub ts: u64,
}

impl RawPolymarketQuote {
    pub fn normalize(self) -> Option<PolymarketQuoteTick> {
        if !self.best_yes_bid.is_finite() || !self.best_yes_ask.is_finite() {
            return None;
        }
        if self.best_yes_bid < 0.0 || self.best_yes_ask > 1.0 || self.best_yes_bid > self.best_yes_ask {
            return None;
        }

        let mid_yes = (self.best_yes_bid + self.best_yes_ask) / 2.0;

        Some(PolymarketQuoteTick {
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
    use super::RawPolymarketQuote;

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
}
