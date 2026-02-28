# Polymarket Lag Refinement Design

Date: 2026-02-28
Status: Approved for implementation
Scope: Local Rust lag-trading refinement with predictor fusion, threshold triggers, and strict risk controls.

## Intent

Refine paper-live flow to fuse TradingView and CryptoQuant predictor inputs, detect Polymarket lag above 0.3%, and route execution through hybrid paper/live mode gates while preserving no-cloud, CPU-only operation.

## Locked Decisions

- `paper` remains default execution mode.
- `live` mode exists behind explicit feature flag.
- Lag trigger threshold default: 0.3%.
- Risk defaults: 0.5% per trade, 2.0% daily loss cap.
- Profit and edge values are target metrics, not guarantees.

## Architecture

- Add predictor normalization and fair-value fusion in `runtime::live`.
- Add lag detector in `runtime::live` using percentage divergence.
- Add lag-aware runner path with risk prechecks before fills.
- Add execution-mode config and feature gate handling in `lab-server`.
- Add API telemetry endpoint for strategy latency/throughput/risk state.

## Safety and Risk

- Hard reject when projected per-trade risk exceeds configured fraction.
- Daily halt when realized PnL breaches configured daily loss cap.
- Live mode disabled unless both mode and feature gate are explicitly enabled.

## Observability

- Keep existing feed, portfolio, and price snapshots.
- Add strategy performance snapshot:
  - execution mode
  - lag threshold
  - decision latency estimate
  - intents/s and fills/s
  - lag trigger count
  - halt status
