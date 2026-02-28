# Rust Latency Risk Lab Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Replace the Python CLI simulator with a Rust monolith that runs a high-fidelity, simulation-only latency-risk engine and serves a real-time web dashboard.

**Architecture:** Build a Rust workspace with focused crates (`core-sim`, `strategy`, `runtime`, `api`, `ui`, `lab-server`) connected by typed events over bounded async channels. Keep hot-path simulation deterministic and lock-light, with risk/accounting channels lossless and UI channels lossy. Serve static dashboard assets from the same `axum` process and stream telemetry over WebSocket.

**Tech Stack:** Rust 1.88+, `tokio`, `axum`, `serde`, `tracing`, `tower-http`, `criterion`, `tokio-tungstenite`, vanilla HTML/JS (Chart.js), CSV/JSONL output.

---

### Task 1: Bootstrap Rust workspace and CI smoke checks

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `crates/core-sim/Cargo.toml`
- Create: `crates/core-sim/src/lib.rs`
- Create: `crates/strategy/Cargo.toml`
- Create: `crates/strategy/src/lib.rs`
- Create: `crates/runtime/Cargo.toml`
- Create: `crates/runtime/src/lib.rs`
- Create: `crates/api/Cargo.toml`
- Create: `crates/api/src/lib.rs`
- Create: `crates/ui/Cargo.toml`
- Create: `crates/ui/src/lib.rs`
- Create: `crates/lab-server/Cargo.toml`
- Create: `crates/lab-server/src/main.rs`
- Modify: `.github/workflows/ci.yml`

**Step 1: Write the failing test**

```rust
// crates/core-sim/src/lib.rs
#[cfg(test)]
mod tests {
    #[test]
    fn workspace_builds() {
        let sum = 2 + 2;
        assert_eq!(sum, 4);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p core-sim -q`
Expected: FAIL with "package ID specification `core-sim` did not match any packages"

**Step 3: Write minimal implementation**

```toml
# Cargo.toml
[workspace]
members = [
  "crates/core-sim",
  "crates/strategy",
  "crates/runtime",
  "crates/api",
  "crates/ui",
  "crates/lab-server"
]
resolver = "2"
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p core-sim -q`
Expected: PASS

**Step 5: Commit**

```bash
git add Cargo.toml rust-toolchain.toml crates .github/workflows/ci.yml
git commit -m "chore: bootstrap rust workspace for simulator rewrite"
```

### Task 2: Add typed config and simulation state models

**Files:**
- Create: `crates/core-sim/src/config.rs`
- Create: `crates/core-sim/src/state.rs`
- Modify: `crates/core-sim/src/lib.rs`
- Test: `crates/core-sim/src/config.rs`

**Step 1: Write the failing test**

```rust
#[cfg(test)]
mod tests {
    use super::SimConfig;

    #[test]
    fn default_risk_limits_match_spec() {
        let cfg = SimConfig::default();
        assert!((cfg.max_position_pct - 0.005).abs() < f64::EPSILON);
        assert!((cfg.daily_loss_cap_pct - 0.02).abs() < f64::EPSILON);
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p core-sim default_risk_limits_match_spec -q`
Expected: FAIL with "cannot find type `SimConfig`"

**Step 3: Write minimal implementation**

```rust
#[derive(Debug, Clone)]
pub struct SimConfig {
    pub threshold: f64,
    pub max_position_pct: f64,
    pub daily_loss_cap_pct: f64,
    pub market_lag_ms: u64,
    pub decision_interval_ms: u64,
    pub fee_bps: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            threshold: 0.003,
            max_position_pct: 0.005,
            daily_loss_cap_pct: 0.02,
            market_lag_ms: 120,
            decision_interval_ms: 50,
            fee_bps: 2.0,
        }
    }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p core-sim default_risk_limits_match_spec -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/core-sim/src/config.rs crates/core-sim/src/state.rs crates/core-sim/src/lib.rs
git commit -m "feat(core-sim): add typed config and state models"
```

### Task 3: Implement deterministic prediction and market lag generators

**Files:**
- Create: `crates/core-sim/src/generators.rs`
- Modify: `crates/core-sim/src/lib.rs`
- Test: `crates/core-sim/src/generators.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn seeded_generators_are_deterministic() {
    let mut a = PriceGenerator::new(42, 100.0, 0.001);
    let mut b = PriceGenerator::new(42, 100.0, 0.001);
    for _ in 0..100 {
        assert_eq!(a.next_tick(), b.next_tick());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p core-sim seeded_generators_are_deterministic -q`
Expected: FAIL with "cannot find type `PriceGenerator`"

**Step 3: Write minimal implementation**

```rust
pub struct PriceGenerator {
    rng: rand_chacha::ChaCha8Rng,
    last: f64,
    sigma: f64,
}

impl PriceGenerator {
    pub fn new(seed: u64, start: f64, sigma: f64) -> Self { /* ... */ }
    pub fn next_tick(&mut self) -> f64 { /* ... */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p core-sim seeded_generators_are_deterministic -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/core-sim/src/generators.rs crates/core-sim/src/lib.rs crates/core-sim/Cargo.toml
git commit -m "feat(core-sim): add deterministic price and lag generators"
```

### Task 4: Build simulated order book and fill engine

**Files:**
- Create: `crates/core-sim/src/orderbook.rs`
- Create: `crates/core-sim/src/fills.rs`
- Modify: `crates/core-sim/src/lib.rs`
- Test: `crates/core-sim/src/orderbook.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn crossing_order_fills_at_best_level() {
    let mut book = OrderBook::new(100.0, 0.01, 20);
    let fill = book.execute_market(Side::Buy, 2.0).unwrap();
    assert!(fill.avg_price >= 100.0);
    assert!(fill.filled_qty > 0.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p core-sim crossing_order_fills_at_best_level -q`
Expected: FAIL with "cannot find type `OrderBook`"

**Step 3: Write minimal implementation**

```rust
pub struct OrderBook { /* best bid/ask ladders */ }

impl OrderBook {
    pub fn new(mid: f64, tick: f64, levels: usize) -> Self { /* ... */ }
    pub fn execute_market(&mut self, side: Side, qty: f64) -> Option<Fill> { /* ... */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p core-sim crossing_order_fills_at_best_level -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/core-sim/src/orderbook.rs crates/core-sim/src/fills.rs crates/core-sim/src/lib.rs
git commit -m "feat(core-sim): add order book and fill simulation"
```

### Task 5: Implement divergence strategy and regime-aware sizing

**Files:**
- Create: `crates/strategy/src/divergence.rs`
- Create: `crates/strategy/src/sizing.rs`
- Modify: `crates/strategy/src/lib.rs`
- Test: `crates/strategy/src/divergence.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn emits_buy_signal_when_prediction_leads_market_above_threshold() {
    let cfg = StrategyConfig::default();
    let signal = compute_signal(101.0, 100.0, &cfg);
    assert_eq!(signal.action, Action::Buy);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p strategy emits_buy_signal_when_prediction_leads_market_above_threshold -q`
Expected: FAIL with "cannot find function `compute_signal`"

**Step 3: Write minimal implementation**

```rust
pub fn compute_signal(prediction_px: f64, market_px: f64, cfg: &StrategyConfig) -> Signal {
    let div = (prediction_px - market_px) / market_px;
    if div > cfg.threshold { Signal::buy(div) }
    else if div < -cfg.threshold { Signal::sell(div) }
    else { Signal::hold(div) }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p strategy emits_buy_signal_when_prediction_leads_market_above_threshold -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/strategy/src/divergence.rs crates/strategy/src/sizing.rs crates/strategy/src/lib.rs
git commit -m "feat(strategy): add divergence signal and sizing"
```

### Task 6: Add risk engine and kill-switch invariants

**Files:**
- Create: `crates/strategy/src/risk.rs`
- Modify: `crates/strategy/src/lib.rs`
- Test: `crates/strategy/src/risk.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn halts_when_daily_loss_cap_is_breached() {
    let mut risk = RiskState::new(100_000.0, 0.02);
    risk.apply_realized_pnl(-2_001.0);
    assert!(risk.halted);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p strategy halts_when_daily_loss_cap_is_breached -q`
Expected: FAIL with "cannot find type `RiskState`"

**Step 3: Write minimal implementation**

```rust
pub struct RiskState {
    starting_equity: f64,
    realized_pnl: f64,
    daily_loss_cap_pct: f64,
    pub halted: bool,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p strategy halts_when_daily_loss_cap_is_breached -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/strategy/src/risk.rs crates/strategy/src/lib.rs
git commit -m "feat(strategy): enforce daily loss cap and kill switch"
```

### Task 7: Build runtime event bus and supervised task orchestration

**Files:**
- Create: `crates/runtime/src/events.rs`
- Create: `crates/runtime/src/engine.rs`
- Create: `crates/runtime/src/supervisor.rs`
- Modify: `crates/runtime/src/lib.rs`
- Test: `crates/runtime/src/engine.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn engine_emits_events_in_expected_order() {
    let mut engine = SimEngine::for_test_seed(7);
    let out = engine.step_once().await;
    assert_eq!(out.stage, Stage::PortfolioUpdated);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime engine_emits_events_in_expected_order -q`
Expected: FAIL with "cannot find type `SimEngine`"

**Step 3: Write minimal implementation**

```rust
pub struct SimEngine {
    // channels and module handles
}

impl SimEngine {
    pub fn for_test_seed(seed: u64) -> Self { /* ... */ }
    pub async fn step_once(&mut self) -> StepOutput { /* ... */ }
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime engine_emits_events_in_expected_order -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/events.rs crates/runtime/src/engine.rs crates/runtime/src/supervisor.rs crates/runtime/src/lib.rs
git commit -m "feat(runtime): add event bus and supervised orchestration"
```

### Task 8: Implement metrics aggregation and structured run logging

**Files:**
- Create: `crates/runtime/src/metrics.rs`
- Create: `crates/runtime/src/logging.rs`
- Modify: `crates/runtime/src/lib.rs`
- Test: `crates/runtime/src/metrics.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn latency_percentiles_are_reported() {
    let mut m = Metrics::default();
    for n in 1..=100 {
        m.record_decision_latency_ms(n as f64);
    }
    assert!(m.p99_decision_latency_ms() >= 99.0);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime latency_percentiles_are_reported -q`
Expected: FAIL with "cannot find type `Metrics`"

**Step 3: Write minimal implementation**

```rust
#[derive(Default)]
pub struct Metrics {
    decision_latencies_ms: Vec<f64>,
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime latency_percentiles_are_reported -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/metrics.rs crates/runtime/src/logging.rs crates/runtime/src/lib.rs
git commit -m "feat(runtime): add metrics and structured run logs"
```

### Task 9: Add HTTP control API for run lifecycle and config

**Files:**
- Create: `crates/api/src/routes.rs`
- Create: `crates/api/src/state.rs`
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/routes.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn post_runs_starts_new_run() {
    let app = app_router_for_test();
    let res = send_post(&app, "/runs", "{}").await;
    assert_eq!(res.status(), 201);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p api post_runs_starts_new_run -q`
Expected: FAIL with "cannot find function `app_router_for_test`"

**Step 3: Write minimal implementation**

```rust
pub fn router(state: ApiState) -> Router {
    Router::new().route("/runs", post(start_run)).with_state(state)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p api post_runs_starts_new_run -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/routes.rs crates/api/src/state.rs crates/api/src/lib.rs
git commit -m "feat(api): add run lifecycle HTTP endpoints"
```

### Task 10: Add WebSocket broadcast channels for live telemetry

**Files:**
- Create: `crates/api/src/ws.rs`
- Modify: `crates/api/src/routes.rs`
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/ws.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn websocket_streams_events_channel() {
    let mut client = connect_ws_for_test("/ws/events").await;
    let msg = client.read_text().await;
    assert!(msg.contains("event_type"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p api websocket_streams_events_channel -q`
Expected: FAIL with "cannot find function `connect_ws_for_test`"

**Step 3: Write minimal implementation**

```rust
pub async fn ws_events_handler(
    ws: WebSocketUpgrade,
    State(state): State<ApiState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| stream_events(socket, state))
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p api websocket_streams_events_channel -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/api/src/ws.rs crates/api/src/routes.rs crates/api/src/lib.rs
git commit -m "feat(api): stream runtime telemetry over websocket"
```

### Task 11: Build and serve dashboard UI assets

**Files:**
- Create: `crates/ui/static/index.html`
- Create: `crates/ui/static/styles.css`
- Create: `crates/ui/static/app.js`
- Create: `crates/ui/src/lib.rs`
- Modify: `crates/api/src/routes.rs`
- Test: `crates/ui/src/lib.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn ui_bundle_contains_index_html() {
    let html = ui::index_html();
    assert!(html.contains("Latency Risk Lab"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p ui ui_bundle_contains_index_html -q`
Expected: FAIL with "cannot find function `index_html`"

**Step 3: Write minimal implementation**

```rust
pub fn index_html() -> &'static str {
    include_str!("../static/index.html")
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p ui ui_bundle_contains_index_html -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/ui/static/index.html crates/ui/static/styles.css crates/ui/static/app.js crates/ui/src/lib.rs crates/api/src/routes.rs
git commit -m "feat(ui): add real-time dashboard served by backend"
```

### Task 12: Wire `lab-server` binary and end-to-end run flow

**Files:**
- Modify: `crates/lab-server/src/main.rs`
- Create: `crates/lab-server/src/config.rs`
- Create: `crates/lab-server/src/wiring.rs`
- Test: `crates/lab-server/src/main.rs`

**Step 1: Write the failing test**

```rust
#[tokio::test]
async fn server_healthcheck_responds_ok() {
    let app = build_app_for_test();
    let res = send_get(&app, "/health").await;
    assert_eq!(res.status(), 200);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p lab-server server_healthcheck_responds_ok -q`
Expected: FAIL with "cannot find function `build_app_for_test`"

**Step 3: Write minimal implementation**

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = wiring::build_app()?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p lab-server server_healthcheck_responds_ok -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/lab-server/src/main.rs crates/lab-server/src/config.rs crates/lab-server/src/wiring.rs
git commit -m "feat(server): wire runtime, api, and ui into executable"
```

### Task 13: Add replay export and compatibility artifact format

**Files:**
- Modify: `crates/runtime/src/logging.rs`
- Create: `crates/runtime/src/replay.rs`
- Modify: `crates/lab-server/src/config.rs`
- Test: `crates/runtime/src/replay.rs`

**Step 1: Write the failing test**

```rust
#[test]
fn replay_writer_outputs_csv_headers() {
    let csv = replay::write_csv_for_test(vec![]).unwrap();
    assert!(csv.starts_with("t,external_px,market_px,divergence,action"));
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime replay_writer_outputs_csv_headers -q`
Expected: FAIL with "cannot find module `replay`"

**Step 3: Write minimal implementation**

```rust
pub fn write_csv_for_test(_events: Vec<ReplayRow>) -> anyhow::Result<String> {
    Ok("t,external_px,market_px,divergence,action,equity,realized_pnl,position,halted\n".to_string())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p runtime replay_writer_outputs_csv_headers -q`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/runtime/src/logging.rs crates/runtime/src/replay.rs crates/lab-server/src/config.rs
git commit -m "feat(runtime): export replay artifacts for analysis"
```

### Task 14: Benchmark throughput and latency budgets

**Files:**
- Create: `crates/runtime/benches/throughput.rs`
- Create: `crates/runtime/benches/latency.rs`
- Modify: `crates/runtime/Cargo.toml`
- Modify: `README.md`

**Step 1: Write the failing benchmark assertion helper test**

```rust
#[test]
fn benchmark_target_is_defined() {
    assert_eq!(TARGET_ORDERS_PER_SEC, 1000);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p runtime benchmark_target_is_defined -q`
Expected: FAIL with "cannot find value `TARGET_ORDERS_PER_SEC`"

**Step 3: Write minimal implementation**

```rust
pub const TARGET_ORDERS_PER_SEC: usize = 1000;
```

**Step 4: Run tests and benches**

Run: `cargo test -p runtime -q && cargo bench -p runtime`
Expected: tests PASS; benchmarks print throughput and p99 latency stats

**Step 5: Commit**

```bash
git add crates/runtime/benches crates/runtime/Cargo.toml README.md
git commit -m "perf(runtime): add benchmark suite and performance targets"
```

### Task 15: Migrate docs and retire Python entrypoint

**Files:**
- Modify: `README.md`
- Modify: `docs/methodology.md`
- Modify: `requirements.txt`
- Delete: `sim/main.py`
- Delete: `sim/models.py`
- Delete: `sim/engine.py`
- Delete: `tests/test_logic.py`
- Create: `docs/migration/python-to-rust.md`

**Step 1: Write the failing test for old path removal**

```bash
test ! -f sim/main.py
```

**Step 2: Run check to verify it fails**

Run: `test ! -f sim/main.py`
Expected: FAIL (file exists)

**Step 3: Write minimal implementation**

```text
Update docs to Rust commands and remove Python simulator files after parity is confirmed.
```

**Step 4: Run verification**

Run: `cargo test --workspace -q && cargo run -p lab-server -- --help`
Expected: PASS; server usage output shown

**Step 5: Commit**

```bash
git add README.md docs/methodology.md docs/migration/python-to-rust.md requirements.txt sim tests
git commit -m "chore: finalize rust migration and retire python simulator"
```

### Task 16: Final verification gate before PR

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `docs/plans/verification-checklist.md`

**Step 1: Write failing CI matrix check**

```yaml
# Add a required step name in CI expected by local script:
# "Rust Workspace Verify"
```

**Step 2: Run local verification script to confirm missing step fails**

Run: `bash scripts/verify-ci.sh`
Expected: FAIL with "missing Rust Workspace Verify step"

**Step 3: Write minimal implementation**

```yaml
- name: Rust Workspace Verify
  run: cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace
```

**Step 4: Run full verification**

Run: `cargo fmt --check && cargo clippy --workspace -- -D warnings && cargo test --workspace && cargo bench -p runtime`
Expected: PASS

**Step 5: Commit**

```bash
git add .github/workflows/ci.yml docs/plans/verification-checklist.md
git commit -m "ci: enforce formatting, lint, tests, and runtime benchmarks"
```

## Notes for Execution

- Apply `@superpowers/test-driven-development` inside every task (fail, implement minimal pass, refactor).
- Apply `@superpowers/verification-before-completion` before claiming task complete.
- If tests fail unexpectedly, apply `@superpowers/systematic-debugging` before changing implementation.
- Keep each commit scoped to one task only.
- Do not add live exchange connectivity; preserve simulation-only boundaries.
