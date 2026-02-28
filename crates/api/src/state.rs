use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, RwLock,
};

use tokio::sync::broadcast;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeedMode {
    PaperLive,
    Sim,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SourceCount {
    pub source: String,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FeedHealthResponse {
    pub mode: FeedMode,
    pub source_counts: Vec<SourceCount>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct DiscoveredMarket {
    pub source: String,
    pub market_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct DiscoveredMarketsResponse {
    pub markets: Vec<DiscoveredMarket>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct PortfolioSummary {
    pub equity: f64,
    pub pnl: f64,
    pub position_qty: f64,
    pub fills: u64,
}

impl Default for PortfolioSummary {
    fn default() -> Self {
        Self {
            equity: 0.0,
            pnl: 0.0,
            position_qty: 0.0,
            fills: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct PriceSnapshot {
    pub coinbase_btc_usd: Option<f64>,
    pub binance_btc_usdt: Option<f64>,
    pub kraken_btc_usd: Option<f64>,
    pub polymarket_market_id: Option<String>,
    pub polymarket_yes_bid: Option<f64>,
    pub polymarket_yes_ask: Option<f64>,
    pub polymarket_yes_mid: Option<f64>,
    pub ts: u64,
}

impl Default for PriceSnapshot {
    fn default() -> Self {
        Self {
            coinbase_btc_usd: None,
            binance_btc_usdt: None,
            kraken_btc_usd: None,
            polymarket_market_id: None,
            polymarket_yes_bid: None,
            polymarket_yes_ask: None,
            polymarket_yes_mid: None,
            ts: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartRunError {
    RunIdOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperOrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum RuntimeEvent {
    Connected {
        run_id: Option<u64>,
    },
    RunStarted {
        run_id: u64,
    },
    PaperIntent {
        market_id: String,
        side: PaperOrderSide,
        qty: f64,
        limit_px: f64,
    },
    PaperFill {
        market_id: String,
        side: PaperOrderSide,
        qty: f64,
        fill_px: f64,
    },
    RiskReject {
        market_id: String,
        reason: String,
        requested_qty: f64,
    },
    FeedHealth {
        mode: FeedMode,
        source_counts: Vec<SourceCount>,
    },
    PortfolioSnapshot {
        equity: f64,
        pnl: f64,
        position_qty: f64,
        fills: u64,
    },
    PriceSnapshot {
        coinbase_btc_usd: Option<f64>,
        binance_btc_usdt: Option<f64>,
        kraken_btc_usd: Option<f64>,
        polymarket_market_id: Option<String>,
        polymarket_yes_bid: Option<f64>,
        polymarket_yes_ask: Option<f64>,
        polymarket_yes_mid: Option<f64>,
        ts: u64,
    },
}

impl RuntimeEvent {
    pub fn connected() -> Self {
        Self::Connected { run_id: None }
    }

    pub fn run_started(run_id: u64) -> Self {
        Self::RunStarted { run_id }
    }

    pub fn paper_intent(
        market_id: impl Into<String>,
        side: PaperOrderSide,
        qty: f64,
        limit_px: f64,
    ) -> Self {
        Self::PaperIntent {
            market_id: market_id.into(),
            side,
            qty,
            limit_px,
        }
    }

    pub fn paper_fill(
        market_id: impl Into<String>,
        side: PaperOrderSide,
        qty: f64,
        fill_px: f64,
    ) -> Self {
        Self::PaperFill {
            market_id: market_id.into(),
            side,
            qty,
            fill_px,
        }
    }

    pub fn risk_reject(
        market_id: impl Into<String>,
        reason: impl Into<String>,
        requested_qty: f64,
    ) -> Self {
        Self::RiskReject {
            market_id: market_id.into(),
            reason: reason.into(),
            requested_qty,
        }
    }

    pub fn feed_health(mode: FeedMode, source_counts: Vec<SourceCount>) -> Self {
        Self::FeedHealth {
            mode,
            source_counts,
        }
    }

    pub fn portfolio_snapshot(summary: PortfolioSummary) -> Self {
        Self::PortfolioSnapshot {
            equity: summary.equity,
            pnl: summary.pnl,
            position_qty: summary.position_qty,
            fills: summary.fills,
        }
    }

    pub fn price_snapshot(snapshot: PriceSnapshot) -> Self {
        Self::PriceSnapshot {
            coinbase_btc_usd: snapshot.coinbase_btc_usd,
            binance_btc_usdt: snapshot.binance_btc_usdt,
            kraken_btc_usd: snapshot.kraken_btc_usd,
            polymarket_market_id: snapshot.polymarket_market_id,
            polymarket_yes_bid: snapshot.polymarket_yes_bid,
            polymarket_yes_ask: snapshot.polymarket_yes_ask,
            polymarket_yes_mid: snapshot.polymarket_yes_mid,
            ts: snapshot.ts,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    next_run_id: Arc<AtomicU64>,
    events_tx: broadcast::Sender<RuntimeEvent>,
    feed_mode: FeedMode,
    source_counts: Arc<RwLock<Vec<SourceCount>>>,
    discovered_markets: Arc<RwLock<Vec<DiscoveredMarket>>>,
    portfolio_summary: Arc<RwLock<PortfolioSummary>>,
    price_snapshot: Arc<RwLock<PriceSnapshot>>,
}

impl Default for AppState {
    fn default() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
            feed_mode: FeedMode::PaperLive,
            source_counts: Arc::new(RwLock::new(Vec::new())),
            discovered_markets: Arc::new(RwLock::new(Vec::new())),
            portfolio_summary: Arc::new(RwLock::new(PortfolioSummary::default())),
            price_snapshot: Arc::new(RwLock::new(PriceSnapshot::default())),
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(&self) -> Result<u64, StartRunError> {
        let previous = self
            .next_run_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| StartRunError::RunIdOverflow)?;

        Ok(previous + 1)
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.events_tx.subscribe()
    }

    pub fn publish_event(
        &self,
        event: RuntimeEvent,
    ) -> Result<usize, broadcast::error::SendError<RuntimeEvent>> {
        self.events_tx.send(event)
    }

    pub fn feed_health(&self) -> FeedHealthResponse {
        FeedHealthResponse {
            mode: self.feed_mode,
            source_counts: self
                .source_counts
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone(),
        }
    }

    pub fn discovered_markets(&self) -> DiscoveredMarketsResponse {
        DiscoveredMarketsResponse {
            markets: self
                .discovered_markets
                .read()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .clone(),
        }
    }

    pub fn portfolio_summary(&self) -> PortfolioSummary {
        *self
            .portfolio_summary
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn price_snapshot(&self) -> PriceSnapshot {
        self.price_snapshot
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn set_feed_source_counts(&self, source_counts: Vec<SourceCount>) {
        *self
            .source_counts
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = source_counts;
    }

    pub fn set_discovered_markets(&self, discovered_markets: Vec<DiscoveredMarket>) {
        *self
            .discovered_markets
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = discovered_markets;
    }

    pub fn set_portfolio_summary(&self, summary: PortfolioSummary) {
        *self
            .portfolio_summary
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = summary;
    }

    pub fn set_price_snapshot(&self, snapshot: PriceSnapshot) {
        *self
            .price_snapshot
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = snapshot;
    }

    #[cfg(test)]
    pub(crate) fn with_next_run_id_for_test(next_run_id: u64) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(next_run_id)),
            events_tx,
            feed_mode: FeedMode::PaperLive,
            source_counts: Arc::new(RwLock::new(Vec::new())),
            discovered_markets: Arc::new(RwLock::new(Vec::new())),
            portfolio_summary: Arc::new(RwLock::new(PortfolioSummary::default())),
            price_snapshot: Arc::new(RwLock::new(PriceSnapshot::default())),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_feed_mode_for_test(feed_mode: FeedMode) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
            feed_mode,
            source_counts: Arc::new(RwLock::new(Vec::new())),
            discovered_markets: Arc::new(RwLock::new(Vec::new())),
            portfolio_summary: Arc::new(RwLock::new(PortfolioSummary::default())),
            price_snapshot: Arc::new(RwLock::new(PriceSnapshot::default())),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_feed_data_for_test(
        feed_mode: FeedMode,
        source_counts: Vec<SourceCount>,
        discovered_markets: Vec<DiscoveredMarket>,
    ) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
            feed_mode,
            source_counts: Arc::new(RwLock::new(source_counts)),
            discovered_markets: Arc::new(RwLock::new(discovered_markets)),
            portfolio_summary: Arc::new(RwLock::new(PortfolioSummary::default())),
            price_snapshot: Arc::new(RwLock::new(PriceSnapshot::default())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{
        AppState, DiscoveredMarket, FeedMode, PortfolioSummary, PriceSnapshot, SourceCount,
    };

    #[test]
    fn start_run_returns_overflow_error_at_u64_max() {
        let state = AppState::new();
        state.next_run_id.store(u64::MAX, Ordering::Relaxed);

        assert!(state.start_run().is_err());
    }

    #[test]
    fn feed_health_returns_configured_mode() {
        let state = AppState::with_feed_mode_for_test(FeedMode::Sim);

        assert_eq!(state.feed_health().mode, FeedMode::Sim);
    }

    #[test]
    fn feed_health_and_discovered_markets_return_seeded_values() {
        let state = AppState::with_feed_data_for_test(
            FeedMode::PaperLive,
            vec![SourceCount {
                source: "polymarket".to_owned(),
                count: 5,
            }],
            vec![DiscoveredMarket {
                source: "polymarket".to_owned(),
                market_id: "btc-up-down".to_owned(),
            }],
        );

        assert_eq!(state.feed_health().source_counts.len(), 1);
        assert_eq!(state.feed_health().source_counts[0].source, "polymarket");
        assert_eq!(state.feed_health().source_counts[0].count, 5);
        assert_eq!(state.discovered_markets().markets.len(), 1);
        assert_eq!(
            state.discovered_markets().markets[0].market_id,
            "btc-up-down"
        );
    }

    #[test]
    fn test_setters_update_feed_snapshots() {
        let state = AppState::new();
        state.set_feed_source_counts(vec![SourceCount {
            source: "kalshi".to_owned(),
            count: 9,
        }]);
        state.set_discovered_markets(vec![DiscoveredMarket {
            source: "kalshi".to_owned(),
            market_id: "eth-up-down".to_owned(),
        }]);

        let feed_health = state.feed_health();
        let discovered = state.discovered_markets();

        assert_eq!(feed_health.source_counts[0].source, "kalshi");
        assert_eq!(feed_health.source_counts[0].count, 9);
        assert_eq!(discovered.markets[0].source, "kalshi");
        assert_eq!(discovered.markets[0].market_id, "eth-up-down");

        state.set_portfolio_summary(PortfolioSummary {
            equity: 12.4,
            pnl: 2.4,
            position_qty: 3.0,
            fills: 7,
        });
        let portfolio = state.portfolio_summary();
        assert_eq!(portfolio.equity, 12.4);
        assert_eq!(portfolio.pnl, 2.4);
        assert_eq!(portfolio.position_qty, 3.0);
        assert_eq!(portfolio.fills, 7);

        state.set_price_snapshot(PriceSnapshot {
            coinbase_btc_usd: Some(64_100.1),
            binance_btc_usdt: Some(64_099.8),
            kraken_btc_usd: Some(64_100.0),
            polymarket_market_id: Some("btc-up-down".to_owned()),
            polymarket_yes_bid: Some(0.49),
            polymarket_yes_ask: Some(0.51),
            polymarket_yes_mid: Some(0.5),
            ts: 10,
        });
        let snapshot = state.price_snapshot();
        assert_eq!(snapshot.coinbase_btc_usd, Some(64_100.1));
        assert_eq!(snapshot.binance_btc_usdt, Some(64_099.8));
        assert_eq!(snapshot.kraken_btc_usd, Some(64_100.0));
        assert_eq!(
            snapshot.polymarket_market_id.as_deref(),
            Some("btc-up-down")
        );
        assert_eq!(snapshot.polymarket_yes_bid, Some(0.49));
        assert_eq!(snapshot.polymarket_yes_ask, Some(0.51));
        assert_eq!(snapshot.polymarket_yes_mid, Some(0.5));
        assert_eq!(snapshot.ts, 10);
    }
}
