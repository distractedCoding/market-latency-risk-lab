# Rust Latency Risk Lab Design

Date: 2026-02-28
Status: Approved for planning
Scope: Full rewrite from Python CLI simulator to Rust monolith with a real-time web dashboard

## 1. Context

The current project is a Python-based educational simulator focused on latency-driven pricing divergence.
The requested rewrite keeps the project simulation-only, but raises fidelity and performance while adding a live dashboard UX.

The design intentionally models claims from high-speed "latency bot" narratives in a controlled environment for education and risk research, not live deployment.

## 2. Goals and Non-Goals

### Goals

- Keep the project simulation-only and safe for research/classroom use.
- Rebuild the engine in Rust for low-latency, high-throughput event processing.
- Support high-fidelity market behavior simulation (order book, queueing, latency, rate limits).
- Provide a web dashboard for live controls, telemetry, and replay views.
- Preserve deterministic reproducibility via seeded runs and structured logs.

### Non-Goals

- No live exchange API execution.
- No real credential handling for trading venues.
- No claims of real-world profitability or production trading readiness.

## 3. Evaluated Approaches

1. Rust monolith (selected)
   - Single Rust executable serving API + WebSocket + static dashboard.
   - Pros: simplest operations, best runtime cohesion, fastest delivery.
   - Cons: frontend flexibility lower than fully separate SPA stack.

2. Rust API + React UI
   - Pros: maximum UI flexibility and ecosystem tooling.
   - Cons: two build/deploy pipelines and extra integration overhead.

3. Rust core + Python shell
   - Pros: lower initial migration risk.
   - Cons: mixed-stack complexity and long-term maintenance cost.

## 4. System Architecture

The runtime is an event-driven Rust monolith built on `tokio` + `axum` with three major planes:

- Simulation plane: deterministic event producers and execution simulator.
- Strategy/risk plane: divergence detection, sizing, and guardrails.
- Interface plane: HTTP control API, WebSocket streams, and static dashboard assets.

High-level flow:

`prediction_tick -> market_tick -> signal -> risk_check -> sim_order -> fill -> portfolio_update -> metrics_publish`

Each event carries monotonic timestamps for precise latency accounting.

## 5. Module Layout

- `crates/core-sim/`
  - Price process and scenario generators
  - Market lag/transport delay model
  - Order book + matching/fill simulation
  - Fee/slippage and queue-position emulation

- `crates/strategy/`
  - Divergence detector (prediction vs market)
  - Regime classifier (trend vs sideways)
  - Position sizing and trade intent
  - Hard risk limits and kill-switch rules

- `crates/runtime/`
  - Async task orchestration and supervision
  - Bounded channels and backpressure management
  - Run lifecycle controls (`start`, `pause`, `stop`, `reset`)
  - Metrics aggregation and replay export

- `crates/api/`
  - `axum` HTTP endpoints for config/control/state
  - WebSocket broadcast streams for live UI updates

- `crates/ui/`
  - Static dashboard assets served by backend
  - Live charts/panels for risk and performance telemetry

- `bin/lab-server`
  - Main executable
  - Profile/config loading (`dev`, `demo`, `bench`)

## 6. Data Flow and Timing Model

### Event stream stages

1. Prediction engine emits synthetic forecast ticks.
2. Market engine emits lagged/perturbed market ticks.
3. Strategy engine computes divergence and candidate actions.
4. Risk engine enforces position and loss constraints.
5. Execution simulator applies queueing, delay, and fill rules.
6. Portfolio engine updates cash, position, equity, realized/unrealized PnL.
7. Metrics engine publishes snapshots/events to WebSocket + logs.

### Required timestamps per event

- `created_at`
- `received_at`
- `acted_at`
- `filled_at`

Derived KPIs include decision latency, simulated exchange latency, end-to-end latency, and p50/p95/p99 summaries.

## 7. Risk and Failure Handling

- Position cap: max 0.5% capital per trade intent.
- Daily realized loss cap: 2%; breach triggers immediate halt.
- Kill-switch on invariant breaches (state inconsistency, unrecoverable task failures).
- Supervisor restarts non-critical tasks (for example, dropped UI broadcaster).
- Critical failures freeze simulation and emit explicit halt reason + terminal snapshot.

Backpressure policy:

- Lossless channels for risk/accounting paths.
- Lossy drop-oldest channels for UI streams so dashboard pressure never blocks core simulation.

## 8. API and Dashboard Contract

### Control endpoints

- `POST /runs` start run
- `POST /runs/{id}/pause` pause run
- `POST /runs/{id}/resume` resume run
- `POST /runs/{id}/stop` stop run
- `GET /state` current engine and risk state
- `PUT /config` runtime configuration update (validated)
- `GET /metrics` aggregate metrics snapshot

### WebSocket channels

- `events` unified event tape
- `orders` simulated order intents
- `fills` simulated executions
- `risk` guardrail and halt signals
- `latency` rolling latency distributions

### Dashboard panels

- Live event tape
- Trigger rate and divergence histogram
- Latency histogram and percentiles
- PnL/equity curve + drawdown
- Position/risk status + halt reason banner

## 9. Testing and Benchmarking Strategy

- Unit tests for divergence math, sizing, fees, risk invariants.
- Deterministic scenario tests using seeded RNG and snapshot assertions.
- Integration tests for complete run lifecycle and API control surface.
- Property checks for accounting invariants.
- Benchmarks for throughput (>=1000 simulated orders/sec) and pipeline latency (report p99).

## 10. Migration Plan (High-Level)

1. Stand up Rust workspace and baseline server.
2. Port simulation primitives and parity-check against current Python outputs.
3. Add strategy/risk modules and deterministic tests.
4. Add API/WebSocket streams and static dashboard.
5. Add benchmark suite and replay/report outputs.
6. Deprecate Python runtime after Rust feature parity is reached.

## 11. Safety and Project Positioning

All examples, docs, and defaults will keep this project in educational simulation scope.
Repository docs will clearly state that no live market integration is provided in this rewrite.
