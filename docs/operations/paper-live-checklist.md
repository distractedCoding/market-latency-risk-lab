# Paper-Live Operator Checklist

Use this checklist before and during `paper-live` runs.

Safety boundary is unchanged:
- no real-money execution
- no live order routing
- no private key based execution paths

## 1. Pre-Start

- Confirm mode intent: `LAB_SERVER_MODE` is unset (defaults to `paper-live`) or explicitly set to `paper-live`.
- Confirm bind address: `LAB_SERVER_ADDR` is valid for the host (default `0.0.0.0:8080`).
- Confirm replay output path: `LAB_SERVER_REPLAY_OUTPUT` is writable (default `artifacts/replay.csv`).
- Confirm execution mode: `LAB_EXECUTION_MODE` is `paper` unless explicitly running gated live shadow mode.
- Confirm feature gate: `LAB_LIVE_FEATURE_ENABLED` is `false` unless intentionally enabling live path checks.
- Confirm risk and lag defaults (or overrides):
  - `LAB_LAG_THRESHOLD_PCT` (default `0.3`)
  - `LAB_RISK_PER_TRADE_PCT` (default `0.5`)
  - `LAB_DAILY_LOSS_CAP_PCT` (default `2.0`)
- Run test gate from repo root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q
```

## 2. Startup

From repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server
```

Expected startup signal in logs:
- `lab-server startup mode: paper-live`
- Note: this confirms mode parsing/banner output in the current build; full mode-specific runtime behavior is still being connected.

## 3. Health Checks

In a second terminal, validate service liveness:

```bash
curl -fsS http://127.0.0.1:8080/health
```

Expected output:
- `ok`

Validate feed mode:

```bash
curl -fsS http://127.0.0.1:8080/feed/health
```

Expected JSON includes:
- `"mode":"paper-live"`
- `"source_counts"`

Validate strategy telemetry:

```bash
curl -fsS http://127.0.0.1:8080/strategy/perf
```

Expected JSON includes:
- `"execution_mode"`
- `"lag_threshold_pct"`
- `"decision_p95_us"`
- `"intents_per_sec"` and `"fills_per_sec"`
- `"halted"`

## 4. Runtime Monitoring

- Watch logs for mode drift or startup retries.
- Confirm replay file exists and updates at `LAB_SERVER_REPLAY_OUTPUT`.
- Monitor `strategy/perf` for decision-latency drift, low throughput, or `halted=true`.

## 5. Stop Conditions

Stop the run and investigate if any of the following occurs:
- health endpoint is unavailable
- `/feed/health` mode is not `paper-live` when paper-live is required
- replay output cannot be written
- any behavior suggests live order submission or real-money execution intent
