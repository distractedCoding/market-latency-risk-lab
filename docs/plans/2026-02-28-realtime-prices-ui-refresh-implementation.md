# Realtime Prices + UI Refresh Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Show Coinbase/Binance/Kraken BTC prices and Polymarket YES bid/ask/mid in realtime on the dashboard while improving layout and readability.

**Architecture:** Extend `api::state` with a typed `PriceSnapshot`, publish it from the existing `paper-live` loop in `lab-server`, and expose a REST fallback endpoint. Update the UI shell/styles/scripts to render a dedicated Live Prices panel driven primarily by websocket events and secondarily by periodic REST polling.

**Tech Stack:** Rust, axum, tokio, serde, reqwest, WebSocket (`/ws/events`), static HTML/CSS/JS UI bundle.

---

### Task 1: Add API tests for price snapshot endpoint and websocket payload

**Files:**
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/lib.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn get_prices_snapshot_returns_typed_payload() {
    // GET /prices/snapshot should return concrete values
}

#[tokio::test]
async fn websocket_emits_price_snapshot_event_payload() {
    // RuntimeEvent::price_snapshot should be forwarded on /ws/events
}
```

**Step 2: Run tests to verify they fail**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api get_prices_snapshot_returns_typed_payload websocket_emits_price_snapshot_event_payload -q`
Expected: FAIL due to missing route/event/model.

**Step 3: Commit (after implementation in later tasks)**

```bash
git add crates/api/src/lib.rs
git commit -m "test(api): cover realtime price snapshot route and ws payload"
```

### Task 2: Implement server state and route for price snapshots

**Files:**
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/routes.rs`

**Step 1: Write minimal implementation in state**

```rust
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
```

Add storage and accessors on `AppState`, and add `RuntimeEvent::PriceSnapshot` + constructor.

**Step 2: Wire route**

Add `GET /prices/snapshot` in `crates/api/src/routes.rs` returning `Json<PriceSnapshot>`.

**Step 3: Run API tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api -q`
Expected: PASS including the new snapshot tests.

**Step 4: Commit**

```bash
git add crates/api/src/state.rs crates/api/src/routes.rs crates/api/src/lib.rs
git commit -m "feat(api): add realtime price snapshot state, route, and ws event"
```

### Task 3: Publish price snapshots from paper-live runtime loop

**Files:**
- Modify: `crates/lab-server/src/main.rs`

**Step 1: Add failing runtime unit test**

```rust
#[test]
fn price_snapshot_event_carries_exchange_and_polymarket_fields() {
    // construct snapshot, convert to RuntimeEvent, assert serialized keys
}
```

**Step 2: Run test to verify it fails**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server price_snapshot_event_carries_exchange_and_polymarket_fields -q`
Expected: FAIL if constructor/event variant is missing.

**Step 3: Implement loop publishing**

Build a `PriceSnapshot` every tick from:
- `coinbase_px`, `binance_px`, `kraken_px`
- first tracked `PolymarketQuoteTick` (`market_slug`, `best_yes_bid`, `best_yes_ask`, `mid_yes`)
- current `tick`

Then:
- `state.set_price_snapshot(snapshot.clone())`
- `state.publish_event(RuntimeEvent::price_snapshot(snapshot))`

**Step 4: Run lab-server tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/lab-server/src/main.rs
git commit -m "feat(server): publish realtime exchange and polymarket price snapshots"
```

### Task 4: Update dashboard markup and styles for Live Prices panel

**Files:**
- Modify: `crates/ui/static/index.html`
- Modify: `crates/ui/static/styles.css`
- Modify: `crates/ui/src/lib.rs`

**Step 1: Write failing UI test**

```rust
#[test]
fn ui_shell_contains_live_prices_panel() {
    let html = ui::index_html();
    assert!(html.contains("Live Prices"));
    assert!(html.contains("Coinbase BTC/USD"));
    assert!(html.contains("Polymarket YES"));
}
```

**Step 2: Run test to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui ui_shell_contains_live_prices_panel -q`
Expected: FAIL before markup change.

**Step 3: Implement markup and styling**

Add price rows, trend badges, and updated visual hierarchy that remains mobile-safe.

**Step 4: Run UI tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui/static/index.html crates/ui/static/styles.css crates/ui/src/lib.rs
git commit -m "feat(ui): add dedicated live prices panel and refined dashboard styling"
```

### Task 5: Add JS routing for price snapshot realtime updates + fallback polling

**Files:**
- Modify: `crates/ui/static/app.js`
- Modify: `crates/ui/src/lib.rs`

**Step 1: Write failing JS bundle test**

```rust
#[test]
fn app_js_routes_price_snapshot_and_polls_snapshot_endpoint() {
    let js = app_js();
    assert!(js.contains("price_snapshot"));
    assert!(js.contains("/prices/snapshot"));
}
```

**Step 2: Run test to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui app_js_routes_price_snapshot_and_polls_snapshot_endpoint -q`
Expected: FAIL before JS changes.

**Step 3: Implement JS behavior**

Add:
- `updatePriceSnapshot` rendering for venue and Polymarket values
- trend computation per venue (`up/down/flat`)
- stale marker after inactivity
- `/prices/snapshot` polling fallback

**Step 4: Run UI + workspace tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui -q && PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q`
Expected: PASS.

**Step 5: Commit**

```bash
git add crates/ui/static/app.js crates/ui/src/lib.rs
git commit -m "feat(ui): render realtime exchange and polymarket prices with trends"
```

### Task 6: Runtime verification in browser

**Files:**
- No code changes expected

**Step 1: Run server**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server`

**Step 2: Verify endpoints**

Run: `curl -sS http://127.0.0.1:8080/prices/snapshot`
Expected: JSON with exchange and Polymarket fields.

**Step 3: Verify websocket payload**

Run a websocket client and confirm `event_type=price_snapshot` events are flowing.

**Step 4: Manual UI verification**

Open `http://127.0.0.1:8080/` and confirm:
- Live Prices panel updates continuously
- trend badges move with price changes
- Polymarket YES bid/ask/mid shown
- existing chart/PnL/feeds still update
