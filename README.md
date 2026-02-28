# Market Latency Risk Lab

Educational, closed-environment simulation to demonstrate how latency gaps can create exploitable pricing inefficiencies — and why this is a market integrity risk.

## Scope
This project is **simulation-only**:
- no live exchange APIs
- no order routing to real venues
- no real-money execution

## What it models
- external signal feed ("prediction")
- slower market feed with configurable delay
- divergence trigger (default `>0.3%`)
- sub-100ms decision loop
- micro-trade style paper execution
- risk caps (max position %, daily loss cap, kill switch)

## Quickstart
```bash
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
python -m sim.main --steps 3000 --threshold 0.003 --output artifacts/run.csv
```

## Outputs
- `artifacts/run.csv` trade/event log
- summary in terminal (PnL, max drawdown, trigger count)

## Class demo ideas
- vary latency (`--market-lag-ms`) and compare PnL / risk
- vary threshold (`--threshold`) to show false positives
- enable stress mode for sudden volatility bursts

See `docs/methodology.md` for assumptions and limitations.

## Runtime benchmarks
Run runtime tests and benchmarks from the repository root:

```bash
PATH="$HOME/.cargo/bin:$PATH" cargo test -p runtime -q
PATH="$HOME/.cargo/bin:$PATH" cargo bench -p runtime --no-fail-fast
```

Benchmark outputs include throughput timing for batched `step_once` execution and latency percentiles (`p50`, `p95`, `p99`) with the runtime budget derived from `TARGET_ORDERS_PER_SEC`.
