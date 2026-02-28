# Market Latency Risk Lab

Educational, closed-environment lab for studying latency-driven market risk mechanics.

## Scope
This project defaults to **live-data paper trading**:
- live market data ingestion for paper decisions
- no order routing to real venues
- no real-money execution
- no private key based execution paths

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
- `LAB_SERVER_MODE` (default `paper-live`; set to `sim` to opt out)
- `LAB_SERVER_REPLAY_OUTPUT` (default `artifacts/replay.csv`)

Example:

```bash
LAB_SERVER_ADDR="127.0.0.1:8080" \
LAB_SERVER_REPLAY_OUTPUT="artifacts/replay.csv" \
PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server
```

## Runtime Benchmarks
Run runtime tests and benchmarks from the repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime -q
PATH="$HOME/.cargo/bin:$PATH" cargo bench -p runtime --no-fail-fast
```

See `docs/methodology.md` for assumptions and limitations and `docs/migration/python-to-rust.md` for migration details.
For runbook checks before starting live-data paper mode, see `docs/operations/paper-live-checklist.md`.
