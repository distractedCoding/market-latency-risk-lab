#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolymarketMarket {
    pub slug: String,
}

pub fn filter_markets(markets: Vec<PolymarketMarket>, needle: &str) -> Vec<PolymarketMarket> {
    let needle = needle.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return markets;
    }

    markets
        .into_iter()
        .filter(|market| market.slug.to_ascii_lowercase().contains(&needle))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{filter_markets, PolymarketMarket};

    #[test]
    fn discovery_filters_market_candidates() {
        let markets = vec![sample_market("btc-up-down"), sample_market("sports-final")];
        let out = filter_markets(markets, "btc");
        assert_eq!(out, vec![sample_market("btc-up-down")]);
    }

    #[test]
    fn discovery_is_case_insensitive() {
        let markets = vec![sample_market("BTC-up-down"), sample_market("sports-final")];
        let out = filter_markets(markets, "btc");
        assert_eq!(out, vec![sample_market("BTC-up-down")]);
    }

    #[test]
    fn discovery_trims_query() {
        let markets = vec![sample_market("btc-up-down"), sample_market("sports-final")];
        let out = filter_markets(markets, "  btc  ");
        assert_eq!(out, vec![sample_market("btc-up-down")]);
    }

    #[test]
    fn discovery_whitespace_only_query_returns_all_markets() {
        let markets = vec![sample_market("btc-up-down"), sample_market("sports-final")];
        let out = filter_markets(markets.clone(), "   ");
        assert_eq!(out, markets);
    }

    fn sample_market(slug: &str) -> PolymarketMarket {
        PolymarketMarket {
            slug: slug.to_string(),
        }
    }
}
