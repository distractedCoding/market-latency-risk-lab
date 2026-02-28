# Methodology

This lab reproduces the *structure* of latency-sensitive trading claims in a controlled sandbox.

## Runtime and Workflow
- Rust workspace is the source of truth (`cargo test --workspace`, `cargo run -p lab-server`).
- `lab-server` exposes simulation services and health endpoints for local study.
- Default server bind is `0.0.0.0:8080` and can be overridden with `LAB_SERVER_ADDR`.
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
- No production trading deployment support

Use this to study **risk mechanics**, not to deploy real trading systems.
