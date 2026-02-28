# Dashboard Settings + BTC 15m Forecast Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a settings-driven dashboard with top KPI strip, center analytics, right-side chat logs, and BTC-only 15-minute forecast telemetry.

**Architecture:** Extend API state with runtime settings, strategy stats, BTC forecast, and execution log snapshots. Update lab-server loop to apply settings each tick and publish new telemetry events. Replace the UI shell with a three-region layout and wire realtime/REST fallback updates for settings, KPIs, forecast, and chat logs.

**Tech Stack:** Rust (axum, tokio, serde), static HTML/CSS/JS dashboard, workspace cargo tests.

---

### Task 1: Add failing API tests for settings and BTC stats/forecast routes

**Files:**
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/lib.rs`

**Step 1: Write the failing tests**

```rust
#[tokio::test]
async fn get_settings_returns_runtime_controls() {}

#[tokio::test]
async fn patch_settings_updates_runtime_controls() {}

#[tokio::test]
async fn get_strategy_stats_returns_top_kpis() {}

#[tokio::test]
async fn get_btc_15m_forecast_returns_fixed_horizon_payload() {}
```

**Step 2: Run tests to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api get_settings_returns_runtime_controls patch_settings_updates_runtime_controls get_strategy_stats_returns_top_kpis get_btc_15m_forecast_returns_fixed_horizon_payload -q`
Expected: FAIL due to missing state/route handlers.

### Task 2: Implement API state models and routes for settings, stats, forecast

**Files:**
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/routes.rs`
- Modify: `crates/api/src/lib.rs`

**Step 1: Add settings and telemetry models**

Add serializable structs with defaults:
- `RuntimeSettings`
- `StrategyStatsSummary`
- `BtcForecastSummary`
- `ExecutionLogEntry`

Add `AppState` storage + getters/setters and websocket event variants:
- `settings_updated`
- `strategy_stats`
- `btc_forecast`
- `execution_log`

**Step 2: Add routes**

Add route handlers:
- `GET /settings`
- `PATCH /settings`
- `GET /strategy/stats`
- `GET /forecast/btc-15m`

**Step 3: Re-run API tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api -q`
Expected: PASS.

### Task 3: Add runtime setting validation tests (red-green)

**Files:**
- Modify: `crates/api/src/lib.rs`
- Test: `crates/api/src/lib.rs`

**Step 1: Write failing tests for bad PATCH payloads**

```rust
#[tokio::test]
async fn patch_settings_rejects_invalid_risk_values() {}

#[tokio::test]
async fn patch_settings_rejects_live_mode_when_feature_disabled() {}
```

**Step 2: Run tests to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api patch_settings_rejects_invalid_risk_values patch_settings_rejects_live_mode_when_feature_disabled -q`
Expected: FAIL.

**Step 3: Implement minimal validation in PATCH handler**

Validate:
- lag threshold > 0 and <= 100
- risk per trade > 0 and <= 100
- daily cap > 0 and <= 100
- reject `execution_mode=live` if `live_feature_enabled=false`

**Step 4: Re-run API tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p api -q`
Expected: PASS.

### Task 4: Wire lab-server loop to apply settings and publish KPI/forecast/log telemetry

**Files:**
- Modify: `crates/lab-server/src/main.rs`
- Modify: `crates/lab-server/src/config.rs`
- Modify: `crates/lab-server/src/predictors.rs` (if needed)
- Test: `crates/lab-server/src/main.rs`

**Step 1: Write failing runtime tests**

```rust
#[test]
fn paused_settings_prevent_new_paper_intents() {}

#[test]
fn strategy_stats_compute_win_rate_for_closed_trades() {}
```

**Step 2: Run tests to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server paused_settings_prevent_new_paper_intents strategy_stats_compute_win_rate_for_closed_trades -q`
Expected: FAIL.

**Step 3: Implement minimal runtime wiring**

- Initialize `RuntimeSettings` in state from env config.
- Read settings each loop tick; apply pause/mode/risk values.
- Publish `StrategyStatsSummary` and `BtcForecastSummary` every tick.
- Emit chat-ready `ExecutionLogEntry` events for intent/fill/reject/settings.
- Restrict market presentation metadata to BTC + 15m horizon.

**Step 4: Re-run lab-server tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p lab-server -q`
Expected: PASS.

### Task 5: Replace dashboard shell with requested layout and controls

**Files:**
- Modify: `crates/ui/static/index.html`
- Modify: `crates/ui/static/styles.css`
- Modify: `crates/ui/src/lib.rs`
- Test: `crates/ui/src/lib.rs`

**Step 1: Write failing UI structure tests**

```rust
#[test]
fn ui_shell_contains_top_kpis_requested_by_user() {}

#[test]
fn ui_shell_contains_left_settings_middle_dashboard_right_chat_logs() {}
```

**Step 2: Run tests to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui ui_shell_contains_top_kpis_requested_by_user ui_shell_contains_left_settings_middle_dashboard_right_chat_logs -q`
Expected: FAIL.

**Step 3: Implement markup/styles**

- Top KPI strip: balance, total pnl, exec latency, win rate, BTC/USD.
- Left settings panel form.
- Center dashboard panels (equity + BTC 15m forecast).
- Right chat-style execution logs panel.
- Responsive behavior for mobile.

**Step 4: Re-run UI tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui -q`
Expected: PASS.

### Task 6: Wire frontend JS for settings edits and realtime telemetry

**Files:**
- Modify: `crates/ui/static/app.js`
- Modify: `crates/ui/src/lib.rs`
- Test: `crates/ui/src/lib.rs`

**Step 1: Write failing JS behavior tests**

```rust
#[test]
fn app_js_patches_settings_and_handles_settings_updated_event() {}

#[test]
fn app_js_renders_strategy_stats_forecast_and_chat_logs() {}
```

**Step 2: Run tests to verify failure**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui app_js_patches_settings_and_handles_settings_updated_event app_js_renders_strategy_stats_forecast_and_chat_logs -q`
Expected: FAIL.

**Step 3: Implement JS wiring**

- Bootstrap via `GET /settings`, `GET /strategy/stats`, `GET /forecast/btc-15m`.
- PATCH settings form submit.
- Route websocket events for settings, stats, forecast, logs.
- Render right-rail logs as chat bubbles with auto-trim.

**Step 4: Re-run UI tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo test -p ui -q`
Expected: PASS.

### Task 7: End-to-end verification

**Files:**
- No new files expected

**Step 1: Format and run full tests**

Run: `PATH="$HOME/.cargo/bin:$PATH" cargo fmt --all && PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q`
Expected: PASS.

**Step 2: Run server and verify routes**

Run:
- `curl -sS http://127.0.0.1:8080/settings`
- `curl -sS http://127.0.0.1:8080/strategy/stats`
- `curl -sS http://127.0.0.1:8080/forecast/btc-15m`

Expected: valid JSON payloads and fixed 15m horizon.

**Step 3: Manual UI check**

Open `http://127.0.0.1:8080/` and verify:
- Top KPI strip values update.
- Left settings change runtime values.
- Center dashboard shows BTC 15m forecast.
- Right chat logs stream execution events.
