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

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct StrategyPerfSummary {
    pub execution_mode: String,
    pub lag_threshold_pct: f64,
    pub decision_p95_us: u64,
    pub intents_per_sec: u64,
    pub fills_per_sec: u64,
    pub lag_triggers: u64,
    pub halted: bool,
}

impl Default for StrategyPerfSummary {
    fn default() -> Self {
        Self {
            execution_mode: "paper".to_string(),
            lag_threshold_pct: 0.3,
            decision_p95_us: 0,
            intents_per_sec: 0,
            fills_per_sec: 0,
            lag_triggers: 0,
            halted: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Paper,
    Live,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Paper
    }
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RuntimeSettings {
    pub execution_mode: ExecutionMode,
    pub trading_paused: bool,
    pub lag_threshold_pct: f64,
    pub risk_per_trade_pct: f64,
    pub daily_loss_cap_pct: f64,
    pub market: String,
    pub forecast_horizon_minutes: u16,
    pub live_feature_enabled: bool,
}

impl Default for RuntimeSettings {
    fn default() -> Self {
        Self {
            execution_mode: ExecutionMode::Paper,
            trading_paused: false,
            lag_threshold_pct: 0.3,
            risk_per_trade_pct: 0.5,
            daily_loss_cap_pct: 2.0,
            market: "BTC/USD".to_string(),
            forecast_horizon_minutes: 15,
            live_feature_enabled: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize)]
pub struct RuntimeSettingsPatch {
    pub execution_mode: Option<ExecutionMode>,
    pub trading_paused: Option<bool>,
    pub lag_threshold_pct: Option<f64>,
    pub risk_per_trade_pct: Option<f64>,
    pub daily_loss_cap_pct: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct StrategyStatsSummary {
    pub balance: f64,
    pub total_pnl: f64,
    pub exec_latency_us: u64,
    pub win_rate: f64,
    pub btc_usd: f64,
}

impl Default for StrategyStatsSummary {
    fn default() -> Self {
        Self {
            balance: 0.0,
            total_pnl: 0.0,
            exec_latency_us: 0,
            win_rate: 0.0,
            btc_usd: 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize)]
pub struct BtcForecastSummary {
    pub horizon_minutes: u16,
    pub current_btc_usd: f64,
    pub forecast_btc_usd: f64,
    pub delta_pct: f64,
    pub ts: u64,
}

impl Default for BtcForecastSummary {
    fn default() -> Self {
        Self {
            horizon_minutes: 15,
            current_btc_usd: 0.0,
            forecast_btc_usd: 0.0,
            delta_pct: 0.0,
            ts: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
pub struct ExecutionLogEntry {
    pub ts: u64,
    pub event: String,
    pub headline: String,
    pub detail: String,
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
    StrategyPerf {
        execution_mode: String,
        lag_threshold_pct: f64,
        decision_p95_us: u64,
        intents_per_sec: u64,
        fills_per_sec: u64,
        lag_triggers: u64,
        halted: bool,
    },
    SettingsUpdated {
        execution_mode: ExecutionMode,
        trading_paused: bool,
        lag_threshold_pct: f64,
        risk_per_trade_pct: f64,
        daily_loss_cap_pct: f64,
    },
    StrategyStats {
        balance: f64,
        total_pnl: f64,
        exec_latency_us: u64,
        win_rate: f64,
        btc_usd: f64,
    },
    BtcForecast {
        horizon_minutes: u16,
        current_btc_usd: f64,
        forecast_btc_usd: f64,
        delta_pct: f64,
        ts: u64,
    },
    ExecutionLog {
        ts: u64,
        event: String,
        headline: String,
        detail: String,
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

    pub fn strategy_perf(summary: StrategyPerfSummary) -> Self {
        Self::StrategyPerf {
            execution_mode: summary.execution_mode,
            lag_threshold_pct: summary.lag_threshold_pct,
            decision_p95_us: summary.decision_p95_us,
            intents_per_sec: summary.intents_per_sec,
            fills_per_sec: summary.fills_per_sec,
            lag_triggers: summary.lag_triggers,
            halted: summary.halted,
        }
    }

    pub fn settings_updated(settings: RuntimeSettings) -> Self {
        Self::SettingsUpdated {
            execution_mode: settings.execution_mode,
            trading_paused: settings.trading_paused,
            lag_threshold_pct: settings.lag_threshold_pct,
            risk_per_trade_pct: settings.risk_per_trade_pct,
            daily_loss_cap_pct: settings.daily_loss_cap_pct,
        }
    }

    pub fn strategy_stats(summary: StrategyStatsSummary) -> Self {
        Self::StrategyStats {
            balance: summary.balance,
            total_pnl: summary.total_pnl,
            exec_latency_us: summary.exec_latency_us,
            win_rate: summary.win_rate,
            btc_usd: summary.btc_usd,
        }
    }

    pub fn btc_forecast(summary: BtcForecastSummary) -> Self {
        Self::BtcForecast {
            horizon_minutes: summary.horizon_minutes,
            current_btc_usd: summary.current_btc_usd,
            forecast_btc_usd: summary.forecast_btc_usd,
            delta_pct: summary.delta_pct,
            ts: summary.ts,
        }
    }

    pub fn execution_log(entry: ExecutionLogEntry) -> Self {
        Self::ExecutionLog {
            ts: entry.ts,
            event: entry.event,
            headline: entry.headline,
            detail: entry.detail,
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
    strategy_perf_summary: Arc<RwLock<StrategyPerfSummary>>,
    runtime_settings: Arc<RwLock<RuntimeSettings>>,
    strategy_stats_summary: Arc<RwLock<StrategyStatsSummary>>,
    btc_forecast_summary: Arc<RwLock<BtcForecastSummary>>,
    execution_logs: Arc<RwLock<Vec<ExecutionLogEntry>>>,
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
            strategy_perf_summary: Arc::new(RwLock::new(StrategyPerfSummary::default())),
            runtime_settings: Arc::new(RwLock::new(RuntimeSettings::default())),
            strategy_stats_summary: Arc::new(RwLock::new(StrategyStatsSummary::default())),
            btc_forecast_summary: Arc::new(RwLock::new(BtcForecastSummary::default())),
            execution_logs: Arc::new(RwLock::new(Vec::new())),
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

    pub fn strategy_perf_summary(&self) -> StrategyPerfSummary {
        self.strategy_perf_summary
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn runtime_settings(&self) -> RuntimeSettings {
        self.runtime_settings
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn strategy_stats_summary(&self) -> StrategyStatsSummary {
        *self
            .strategy_stats_summary
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn btc_forecast_summary(&self) -> BtcForecastSummary {
        *self
            .btc_forecast_summary
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn execution_logs(&self) -> Vec<ExecutionLogEntry> {
        self.execution_logs
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

    pub fn set_strategy_perf_summary(&self, summary: StrategyPerfSummary) {
        *self
            .strategy_perf_summary
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = summary;
    }

    pub fn set_runtime_settings(&self, settings: RuntimeSettings) {
        *self
            .runtime_settings
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = settings;
    }

    pub fn patch_runtime_settings(&self, patch: RuntimeSettingsPatch) -> RuntimeSettings {
        let mut guard = self
            .runtime_settings
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());

        if let Some(execution_mode) = patch.execution_mode {
            guard.execution_mode = execution_mode;
        }
        if let Some(trading_paused) = patch.trading_paused {
            guard.trading_paused = trading_paused;
        }
        if let Some(lag_threshold_pct) = patch.lag_threshold_pct {
            guard.lag_threshold_pct = lag_threshold_pct;
        }
        if let Some(risk_per_trade_pct) = patch.risk_per_trade_pct {
            guard.risk_per_trade_pct = risk_per_trade_pct;
        }
        if let Some(daily_loss_cap_pct) = patch.daily_loss_cap_pct {
            guard.daily_loss_cap_pct = daily_loss_cap_pct;
        }

        guard.clone()
    }

    pub fn set_strategy_stats_summary(&self, summary: StrategyStatsSummary) {
        *self
            .strategy_stats_summary
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = summary;
    }

    pub fn set_btc_forecast_summary(&self, summary: BtcForecastSummary) {
        *self
            .btc_forecast_summary
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = summary;
    }

    pub fn push_execution_log(&self, entry: ExecutionLogEntry, max_entries: usize) {
        let mut guard = self
            .execution_logs
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.push(entry);

        if guard.len() > max_entries {
            let overflow = guard.len() - max_entries;
            guard.drain(0..overflow);
        }
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
            strategy_perf_summary: Arc::new(RwLock::new(StrategyPerfSummary::default())),
            runtime_settings: Arc::new(RwLock::new(RuntimeSettings::default())),
            strategy_stats_summary: Arc::new(RwLock::new(StrategyStatsSummary::default())),
            btc_forecast_summary: Arc::new(RwLock::new(BtcForecastSummary::default())),
            execution_logs: Arc::new(RwLock::new(Vec::new())),
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
            strategy_perf_summary: Arc::new(RwLock::new(StrategyPerfSummary::default())),
            runtime_settings: Arc::new(RwLock::new(RuntimeSettings::default())),
            strategy_stats_summary: Arc::new(RwLock::new(StrategyStatsSummary::default())),
            btc_forecast_summary: Arc::new(RwLock::new(BtcForecastSummary::default())),
            execution_logs: Arc::new(RwLock::new(Vec::new())),
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
            strategy_perf_summary: Arc::new(RwLock::new(StrategyPerfSummary::default())),
            runtime_settings: Arc::new(RwLock::new(RuntimeSettings::default())),
            strategy_stats_summary: Arc::new(RwLock::new(StrategyStatsSummary::default())),
            btc_forecast_summary: Arc::new(RwLock::new(BtcForecastSummary::default())),
            execution_logs: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{
        AppState, BtcForecastSummary, DiscoveredMarket, ExecutionLogEntry, FeedMode,
        PortfolioSummary, PriceSnapshot, RuntimeSettingsPatch, SourceCount, StrategyPerfSummary,
        StrategyStatsSummary,
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

        state.set_strategy_perf_summary(StrategyPerfSummary {
            execution_mode: "paper".to_owned(),
            lag_threshold_pct: 0.3,
            decision_p95_us: 88,
            intents_per_sec: 1100,
            fills_per_sec: 700,
            lag_triggers: 10,
            halted: false,
        });
        let perf = state.strategy_perf_summary();
        assert_eq!(perf.execution_mode, "paper");
        assert_eq!(perf.lag_threshold_pct, 0.3);
        assert_eq!(perf.decision_p95_us, 88);
        assert_eq!(perf.intents_per_sec, 1100);
        assert_eq!(perf.fills_per_sec, 700);
        assert_eq!(perf.lag_triggers, 10);
        assert!(!perf.halted);

        let patched = state.patch_runtime_settings(RuntimeSettingsPatch {
            trading_paused: Some(true),
            lag_threshold_pct: Some(0.44),
            risk_per_trade_pct: Some(0.7),
            daily_loss_cap_pct: Some(2.8),
            ..RuntimeSettingsPatch::default()
        });
        assert!(patched.trading_paused);
        assert_eq!(patched.lag_threshold_pct, 0.44);
        assert_eq!(patched.risk_per_trade_pct, 0.7);
        assert_eq!(patched.daily_loss_cap_pct, 2.8);

        state.set_strategy_stats_summary(StrategyStatsSummary {
            balance: 10_100.0,
            total_pnl: 100.0,
            exec_latency_us: 77,
            win_rate: 60.0,
            btc_usd: 66_000.0,
        });
        assert_eq!(state.strategy_stats_summary().balance, 10_100.0);

        state.set_btc_forecast_summary(BtcForecastSummary {
            horizon_minutes: 15,
            current_btc_usd: 66_000.0,
            forecast_btc_usd: 66_120.0,
            delta_pct: 0.18,
            ts: 12,
        });
        assert_eq!(state.btc_forecast_summary().horizon_minutes, 15);

        state.push_execution_log(
            ExecutionLogEntry {
                ts: 12,
                event: "paper_fill".to_string(),
                headline: "Filled BUY".to_string(),
                detail: "qty 1 @ 0.51".to_string(),
            },
            128,
        );
        assert_eq!(state.execution_logs().len(), 1);
    }
}
