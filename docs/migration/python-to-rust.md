# Python to Rust Migration

## Status
Python simulator entrypoints have been retired. The Rust workspace is now the only supported runtime for paper-live and simulation server workflows.

## What Changed
- Removed legacy Python simulator files under `sim/`.
- Removed legacy Python logic tests under `tests/`.
- Updated top-level docs to use Cargo-based workflows.
- Removed legacy root `requirements.txt`; Cargo manifests are now authoritative.

## Supported Workflow
Run from repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test --workspace -q
PATH="$HOME/.cargo/bin:$PATH" cargo run -p lab-server
```

Optional server configuration:
- `LAB_SERVER_ADDR` to override listen address (default `0.0.0.0:8080`)
- `LAB_SERVER_MODE` to switch runtime mode (default `paper-live`, optional `sim`)
- `LAB_SERVER_REPLAY_OUTPUT` to override replay path (default `artifacts/replay.csv`)

## Scope Reminder
This project now defaults configuration to paper-live:
- paper-live scaffolding and default contract are present
- `LAB_SERVER_MODE` is parsed and shown at startup, while live ingest and full mode-specific runtime wiring are still in progress
- no order routing to real venues
- no real-money execution
- no private key based execution paths
