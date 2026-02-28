mod config;
mod predictors;
mod wiring;

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use api::state::{
    AppState, BtcForecastSummary, DiscoveredMarket, ExecutionLogEntry,
    ExecutionMode as StateExecutionMode, FeedMode, PaperOrderSide, PortfolioSummary, PriceSnapshot,
    RuntimeEvent, RuntimeSettings, SourceCount, StrategyPerfSummary, StrategyStatsSummary,
};
use config::ExecutionMode as ConfigExecutionMode;
use reqwest::Client;
use runtime::events::RuntimeStage;
use runtime::live::{
    fuse_predictors, BtcMedianTick, PolymarketQuoteTick, PredictorTick, RawPolymarketQuote,
};
use runtime::live_runner::{run_paper_live_once_with_lag, JoinedLiveInputs};
use runtime::logging::{PaperJournalRow, PaperJournalRowKind};
use runtime::replay::ReplayCsvWriter;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::time::{self, Duration, MissedTickBehavior};

const BOOTSTRAP_ROWS_ENV: &str = "LAB_SERVER_INITIAL_PAPER_JOURNAL_ROWS";
const PAPER_MARKET_ID: &str = "btc-15m-forecast";
const PAPER_ORDER_QTY: f64 = 1.0;
const LIVE_LOOP_INTERVAL_MS: u64 = 1500;
const POLY_REFRESH_EVERY_TICKS: u64 = 10;
const MAX_TRACKED_POLY_MARKETS: usize = 3;
const BTC_COINBASE_URL: &str = "https://api.coinbase.com/v2/prices/BTC-USD/spot";
const BTC_BINANCE_URL: &str = "https://api.binance.com/api/v3/ticker/price?symbol=BTCUSDT";
const BTC_KRAKEN_URL: &str = "https://api.kraken.com/0/public/Ticker?pair=XBTUSD";
const POLY_GAMMA_MARKETS_URL: &str =
    "https://gamma-api.polymarket.com/markets?active=true&closed=false&limit=200";
const BTC_MOMENTUM_MULTIPLIER: f64 = 60.0;
const SPREAD_SIGNAL_TO_YES_COEFF: f64 = 0.00001;
const DEFAULT_STARTING_EQUITY: f64 = 10_000.0;

#[derive(Debug, Clone, Copy)]
struct RuntimeTradingConfig {
    live_feature_enabled: bool,
    starting_equity: f64,
}

#[derive(Debug, Default, Clone, Copy)]
struct TradeOutcomeTracker {
    open_qty: f64,
    avg_entry: f64,
    winning_closes: u64,
    losing_closes: u64,
}

impl TradeOutcomeTracker {
    fn apply_fill(&mut self, side: PaperOrderSide, fill_px: f64, qty: f64) {
        let signed_qty = match side {
            PaperOrderSide::Buy => qty,
            PaperOrderSide::Sell => -qty,
        };

        if self.open_qty == 0.0 || self.open_qty.signum() == signed_qty.signum() {
            let total_qty = self.open_qty.abs() + signed_qty.abs();
            if total_qty > 0.0 {
                let weighted_cost =
                    (self.avg_entry * self.open_qty.abs()) + (fill_px * signed_qty.abs());
                self.avg_entry = weighted_cost / total_qty;
            }
            self.open_qty += signed_qty;
            return;
        }

        let close_qty = self.open_qty.abs().min(signed_qty.abs());
        let realized = if self.open_qty > 0.0 {
            (fill_px - self.avg_entry) * close_qty
        } else {
            (self.avg_entry - fill_px) * close_qty
        };

        if realized > 0.0 {
            self.winning_closes = self.winning_closes.saturating_add(1);
        } else if realized < 0.0 {
            self.losing_closes = self.losing_closes.saturating_add(1);
        }

        self.open_qty += signed_qty;
        if self.open_qty == 0.0 {
            self.avg_entry = 0.0;
        } else if self.open_qty.signum() == signed_qty.signum() && signed_qty.abs() > close_qty {
            self.avg_entry = fill_px;
        }
    }

    fn win_rate_pct(self) -> f64 {
        let total = self.winning_closes + self.losing_closes;
        if total == 0 {
            return 0.0;
        }

        (self.winning_closes as f64 / total as f64) * 100.0
    }
}

#[derive(Default)]
struct SourceCounters {
    coinbase: u64,
    binance: u64,
    kraken: u64,
    polymarket: u64,
}

impl SourceCounters {
    fn as_source_counts(&self) -> Vec<SourceCount> {
        vec![
            SourceCount {
                source: "coinbase".to_string(),
                count: self.coinbase,
            },
            SourceCount {
                source: "binance".to_string(),
                count: self.binance,
            },
            SourceCount {
                source: "kraken".to_string(),
                count: self.kraken,
            },
            SourceCount {
                source: "polymarket".to_string(),
                count: self.polymarket,
            },
        ]
    }
}

#[derive(Debug, Deserialize)]
struct CoinbaseSpotResponse {
    data: CoinbaseSpotData,
}

#[derive(Debug, Deserialize)]
struct CoinbaseSpotData {
    amount: String,
}

#[derive(Debug, Deserialize)]
struct BinanceTickerResponse {
    price: String,
}

#[derive(Debug, Deserialize)]
struct GammaMarket {
    slug: String,
    #[serde(default)]
    question: String,
    #[serde(rename = "bestBid", default)]
    best_bid: Option<serde_json::Value>,
    #[serde(rename = "bestAsk", default)]
    best_ask: Option<serde_json::Value>,
    #[serde(rename = "outcomePrices", default)]
    outcome_prices_raw: Option<serde_json::Value>,
    #[serde(default)]
    outcomes_raw: Option<serde_json::Value>,
}

struct PolymarketSnapshot {
    discovered: Vec<DiscoveredMarket>,
    quotes: Vec<PolymarketQuoteTick>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config::Config {
        listen_addr,
        mode,
        replay_output_path,
        execution_mode,
        live_feature_enabled,
        lag_threshold_pct,
        per_trade_risk_pct,
        daily_loss_cap_pct,
    } = config::Config::from_env()?;

    let runtime_trading_config = RuntimeTradingConfig {
        live_feature_enabled,
        starting_equity: DEFAULT_STARTING_EQUITY,
    };

    println!("{}", startup_mode_banner(mode));
    initialize_replay_output(&replay_output_path)?;
    let app_state = AppState::new();
    app_state.set_runtime_settings(RuntimeSettings {
        execution_mode: to_state_execution_mode(execution_mode),
        trading_paused: false,
        lag_threshold_pct,
        risk_per_trade_pct: per_trade_risk_pct,
        daily_loss_cap_pct,
        market: "BTC/USD".to_string(),
        forecast_horizon_minutes: 15,
        live_feature_enabled,
    });

    if mode == config::RunMode::PaperLive {
        let client = Client::builder()
            .user_agent("market-latency-risk-lab/paper-live")
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()?;
        tokio::spawn(run_paper_live_loop(
            app_state.clone(),
            client,
            runtime_trading_config,
        ));
    }

    let listener = TcpListener::bind(listen_addr).await?;
    axum::serve(listener, wiring::build_app_with_state(app_state)).await?;
    Ok(())
}

async fn run_paper_live_loop(state: AppState, client: Client, runtime_cfg: RuntimeTradingConfig) {
    let mut interval = time::interval(Duration::from_millis(LIVE_LOOP_INTERVAL_MS));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut tick = 0_u64;
    let mut counters = SourceCounters::default();
    let mut last_btc_median: Option<f64> = None;
    let mut tracked_quotes: Vec<PolymarketQuoteTick> = Vec::new();

    let mut cash = runtime_cfg.starting_equity;
    let mut position_qty = 0.0_f64;
    let mut fills = 0_u64;
    let mut outcomes = TradeOutcomeTracker::default();
    let mut last_pause_state = false;

    state.set_discovered_markets(vec![DiscoveredMarket {
        source: "polymarket".to_string(),
        market_id: PAPER_MARKET_ID.to_string(),
    }]);

    loop {
        interval.tick().await;
        tick = tick.saturating_add(1);
        let mut tick_intents = 0_u64;
        let mut tick_fills = 0_u64;
        let mut tick_lag_triggers = 0_u64;

        let (coinbase_px, binance_px, kraken_px) = tokio::join!(
            fetch_coinbase_btc_usd(&client),
            fetch_binance_btc_usdt(&client),
            fetch_kraken_btc_usd(&client),
        );

        let mut btc_samples = Vec::new();
        if let Some(px) = coinbase_px {
            counters.coinbase = counters.coinbase.saturating_add(1);
            btc_samples.push(px);
        }
        if let Some(px) = binance_px {
            counters.binance = counters.binance.saturating_add(1);
            btc_samples.push(px);
        }
        if let Some(px) = kraken_px {
            counters.kraken = counters.kraken.saturating_add(1);
            btc_samples.push(px);
        }

        let btc_median = median_f64(&btc_samples)
            .or(last_btc_median)
            .unwrap_or(64_000.0);
        let spread_signal = match last_btc_median {
            Some(previous) if previous > 0.0 => {
                ((btc_median - previous) / previous) * 10_000.0 * BTC_MOMENTUM_MULTIPLIER
            }
            _ => 0.0,
        };
        last_btc_median = Some(btc_median);

        let settings = state.runtime_settings();
        let (forecast_btc_usd, forecast_delta_pct) = forecast_btc_15m(btc_median, spread_signal);
        let forecast_summary = BtcForecastSummary {
            horizon_minutes: 15,
            current_btc_usd: btc_median,
            forecast_btc_usd,
            delta_pct: forecast_delta_pct,
            ts: tick,
        };
        state.set_btc_forecast_summary(forecast_summary);
        let _ = state.publish_event(RuntimeEvent::btc_forecast(forecast_summary));

        if tick == 1 || tick % POLY_REFRESH_EVERY_TICKS == 0 || tracked_quotes.is_empty() {
            if let Some(snapshot) = fetch_polymarket_snapshot(&client, tick).await {
                if !snapshot.quotes.is_empty() {
                    counters.polymarket = counters.polymarket.saturating_add(1);
                    tracked_quotes = snapshot.quotes;
                    state.set_discovered_markets(snapshot.discovered);
                }
            }
        }

        if tracked_quotes.is_empty() {
            tracked_quotes.push(PolymarketQuoteTick {
                market_slug: PAPER_MARKET_ID.to_string(),
                best_yes_bid: 0.48,
                best_yes_ask: 0.52,
                mid_yes: 0.50,
                ts: tick,
            });
        }

        let primary_quote = tracked_quotes.first();
        let price_snapshot = PriceSnapshot {
            coinbase_btc_usd: coinbase_px,
            binance_btc_usdt: binance_px,
            kraken_btc_usd: kraken_px,
            polymarket_market_id: primary_quote.map(|quote| quote.market_slug.clone()),
            polymarket_yes_bid: primary_quote.map(|quote| quote.best_yes_bid),
            polymarket_yes_ask: primary_quote.map(|quote| quote.best_yes_ask),
            polymarket_yes_mid: primary_quote.map(|quote| quote.mid_yes),
            ts: tick,
        };
        state.set_price_snapshot(price_snapshot.clone());
        let _ = state.publish_event(RuntimeEvent::price_snapshot(price_snapshot));

        let predictor_now_ms = now_unix_ms();
        let (tradingview_predictor, cryptoquant_predictor) = tokio::join!(
            fetch_tradingview_predictor(&client, predictor_now_ms),
            fetch_cryptoquant_predictor(&client, predictor_now_ms),
        );
        let predictor_ticks: Vec<PredictorTick> = [tradingview_predictor, cryptoquant_predictor]
            .into_iter()
            .flatten()
            .collect();
        let fused_fair_yes = fuse_predictors(&predictor_ticks, predictor_now_ms)
            .ok()
            .map(|fused| fused.fair_yes_px);

        let source_counts = counters.as_source_counts();
        state.set_feed_source_counts(source_counts.clone());
        let _ = state.publish_event(RuntimeEvent::feed_health(
            FeedMode::PaperLive,
            source_counts,
        ));

        let current_mark = tracked_quotes
            .first()
            .map(|quote| quote.mid_yes)
            .unwrap_or(0.5);
        let equity_before = cash + (position_qty * current_mark);
        let pnl_before = equity_before - runtime_cfg.starting_equity;
        let daily_loss_limit = runtime_cfg.starting_equity * (settings.daily_loss_cap_pct / 100.0);
        let daily_halted = pnl_before <= -daily_loss_limit;

        let decision_started = Instant::now();

        if settings.trading_paused != last_pause_state {
            let status = if settings.trading_paused {
                "Trading Paused"
            } else {
                "Trading Resumed"
            };
            let log = ExecutionLogEntry {
                ts: tick,
                event: "pause_state".to_string(),
                headline: status.to_string(),
                detail: format!("execution_mode={:?}", settings.execution_mode),
            };
            state.push_execution_log(log.clone(), 500);
            let _ = state.publish_event(RuntimeEvent::execution_log(log));
            last_pause_state = settings.trading_paused;
        }

        for quote in tracked_quotes.iter().take(MAX_TRACKED_POLY_MARKETS) {
            if settings.trading_paused {
                continue;
            }

            if daily_halted {
                let _ = state.publish_event(RuntimeEvent::risk_reject(
                    &quote.market_slug,
                    "daily loss cap reached",
                    PAPER_ORDER_QTY,
                ));
                let log = ExecutionLogEntry {
                    ts: tick,
                    event: "risk_reject".to_string(),
                    headline: "Daily Cap Halt".to_string(),
                    detail: format!("{} qty={}", quote.market_slug, PAPER_ORDER_QTY),
                };
                state.push_execution_log(log.clone(), 500);
                let _ = state.publish_event(RuntimeEvent::execution_log(log));
                continue;
            }

            let joined = JoinedLiveInputs {
                btc_tick: BtcMedianTick::new(
                    btc_median,
                    spread_signal,
                    btc_samples.len() as u32,
                    tick,
                ),
                quote_tick: quote.clone(),
            };

            let fair_yes_px = fused_fair_yes
                .unwrap_or_else(|| fallback_fair_yes_from_spread(quote.mid_yes, spread_signal));

            let runtime_events = run_paper_live_once_with_lag(
                tick,
                &joined,
                fair_yes_px,
                settings.lag_threshold_pct,
                settings.risk_per_trade_pct / 100.0,
                runtime_cfg.starting_equity,
                settings.daily_loss_cap_pct / 100.0,
            );
            let has_intent = runtime_events
                .iter()
                .any(|event| event.stage == RuntimeStage::PaperIntentCreated);
            if !has_intent {
                continue;
            }
            tick_intents = tick_intents.saturating_add(1);
            tick_lag_triggers = tick_lag_triggers.saturating_add(1);

            let side = if fair_yes_px >= quote.mid_yes {
                PaperOrderSide::Buy
            } else {
                PaperOrderSide::Sell
            };
            let limit_px = if matches!(side, PaperOrderSide::Buy) {
                quote.best_yes_ask
            } else {
                quote.best_yes_bid
            };
            let _ = state.publish_event(RuntimeEvent::paper_intent(
                &quote.market_slug,
                side,
                PAPER_ORDER_QTY,
                limit_px,
            ));
            let intent_log = ExecutionLogEntry {
                ts: tick,
                event: "paper_intent".to_string(),
                headline: format!("Intent {:?}", side),
                detail: format!(
                    "{} qty={} @ {:.4}",
                    quote.market_slug, PAPER_ORDER_QTY, limit_px
                ),
            };
            state.push_execution_log(intent_log.clone(), 500);
            let _ = state.publish_event(RuntimeEvent::execution_log(intent_log));

            let has_fill = runtime_events
                .iter()
                .any(|event| event.stage == RuntimeStage::PaperFillRecorded);
            if has_fill {
                if settings.execution_mode == StateExecutionMode::Live
                    && !runtime_cfg.live_feature_enabled
                {
                    let _ = state.publish_event(RuntimeEvent::risk_reject(
                        &quote.market_slug,
                        "live mode disabled by feature flag",
                        PAPER_ORDER_QTY,
                    ));
                    let log = ExecutionLogEntry {
                        ts: tick,
                        event: "risk_reject".to_string(),
                        headline: "Live Mode Blocked".to_string(),
                        detail: "Enable LAB_LIVE_FEATURE_ENABLED to allow live mode".to_string(),
                    };
                    state.push_execution_log(log.clone(), 500);
                    let _ = state.publish_event(RuntimeEvent::execution_log(log));
                    continue;
                }

                let fill_px = if matches!(side, PaperOrderSide::Buy) {
                    quote.best_yes_ask
                } else {
                    quote.best_yes_bid
                };

                if matches!(side, PaperOrderSide::Buy) {
                    cash -= fill_px * PAPER_ORDER_QTY;
                    position_qty += PAPER_ORDER_QTY;
                } else {
                    cash += fill_px * PAPER_ORDER_QTY;
                    position_qty -= PAPER_ORDER_QTY;
                }
                fills = fills.saturating_add(1);
                tick_fills = tick_fills.saturating_add(1);
                outcomes.apply_fill(side, fill_px, PAPER_ORDER_QTY);

                let _ = state.publish_event(RuntimeEvent::paper_fill(
                    &quote.market_slug,
                    side,
                    PAPER_ORDER_QTY,
                    fill_px,
                ));
                let fill_log = ExecutionLogEntry {
                    ts: tick,
                    event: "paper_fill".to_string(),
                    headline: format!("Filled {:?}", side),
                    detail: format!(
                        "{} qty={} @ {:.4}",
                        quote.market_slug, PAPER_ORDER_QTY, fill_px
                    ),
                };
                state.push_execution_log(fill_log.clone(), 500);
                let _ = state.publish_event(RuntimeEvent::execution_log(fill_log));
            } else {
                let _ = state.publish_event(RuntimeEvent::risk_reject(
                    &quote.market_slug,
                    "risk gate rejected",
                    PAPER_ORDER_QTY,
                ));
                let reject_log = ExecutionLogEntry {
                    ts: tick,
                    event: "risk_reject".to_string(),
                    headline: "Risk Rejected".to_string(),
                    detail: format!("{} qty={}", quote.market_slug, PAPER_ORDER_QTY),
                };
                state.push_execution_log(reject_log.clone(), 500);
                let _ = state.publish_event(RuntimeEvent::execution_log(reject_log));
            }
        }

        let throughput_scale = 1000.0 / (LIVE_LOOP_INTERVAL_MS as f64);
        let perf_summary = StrategyPerfSummary {
            execution_mode: match settings.execution_mode {
                StateExecutionMode::Paper => "paper".to_string(),
                StateExecutionMode::Live => "live".to_string(),
            },
            lag_threshold_pct: settings.lag_threshold_pct,
            decision_p95_us: decision_started.elapsed().as_micros() as u64,
            intents_per_sec: ((tick_intents as f64) * throughput_scale).round() as u64,
            fills_per_sec: ((tick_fills as f64) * throughput_scale).round() as u64,
            lag_triggers: tick_lag_triggers,
            halted: daily_halted,
        };
        state.set_strategy_perf_summary(perf_summary.clone());
        let _ = state.publish_event(RuntimeEvent::strategy_perf(perf_summary));

        let mark_price = tracked_quotes
            .first()
            .map(|quote| quote.mid_yes)
            .unwrap_or(0.5);
        let equity = cash + (position_qty * mark_price);
        let summary = PortfolioSummary {
            equity,
            pnl: equity - runtime_cfg.starting_equity,
            position_qty,
            fills,
        };

        let stats_summary = StrategyStatsSummary {
            balance: equity,
            total_pnl: summary.pnl,
            exec_latency_us: decision_started.elapsed().as_micros() as u64,
            win_rate: outcomes.win_rate_pct(),
            btc_usd: btc_median,
        };
        state.set_strategy_stats_summary(stats_summary);
        let _ = state.publish_event(RuntimeEvent::strategy_stats(stats_summary));

        state.set_portfolio_summary(summary);
        let _ = state.publish_event(RuntimeEvent::portfolio_snapshot(summary));
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0)
}

fn fallback_fair_yes_from_spread(poly_mid_yes: f64, spread_signal: f64) -> f64 {
    (poly_mid_yes + (spread_signal * SPREAD_SIGNAL_TO_YES_COEFF)).clamp(0.0, 1.0)
}

fn forecast_btc_15m(current_btc_usd: f64, spread_signal: f64) -> (f64, f64) {
    let immediate_bps = spread_signal / BTC_MOMENTUM_MULTIPLIER;
    let projected_pct = ((immediate_bps * 15.0) / 10_000.0).clamp(-0.01, 0.01);
    let forecast = current_btc_usd * (1.0 + projected_pct);
    (forecast, projected_pct * 100.0)
}

fn to_state_execution_mode(mode: ConfigExecutionMode) -> StateExecutionMode {
    match mode {
        ConfigExecutionMode::Paper => StateExecutionMode::Paper,
        ConfigExecutionMode::Live => StateExecutionMode::Live,
    }
}

async fn fetch_tradingview_predictor(client: &Client, ts_ms: u64) -> Option<PredictorTick> {
    let url = env::var("LAB_TRADINGVIEW_PREDICT_URL").ok()?;
    if url.trim().is_empty() {
        return None;
    }

    let payload = client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;

    predictors::parse_tradingview_payload(&payload, ts_ms).ok()
}

async fn fetch_cryptoquant_predictor(client: &Client, ts_ms: u64) -> Option<PredictorTick> {
    let url = env::var("LAB_CRYPTOQUANT_PREDICT_URL").ok()?;
    if url.trim().is_empty() {
        return None;
    }

    let payload = client
        .get(url)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .text()
        .await
        .ok()?;

    predictors::parse_cryptoquant_payload(&payload, ts_ms).ok()
}

async fn fetch_coinbase_btc_usd(client: &Client) -> Option<f64> {
    let response = client
        .get(BTC_COINBASE_URL)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let payload: CoinbaseSpotResponse = response.json().await.ok()?;
    parse_positive_f64(&payload.data.amount)
}

async fn fetch_binance_btc_usdt(client: &Client) -> Option<f64> {
    let response = client
        .get(BTC_BINANCE_URL)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let payload: BinanceTickerResponse = response.json().await.ok()?;
    parse_positive_f64(&payload.price)
}

async fn fetch_kraken_btc_usd(client: &Client) -> Option<f64> {
    let response = client
        .get(BTC_KRAKEN_URL)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let payload: serde_json::Value = response.json().await.ok()?;
    let result = payload.get("result")?.as_object()?;
    let first = result.values().next()?;
    let close = first.get("c")?.as_array()?.first()?.as_str()?;
    parse_positive_f64(close)
}

async fn fetch_polymarket_snapshot(client: &Client, tick: u64) -> Option<PolymarketSnapshot> {
    let response = client
        .get(POLY_GAMMA_MARKETS_URL)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?;
    let markets: Vec<GammaMarket> = response.json().await.ok()?;

    let mut discovered = Vec::new();
    let mut quotes = Vec::new();

    for market in markets.iter() {
        if !is_btc_15m_market(&market.slug, &market.question) {
            continue;
        }

        if let Some(quote) = gamma_market_to_quote(market, tick) {
            discovered.push(DiscoveredMarket {
                source: "polymarket".to_string(),
                market_id: market.slug.clone(),
            });
            quotes.push(quote);
        }

        if quotes.len() >= MAX_TRACKED_POLY_MARKETS {
            break;
        }
    }

    if quotes.is_empty() {
        return None;
    }

    Some(PolymarketSnapshot { discovered, quotes })
}

fn is_btc_15m_market(slug: &str, question: &str) -> bool {
    let haystack = format!(
        "{} {}",
        slug.to_ascii_lowercase(),
        question.to_ascii_lowercase()
    );

    let has_btc = haystack.contains("btc") || haystack.contains("bitcoin");
    if !has_btc {
        return false;
    }

    const FIFTEEN_MINUTE_TOKENS: [&str; 8] = [
        "15m",
        "15-min",
        "15 min",
        "15 minute",
        "15-minute",
        "15 minutes",
        "next 15",
        "in 15",
    ];

    FIFTEEN_MINUTE_TOKENS
        .iter()
        .any(|token| haystack.contains(token))
}

fn gamma_market_to_quote(market: &GammaMarket, tick: u64) -> Option<PolymarketQuoteTick> {
    let fallback_mid = match (
        market.best_bid.as_ref().and_then(parse_probability_json),
        market.best_ask.as_ref().and_then(parse_probability_json),
    ) {
        (Some(best_bid), Some(best_ask)) => (best_bid + best_ask) / 2.0,
        _ => 0.5,
    };
    let yes_mid = yes_price_from_market(market).unwrap_or(fallback_mid.clamp(0.0, 1.0));
    let fallback_bid = (yes_mid - 0.01).clamp(0.0, 1.0);
    let fallback_ask = (yes_mid + 0.01).clamp(0.0, 1.0);
    let mut best_bid = market
        .best_bid
        .as_ref()
        .and_then(parse_probability_json)
        .unwrap_or(fallback_bid);
    let mut best_ask = market
        .best_ask
        .as_ref()
        .and_then(parse_probability_json)
        .unwrap_or(fallback_ask);

    if best_bid > best_ask {
        std::mem::swap(&mut best_bid, &mut best_ask);
    }

    RawPolymarketQuote {
        market_slug: market.slug.clone(),
        best_yes_bid: best_bid,
        best_yes_ask: best_ask,
        ts: tick,
    }
    .normalize()
    .ok()
}

fn yes_price_from_market(market: &GammaMarket) -> Option<f64> {
    let outcomes = parse_string_list(market.outcomes_raw.as_ref());
    let outcome_prices = parse_string_list(market.outcome_prices_raw.as_ref());

    if !outcomes.is_empty() && outcomes.len() == outcome_prices.len() {
        for (idx, outcome) in outcomes.iter().enumerate() {
            if outcome.eq_ignore_ascii_case("yes") {
                return parse_probability_str(&outcome_prices[idx]);
            }
        }
    }

    outcome_prices
        .first()
        .and_then(|value| parse_probability_str(value))
}

fn parse_string_list(value: Option<&serde_json::Value>) -> Vec<String> {
    let Some(value) = value else {
        return Vec::new();
    };

    match value {
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str().map(ToOwned::to_owned))
            .collect(),
        serde_json::Value::String(text) => {
            if let Ok(items) = serde_json::from_str::<Vec<String>>(text) {
                return items;
            }

            text.split(',')
                .map(str::trim)
                .map(|entry| entry.trim_matches(|ch| ch == '[' || ch == ']' || ch == '"'))
                .filter(|entry| !entry.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        }
        _ => Vec::new(),
    }
}

fn parse_positive_f64(value: &str) -> Option<f64> {
    let parsed = value.parse::<f64>().ok()?;
    if parsed.is_finite() && parsed > 0.0 {
        Some(parsed)
    } else {
        None
    }
}

fn parse_probability_str(value: &str) -> Option<f64> {
    let parsed = value.parse::<f64>().ok()?;
    parse_probability(parsed)
}

fn parse_probability_json(value: &serde_json::Value) -> Option<f64> {
    match value {
        serde_json::Value::Number(number) => parse_probability(number.as_f64()?),
        serde_json::Value::String(text) => parse_probability_str(text),
        _ => None,
    }
}

fn parse_probability(value: f64) -> Option<f64> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Some(value)
    } else {
        None
    }
}

fn median_f64(values: &[f64]) -> Option<f64> {
    let mut sorted = values
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value > 0.0)
        .collect::<Vec<_>>();
    if sorted.is_empty() {
        return None;
    }

    sorted.sort_by(f64::total_cmp);
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

fn startup_mode_banner(mode: config::RunMode) -> String {
    format!("lab-server startup mode: {}", mode.as_str())
}

fn initialize_replay_output(path: &str) -> Result<(), std::io::Error> {
    let replay_path = Path::new(path);

    if let Some(parent) = replay_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }

    let replay_file = File::create(replay_path)?;
    let mut replay_writer = ReplayCsvWriter::new(replay_file);
    replay_writer.write_header()?;
    replay_writer.append_paper_journal_rows(&initial_paper_journal_rows())?;
    Ok(())
}

fn initial_paper_journal_rows() -> Vec<PaperJournalRow> {
    let Ok(value) = env::var(BOOTSTRAP_ROWS_ENV) else {
        return Vec::new();
    };

    value
        .split(';')
        .filter_map(parse_bootstrap_paper_journal_row)
        .collect()
}

fn parse_bootstrap_paper_journal_row(value: &str) -> Option<PaperJournalRow> {
    let mut parts = value.splitn(3, '|');
    let tick = parts.next()?.trim().parse::<u64>().ok()?;
    let kind = match parts.next()?.trim() {
        "paper_fill" => PaperJournalRowKind::PaperFill,
        _ => return None,
    };
    let action_detail = parts.next()?.trim();
    if action_detail.is_empty() {
        return None;
    }

    Some(PaperJournalRow {
        tick,
        kind,
        action_detail: action_detail.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::RunMode;
    use runtime::logging::PaperJournalRowKind;
    use runtime::replay::REPLAY_CSV_HEADER;

    use super::{
        initial_paper_journal_rows, initialize_replay_output, is_btc_15m_market, median_f64,
        parse_probability_str, startup_mode_banner,
    };

    static ENV_LOCK: Mutex<()> = Mutex::new(());
    const ENV_BOOTSTRAP_ROWS: &str = "LAB_SERVER_INITIAL_PAPER_JOURNAL_ROWS";

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = env::var_os(key);
            env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = env::var_os(key);
            env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.take() {
                Some(value) => env::set_var(self.key, value),
                None => env::remove_var(self.key),
            }
        }
    }

    #[test]
    fn initialize_replay_output_creates_parent_dir_and_writes_csv_header() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _bootstrap_guard = EnvVarGuard::unset(ENV_BOOTSTRAP_ROWS);
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lab-server-replay-{unique}"));
        let replay_path = root.join("nested").join("replay.csv");

        initialize_replay_output(replay_path.to_str().unwrap())
            .expect("startup should initialize replay output");

        let actual = fs::read_to_string(&replay_path).expect("replay output file should exist");
        assert_eq!(actual, REPLAY_CSV_HEADER);

        fs::remove_dir_all(&root).expect("temp replay directory should be removable");
    }

    #[test]
    fn startup_mode_banner_reports_selected_mode() {
        assert_eq!(
            startup_mode_banner(RunMode::PaperLive),
            "lab-server startup mode: paper-live"
        );
        assert_eq!(
            startup_mode_banner(RunMode::Sim),
            "lab-server startup mode: sim"
        );
    }

    #[test]
    fn initial_paper_journal_rows_is_empty_without_bootstrap_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::unset(ENV_BOOTSTRAP_ROWS);

        let rows = initial_paper_journal_rows();

        assert!(rows.is_empty());
    }

    #[test]
    fn initial_paper_journal_rows_reads_bootstrap_rows_from_env() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::set(
            ENV_BOOTSTRAP_ROWS,
            "17|paper_fill|buy:market-1@0.62x5;18|paper_fill|sell:market-2@0.41x2",
        );

        let rows = initial_paper_journal_rows();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].tick, 17);
        assert_eq!(rows[0].kind, PaperJournalRowKind::PaperFill);
        assert_eq!(rows[0].action_detail, "buy:market-1@0.62x5");
        assert_eq!(rows[1].tick, 18);
        assert_eq!(rows[1].kind, PaperJournalRowKind::PaperFill);
        assert_eq!(rows[1].action_detail, "sell:market-2@0.41x2");
    }

    #[test]
    fn initialize_replay_output_appends_bootstrap_rows_when_provided() {
        let _lock = ENV_LOCK.lock().unwrap_or_else(|poison| poison.into_inner());
        let _guard = EnvVarGuard::set(ENV_BOOTSTRAP_ROWS, "17|paper_fill|buy:market-1@0.62x5");
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lab-server-replay-bootstrap-{unique}"));
        let replay_path = root.join("nested").join("replay.csv");

        initialize_replay_output(replay_path.to_str().unwrap())
            .expect("startup should initialize replay output");

        let actual = fs::read_to_string(&replay_path).expect("replay output file should exist");
        assert_eq!(
            actual,
            format!("{REPLAY_CSV_HEADER}17,,,,paper_fill:buy:market-1@0.62x5,,,,\n")
        );

        fs::remove_dir_all(&root).expect("temp replay directory should be removable");
    }

    #[test]
    fn median_f64_returns_middle_value() {
        let values = vec![3.0, 5.0, 1.0, 7.0, 9.0];
        assert_eq!(median_f64(&values), Some(5.0));
    }

    #[test]
    fn parse_probability_str_rejects_out_of_range_values() {
        assert_eq!(parse_probability_str("1.1"), None);
        assert_eq!(parse_probability_str("-0.1"), None);
        assert_eq!(parse_probability_str("0.42"), Some(0.42));
    }

    #[test]
    fn btc_15m_market_filter_accepts_matching_market() {
        assert!(is_btc_15m_market(
            "bitcoin-15m-forecast",
            "Will BTC be above 66k in the next 15 minutes?"
        ));
    }

    #[test]
    fn btc_15m_market_filter_rejects_non_15m_or_non_btc_market() {
        assert!(!is_btc_15m_market(
            "bitcoin-daily-forecast",
            "Will BTC be above 70k tomorrow?"
        ));
        assert!(!is_btc_15m_market(
            "eth-15m-forecast",
            "Will ETH rise in 15 minutes?"
        ));
    }
}
