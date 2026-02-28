# Market Latency Risk Lab

Educational, closed-environment lab for studying latency-driven market risk mechanics.

## Scope
This project defaults to **paper-live** mode and supports local predictor-driven lag detection:
- live BTC + Polymarket ingest with paper execution loop
- lag trigger model (default 0.3%) using fused predictor inputs
- per-trade and daily risk guardrails (default 0.5% and 2%)
- no real-money execution by default
- live execution mode is feature-gated and disabled by default

Simulation mode remains available as an explicit fallback (`LAB_SERVER_MODE=sim`).

The Rust monolith replaces the legacy Python simulator. All workflows run through Cargo.

## Quickstart (Rust)
From the repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q
PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server
```

The server listens on `0.0.0.0:8080` by default.

## Server Configuration
Use environment variables to override defaults:
- `LAB_SERVER_ADDR` (default `0.0.0.0:8080`)
- `LAB_SERVER_MODE` (default `paper-live`; fallback `sim`)
- `LAB_SERVER_REPLAY_OUTPUT` (default `artifacts/replay.csv`)
- `LAB_EXECUTION_MODE` (`paper` or `live`, default `paper`)
- `LAB_LIVE_FEATURE_ENABLED` (`true`/`false`, default `false`)
- `LAB_LAG_THRESHOLD_PCT` (default `0.3`)
- `LAB_RISK_PER_TRADE_PCT` (default `0.5`)
- `LAB_DAILY_LOSS_CAP_PCT` (default `2.0`)
- `LAB_TRADINGVIEW_PREDICT_URL` (optional predictor endpoint)
- `LAB_CRYPTOQUANT_PREDICT_URL` (optional predictor endpoint)

Example:

```bash
LAB_SERVER_ADDR="127.0.0.1:8080" \
LAB_SERVER_REPLAY_OUTPUT="artifacts/replay.csv" \
LAB_EXECUTION_MODE="paper" \
LAB_LIVE_FEATURE_ENABLED="false" \
PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server
```

## Strategy Telemetry

The runtime publishes strategy performance snapshots at:

```bash
curl -fsS http://127.0.0.1:8080/strategy/perf
```

Payload includes execution mode, lag threshold, decision latency estimate, throughput, lag trigger count, and halt status.

## Runtime Benchmarks
Run runtime tests and benchmarks from the repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime -q
PATH="$HOME/.cargo/bin:$PATH" cargo bench -p runtime --no-fail-fast
```

See `docs/methodology.md` for assumptions and limitations and `docs/migration/python-to-rust.md` for migration details.
For runbook checks before starting live-data paper mode, see `docs/operations/paper-live-checklist.md`.
