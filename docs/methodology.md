# Methodology

This lab reproduces the *structure* of latency-sensitive trading claims in a controlled sandbox.

## Runtime and Workflow
- Rust workspace is the source of truth (`cargo test --workspace`, `cargo run -p lab-server`).
- `lab-server` defaults to `paper-live` mode at the configuration/interface level.
- `LAB_SERVER_MODE` is currently parsed and surfaced in the startup banner; full mode-specific execution paths are still being wired.
- `lab-server` exposes runtime services and health endpoints for local study.
- Default server bind is `0.0.0.0:8080` and can be overridden with `LAB_SERVER_ADDR`.
- Runtime mode can be overridden with `LAB_SERVER_MODE` (`paper-live` default, `sim` fallback).
- Replay output defaults to `artifacts/replay.csv` and can be overridden with `LAB_SERVER_REPLAY_OUTPUT`.

## Assumptions
- External signal is synthetic stochastic process.
- Market feed is lagged copy of external feed.
- Orders always fill at observed market price (simplified).
- Costs represented as basis-point fee only.

## Limitations
- No real market microstructure
- No queue priority / matching-engine dynamics
- No adversarial participants
- No legal/regulatory execution inferences
- No live order routing or private-key order submission
- No real-money execution
- No production trading deployment support

Use this to study **risk mechanics**, not to deploy real trading systems.
