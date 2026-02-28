# Paper-Live Polymarket Adapter Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add default live-data paper-trading mode that consumes multi-venue BTC prices and auto-discovered Polymarket quotes, then generates and tracks paper orders/fills without routing real orders.

**Architecture:** Extend the current Rust monolith with a live ingest pipeline (`runtime::live`), a paper execution pipeline (`runtime::paper_exec`), and strategy/risk wiring that emits typed events consumed by API/UI. Keep external API integration isolated behind provider traits and normalize all feed data into internal event types before strategy logic. Preserve safety by allowing live data only and preventing authenticated/live execution paths.

**Tech Stack:** Rust, tokio, axum, serde, tokio-tungstenite, reqwest, tracing, existing runtime/strategy/api/ui crates.

---

### Task 1: Add paper-live mode configuration defaults

**Files:**
- Modify: `crates/lab-server/src/config.rs`
- Modify: `crates/lab-server/src/main.rs`
- Test: `crates/lab-server/src/config.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn defaults_to_paper_live_mode() {
    let cfg = Config::from_env().unwrap();
    assert_eq!(cfg.mode, RunMode::PaperLive);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server defaults_to_paper_live_mode -q`
Expected: FAIL with "no field `mode`" or missing `RunMode`

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum RunMode { PaperLive, Sim }

pub struct Config {
    pub mode: RunMode,
    // existing fields...
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server defaults_to_paper_live_mode -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/lab-server/src/config.rs crates/lab-server/src/main.rs
git commit -m "feat(server): add default paper-live runtime mode"
```

### Task 2: Define live ingest event models

**Files:**
- Create: `crates/runtime/src/live/mod.rs`
- Create: `crates/runtime/src/live/types.rs`
- Modify: `crates/runtime/src/lib.rs`
- Test: `crates/runtime/src/live/types.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn btc_median_tick_serializes_with_required_fields() {
    let tick = BtcMedianTick::new(64_000.0, 12.5, 3);
    let json = serde_json::to_value(tick).unwrap();
    assert!(json.get("px_median").is_some());
    assert!(json.get("venue_count").is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime btc_median_tick_serializes_with_required_fields -q`
Expected: FAIL with missing `BtcMedianTick`

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BtcMedianTick {
    pub px_median: f64,
    pub px_spread_bps: f64,
    pub venue_count: usize,
    pub ts_unix_ms: i64,
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime btc_median_tick_serializes_with_required_fields -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/live crates/runtime/src/lib.rs crates/runtime/Cargo.toml
git commit -m "feat(runtime): add live ingest event models"
```

### Task 3: Add BTC venue adapters and normalized trade parser

**Files:**
- Create: `crates/runtime/src/live/btc_feed.rs`
- Create: `crates/runtime/src/live/btc_parse.rs`
- Modify: `crates/runtime/src/live/mod.rs`
- Test: `crates/runtime/src/live/btc_parse.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn parses_coinbase_trade_into_normalized_tick() {
    let raw = r#"{"type":"match","price":"64001.2","size":"0.01","time":"2026-02-28T12:00:00Z"}"#;
    let tick = parse_coinbase_trade(raw).unwrap();
    assert_eq!(tick.venue, "coinbase");
    assert!(tick.px > 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime parses_coinbase_trade_into_normalized_tick -q`
Expected: FAIL with missing parser function

**Step 3: Write minimal implementation**

```rust
pub fn parse_coinbase_trade(raw: &str) -> anyhow::Result<BtcVenueTick> {
    // deserialize and map fields into normalized tick
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime parses_coinbase_trade_into_normalized_tick -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/live/btc_feed.rs crates/runtime/src/live/btc_parse.rs crates/runtime/src/live/mod.rs
git commit -m "feat(runtime): add normalized BTC venue adapters"
```

### Task 4: Implement multi-venue median aggregation

**Files:**
- Create: `crates/runtime/src/live/median.rs`
- Modify: `crates/runtime/src/live/mod.rs`
- Test: `crates/runtime/src/live/median.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn median_ignores_stale_and_outlier_ticks() {
    let mut agg = MedianAggregator::new(2_000, 200.0);
    // fresh ticks + one stale + one outlier
    let out = agg.compute().unwrap();
    assert_eq!(out.venue_count, 3);
    assert!(out.px_median > 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime median_ignores_stale_and_outlier_ticks -q`
Expected: FAIL with missing `MedianAggregator`

**Step 3: Write minimal implementation**

```rust
pub struct MedianAggregator { /* staleness/outlier config */ }
impl MedianAggregator {
    pub fn update(&mut self, tick: BtcVenueTick) { /* ... */ }
    pub fn compute(&self) -> Option<BtcMedianTick> { /* ... */ }
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime median_ignores_stale_and_outlier_ticks -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/live/median.rs crates/runtime/src/live/mod.rs
git commit -m "feat(runtime): add robust multi-venue BTC median aggregator"
```

### Task 5: Add Polymarket auto-discovery and quote normalization

**Files:**
- Create: `crates/runtime/src/live/polymarket_discovery.rs`
- Create: `crates/runtime/src/live/polymarket_quote.rs`
- Modify: `crates/runtime/src/live/mod.rs`
- Test: `crates/runtime/src/live/polymarket_discovery.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn discovery_filters_market_candidates() {
    let markets = vec![sample_market("btc-up-down"), sample_market("sports-final")];
    let out = filter_markets(markets, "btc");
    assert_eq!(out.len(), 1);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime discovery_filters_market_candidates -q`
Expected: FAIL with missing `filter_markets`

**Step 3: Write minimal implementation**

```rust
pub fn filter_markets(markets: Vec<DiscoveryMarket>, keyword: &str) -> Vec<DiscoveryMarket> {
    markets.into_iter().filter(|m| m.slug.contains(keyword)).collect()
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime discovery_filters_market_candidates -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/live/polymarket_discovery.rs crates/runtime/src/live/polymarket_quote.rs crates/runtime/src/live/mod.rs
git commit -m "feat(runtime): add polymarket discovery and quote normalization"
```

### Task 6: Wire divergence strategy for live ticks

**Files:**
- Modify: `crates/strategy/src/divergence.rs`
- Create: `crates/strategy/src/live_signal.rs`
- Modify: `crates/strategy/src/lib.rs`
- Test: `crates/strategy/src/live_signal.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn emits_buy_intent_when_btc_reference_exceeds_market_threshold() {
    let signal = live_signal(64_200.0, 63_800.0, 0.003).unwrap();
    assert_eq!(signal.action, Action::Buy);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p strategy emits_buy_intent_when_btc_reference_exceeds_market_threshold -q`
Expected: FAIL with missing `live_signal`

**Step 3: Write minimal implementation**

```rust
pub fn live_signal(reference_px: f64, market_mid: f64, threshold: f64) -> Result<Signal, StrategyError> {
    // compute normalized divergence and map to buy/sell/hold
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p strategy emits_buy_intent_when_btc_reference_exceeds_market_threshold -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/strategy/src/divergence.rs crates/strategy/src/live_signal.rs crates/strategy/src/lib.rs
git commit -m "feat(strategy): add live divergence signal generation"
```

### Task 7: Implement paper execution pricing (BBO + slippage)

**Files:**
- Create: `crates/runtime/src/paper_exec.rs`
- Modify: `crates/runtime/src/lib.rs`
- Test: `crates/runtime/src/paper_exec.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn buy_fill_uses_ask_plus_slippage_and_fee() {
    let fill = paper_fill_buy(0.62, 5.0, 10.0, 2.0).unwrap();
    assert!(fill.fill_px > 0.62);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime buy_fill_uses_ask_plus_slippage_and_fee -q`
Expected: FAIL with missing paper fill function

**Step 3: Write minimal implementation**

```rust
pub fn paper_fill_buy(ask: f64, qty: f64, slippage_bps: f64, fee_bps: f64) -> anyhow::Result<PaperFill> {
    // apply ask-side slippage + fee model
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime buy_fill_uses_ask_plus_slippage_and_fee -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/paper_exec.rs crates/runtime/src/lib.rs
git commit -m "feat(runtime): add paper execution bbo pricing"
```

### Task 8: Extend risk gates for paper-live flow

**Files:**
- Modify: `crates/strategy/src/risk.rs`
- Modify: `crates/strategy/src/lib.rs`
- Test: `crates/strategy/src/risk.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn rejects_intent_when_market_exposure_cap_exceeded() {
    let mut risk = RiskState::new(100_000.0, 0.02);
    let decision = risk.check_market_exposure("btc-up", 10_000.0, 2_000.0);
    assert!(decision.is_err());
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p strategy rejects_intent_when_market_exposure_cap_exceeded -q`
Expected: FAIL with missing exposure check

**Step 3: Write minimal implementation**

```rust
pub fn check_market_exposure(&self, market_id: &str, current: f64, incoming: f64) -> Result<(), RiskError> {
    // enforce per-market cap
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p strategy rejects_intent_when_market_exposure_cap_exceeded -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/strategy/src/risk.rs crates/strategy/src/lib.rs
git commit -m "feat(strategy): add paper-live exposure risk gates"
```

### Task 9: Orchestrate live ingest + strategy + paper execution runtime

**Files:**
- Modify: `crates/runtime/src/engine.rs`
- Modify: `crates/runtime/src/events.rs`
- Create: `crates/runtime/src/live_runner.rs`
- Test: `crates/runtime/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn live_runner_emits_intent_then_fill_events() {
    let mut engine = SimEngine::for_test_seed(7);
    let out = engine.step_live_once().await;
    assert!(out.iter().any(|e| e.stage == RuntimeStage::PaperIntentCreated));
    assert!(out.iter().any(|e| e.stage == RuntimeStage::PaperFillRecorded));
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime live_runner_emits_intent_then_fill_events -q`
Expected: FAIL with missing stage/method

**Step 3: Write minimal implementation**

```rust
pub async fn step_live_once(&mut self) -> Vec<RuntimeEvent> {
    // read normalized ticks, produce intent, run paper fill, emit events
}
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime live_runner_emits_intent_then_fill_events -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/engine.rs crates/runtime/src/events.rs crates/runtime/src/live_runner.rs
git commit -m "feat(runtime): orchestrate paper-live execution flow"
```

### Task 10: Add API endpoints for feed health and discovered markets

**Files:**
- Modify: `crates/api/src/routes.rs`
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn get_feed_health_returns_mode_and_source_counts() {
    let app = app();
    let res = send_get(&app, "/feed/health").await;
    assert_eq!(res.status(), 200);
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api get_feed_health_returns_mode_and_source_counts -q`
Expected: FAIL with 404 or missing route

**Step 3: Write minimal implementation**

```rust
Router::new()
  .route("/feed/health", get(feed_health))
  .route("/markets/discovered", get(discovered_markets))
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api get_feed_health_returns_mode_and_source_counts -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/routes.rs crates/api/src/state.rs crates/api/src/lib.rs
git commit -m "feat(api): expose feed health and discovered markets"
```

### Task 11: Expand WebSocket payloads for paper-live telemetry

**Files:**
- Modify: `crates/api/src/ws.rs`
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn websocket_emits_paper_fill_event_payload() {
    let msg = next_ws_json().await;
    assert_eq!(msg["event_type"], "paper_fill");
    assert!(msg["fill_px"].as_f64().is_some());
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api websocket_emits_paper_fill_event_payload -q`
Expected: FAIL due to missing event schema

**Step 3: Write minimal implementation**

```rust
#[serde(rename_all = "snake_case")]
enum RuntimeEventType { Connected, RunStarted, PaperIntent, PaperFill, RiskReject }
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api websocket_emits_paper_fill_event_payload -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/ws.rs crates/api/src/state.rs crates/api/src/lib.rs
git commit -m "feat(api): stream paper-live telemetry over websocket"
```

### Task 12: Update dashboard for live paper-trading panels

**Files:**
- Modify: `crates/ui/static/index.html`
- Modify: `crates/ui/static/styles.css`
- Modify: `crates/ui/static/app.js`
- Test: `crates/ui/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn ui_shell_contains_paper_live_panels() {
    let html = ui::index_html();
    assert!(html.contains("Feed Health"));
    assert!(html.contains("Paper Fills"));
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui ui_shell_contains_paper_live_panels -q`
Expected: FAIL with missing panel labels

**Step 3: Write minimal implementation**

```html
<section id="feed-health">...</section>
<section id="paper-fills">...</section>
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui ui_shell_contains_paper_live_panels -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/static/index.html crates/ui/static/styles.css crates/ui/static/app.js crates/ui/src/lib.rs
git commit -m "feat(ui): add paper-live monitoring panels"
```

### Task 13: Persist paper journal and replay artifacts

**Files:**
- Modify: `crates/runtime/src/replay.rs`
- Modify: `crates/runtime/src/logging.rs`
- Modify: `crates/lab-server/src/main.rs`
- Test: `crates/runtime/src/replay.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn replay_writer_appends_paper_fill_rows() {
    let csv = write_csv_for_test(vec![sample_paper_fill_row()]).unwrap();
    assert!(csv.contains("paper_fill"));
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime replay_writer_appends_paper_fill_rows -q`
Expected: FAIL with missing row type/serialization

**Step 3: Write minimal implementation**

```rust
pub struct PaperJournalRow { /* event_type, market_id, qty, fill_px, pnl */ }
```

**Step 4: Run test to verify it passes**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime replay_writer_appends_paper_fill_rows -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/replay.rs crates/runtime/src/logging.rs crates/lab-server/src/main.rs
git commit -m "feat(runtime): persist paper journal artifacts"
```

### Task 14: Update docs and safety language for live-data paper mode

**Files:**
- Modify: `README.md`
- Modify: `docs/methodology.md`
- Modify: `docs/migration/python-to-rust.md`
- Create: `docs/operations/paper-live-checklist.md`

**Step 1: Write the failing doc check**

```bash
rg -n "simulation-only" README.md docs/methodology.md docs/migration/python-to-rust.md
```

**Step 2: Run check to verify it fails current expectation**

Run: `rg -n "simulation-only" README.md docs/methodology.md docs/migration/python-to-rust.md`
Expected: matches exist that need updating

**Step 3: Write minimal documentation updates**

```text
State: live-data paper trading is supported by default.
State: no real-money execution and no live order routing.
```

**Step 4: Run doc checks**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q`
Expected: PASS with doc updates committed

**Step 5: Commit**

```bash
git add README.md docs/methodology.md docs/migration/python-to-rust.md docs/operations/paper-live-checklist.md
git commit -m "docs: document live-data paper trading mode"
```

### Task 15: Final verification gate for paper-live release

**Files:**
- Modify: `.github/workflows/sim-ci.yml`
- Modify: `docs/plans/verification-checklist.md`

**Step 1: Write failing verification expectation**

```text
CI must include a paper-live integration test step.
```

**Step 2: Run current checks to verify missing coverage**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q`
Expected: PASS, but CI still lacks explicit paper-live integration test step

**Step 3: Write minimal implementation**

```yaml
- name: Paper Live Integration Verify
  run: cargo test -p runtime live_ -- --nocapture
```

**Step 4: Run full verification locally**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --all -- --check && PATH="$HOME/.cargo/bin:$PATH" cargo clippy --workspace -- -D warnings && PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/sim-ci.yml docs/plans/verification-checklist.md
git commit -m "ci: add paper-live verification coverage"
```

## Notes for Execution

- Apply `@superpowers/test-driven-development` for every task (red -> green -> refactor).
- Apply `@superpowers/systematic-debugging` before changing code if unexpected failures occur.
- Apply `@superpowers/verification-before-completion` before each completion claim.
- Keep live-data adapters read-only to external venues; do not add authenticated order submission.
- Keep commits scoped one task at a time.
