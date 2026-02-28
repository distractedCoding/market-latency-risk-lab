# Polymarket Lag Refinement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Deliver local Rust lag refinement with predictor fusion, lag-triggered execution, and hard risk controls.

**Architecture:** Extend existing paper-live runtime with predictor ingestion and lag detection; gate execution by mode and feature flags; surface strategy telemetry in API/UI routes.

**Tech Stack:** Rust, tokio, axum, serde, reqwest, workspace tests.

---

### Task 1: Config and defaults

**Files:**
- Modify: `crates/lab-server/src/config.rs`
- Test: `crates/lab-server/src/config.rs`

**Step 1:** Add failing tests for execution mode and lag/risk defaults.
**Step 2:** Run `cargo test -p lab-server <test-name> -q` and verify failure.
**Step 3:** Implement config parsing for `LAB_EXECUTION_MODE`, `LAB_LIVE_FEATURE_ENABLED`, `LAB_LAG_THRESHOLD_PCT`, `LAB_RISK_PER_TRADE_PCT`, and `LAB_DAILY_LOSS_CAP_PCT`.
**Step 4:** Re-run `cargo test -p lab-server -q` and verify pass.

### Task 2: Predictor and lag models

**Files:**
- Create: `crates/runtime/src/live/predictors.rs`
- Create: `crates/runtime/src/live/lag_detector.rs`
- Modify: `crates/runtime/src/live/mod.rs`

**Step 1:** Add failing tests for fresh/stale fusion and 0.3% lag trigger boundaries.
**Step 2:** Run runtime tests and verify failures.
**Step 3:** Implement predictor fusion + lag detector types/functions.
**Step 4:** Re-run `cargo test -p runtime -q` and verify pass.

### Task 3: Predictor payload parsers

**Files:**
- Create: `crates/lab-server/src/predictors.rs`
- Modify: `crates/lab-server/src/main.rs`

**Step 1:** Add failing tests for TradingView and CryptoQuant parse helpers.
**Step 2:** Run `cargo test -p lab-server <test-name> -q` and verify failures.
**Step 3:** Implement parser helpers to normalized `PredictorTick`.
**Step 4:** Re-run `cargo test -p lab-server -q` and verify pass.

### Task 4: Lag-aware runner and risk integration

**Files:**
- Modify: `crates/runtime/src/live_runner.rs`
- Modify: `crates/strategy/src/risk.rs`
- Modify: `crates/strategy/src/divergence.rs`

**Step 1:** Add failing tests for lag-triggered intent emission and per-trade risk cap.
**Step 2:** Run targeted tests and verify failures.
**Step 3:** Implement `run_paper_live_once_with_lag` and `check_per_trade_risk`.
**Step 4:** Re-run `cargo test -p runtime -q && cargo test -p strategy -q` and verify pass.

### Task 5: API strategy telemetry

**Files:**
- Modify: `crates/api/src/state.rs`
- Modify: `crates/api/src/routes.rs`
- Modify: `crates/api/src/lib.rs`
- Modify: `crates/lab-server/src/main.rs`

**Step 1:** Add failing tests for `/strategy/perf` and websocket `strategy_perf` payload.
**Step 2:** Run `cargo test -p api <test-name> -q` and verify failures.
**Step 3:** Implement `StrategyPerfSummary` state, route, event constructor, and live-loop publishing.
**Step 4:** Re-run `cargo test -p api -q` and verify pass.

### Task 6: Docs and final verification

**Files:**
- Modify: `README.md`
- Modify: `docs/operations/paper-live-checklist.md`

**Step 1:** Update env var and strategy telemetry documentation.
**Step 2:** Run `cargo fmt --all`.
**Step 3:** Run `cargo test --workspace -q`.
**Step 4:** Verify runtime endpoints: `/prices/snapshot`, `/strategy/perf`, `/feed/health`, `/portfolio/summary`.
