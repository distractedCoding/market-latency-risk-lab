mod config;
mod wiring;

use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::path::Path;

use api::state::{
    AppState, DiscoveredMarket, FeedMode, PaperOrderSide, PortfolioSummary, RuntimeEvent,
    SourceCount,
};
use reqwest::Client;
use runtime::events::RuntimeStage;
use runtime::live::{
    filter_markets, BtcMedianTick, PolymarketMarket, PolymarketQuoteTick, RawPolymarketQuote,
};
use runtime::live_runner::{run_paper_live_once, JoinedLiveInputs};
use runtime::logging::{PaperJournalRow, PaperJournalRowKind};
use runtime::replay::ReplayCsvWriter;
use serde::Deserialize;
use tokio::net::TcpListener;
use tokio::time::{self, Duration, MissedTickBehavior};

const BOOTSTRAP_ROWS_ENV: &str = "LAB_SERVER_INITIAL_PAPER_JOURNAL_ROWS";
const PAPER_MARKET_ID: &str = "btc-up-down";
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
    } = config::Config::from_env()?;

    println!("{}", startup_mode_banner(mode));
    initialize_replay_output(&replay_output_path)?;
    let app_state = AppState::new();

    if mode == config::RunMode::PaperLive {
        let client = Client::builder()
            .user_agent("market-latency-risk-lab/paper-live")
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()?;
        tokio::spawn(run_paper_live_loop(app_state.clone(), client));
    }

    let listener = TcpListener::bind(listen_addr).await?;
    axum::serve(listener, wiring::build_app_with_state(app_state)).await?;
    Ok(())
}

async fn run_paper_live_loop(state: AppState, client: Client) {
    let mut interval = time::interval(Duration::from_millis(LIVE_LOOP_INTERVAL_MS));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let mut tick = 0_u64;
    let mut counters = SourceCounters::default();
    let mut last_btc_median: Option<f64> = None;
    let mut tracked_quotes: Vec<PolymarketQuoteTick> = Vec::new();

    let mut cash = 0.0_f64;
    let mut position_qty = 0.0_f64;
    let mut fills = 0_u64;

    state.set_discovered_markets(vec![DiscoveredMarket {
        source: "polymarket".to_string(),
        market_id: PAPER_MARKET_ID.to_string(),
    }]);

    loop {
        interval.tick().await;
        tick = tick.saturating_add(1);

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

        let source_counts = counters.as_source_counts();
        state.set_feed_source_counts(source_counts.clone());
        let _ = state.publish_event(RuntimeEvent::feed_health(
            FeedMode::PaperLive,
            source_counts,
        ));

        for quote in tracked_quotes.iter().take(MAX_TRACKED_POLY_MARKETS) {
            let joined = JoinedLiveInputs {
                btc_tick: BtcMedianTick::new(
                    btc_median,
                    spread_signal,
                    btc_samples.len() as u32,
                    tick,
                ),
                quote_tick: quote.clone(),
            };

            let runtime_events = run_paper_live_once(tick, &joined);
            let has_intent = runtime_events
                .iter()
                .any(|event| event.stage == RuntimeStage::PaperIntentCreated);
            if !has_intent {
                continue;
            }

            let side = if spread_signal >= 0.0 {
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

            let has_fill = runtime_events
                .iter()
                .any(|event| event.stage == RuntimeStage::PaperFillRecorded);
            if has_fill {
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

                let _ = state.publish_event(RuntimeEvent::paper_fill(
                    &quote.market_slug,
                    side,
                    PAPER_ORDER_QTY,
                    fill_px,
                ));
            } else {
                let _ = state.publish_event(RuntimeEvent::risk_reject(
                    &quote.market_slug,
                    "risk gate rejected",
                    PAPER_ORDER_QTY,
                ));
            }
        }

        let mark_price = tracked_quotes
            .first()
            .map(|quote| quote.mid_yes)
            .unwrap_or(0.5);
        let equity = cash + (position_qty * mark_price);
        let summary = PortfolioSummary {
            equity,
            pnl: equity,
            position_qty,
            fills,
        };
        state.set_portfolio_summary(summary);
        let _ = state.publish_event(RuntimeEvent::portfolio_snapshot(summary));
    }
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

    let candidates = markets
        .iter()
        .map(|market| PolymarketMarket {
            slug: format!(
                "{} {}",
                market.slug.to_ascii_lowercase(),
                market.question.to_ascii_lowercase()
            ),
        })
        .collect::<Vec<_>>();
    let mut bitcoin_candidates = filter_markets(candidates.clone(), "btc");
    if bitcoin_candidates.is_empty() {
        bitcoin_candidates = filter_markets(candidates, "bitcoin");
    }

    let mut discovered = Vec::new();
    let mut quotes = Vec::new();

    for market in markets.iter() {
        let haystack = format!(
            "{} {}",
            market.slug.to_ascii_lowercase(),
            market.question.to_ascii_lowercase()
        );
        if !bitcoin_candidates
            .iter()
            .any(|candidate| candidate.slug == haystack)
        {
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
        initial_paper_journal_rows, initialize_replay_output, median_f64, parse_probability_str,
        startup_mode_banner,
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
}
