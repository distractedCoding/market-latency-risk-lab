from __future__ import annotations
import argparse
from collections import deque
from pathlib import Path
import pandas as pd

from sim.models import Config
from sim.engine import State, step_signal, maybe_trade


def run(steps: int, cfg: Config, stress: bool = False):
    external = 100.0
    lag_ticks = max(1, cfg.market_lag_ms // cfg.decision_interval_ms)
    lag_buffer = deque([external] * lag_ticks, maxlen=lag_ticks)

    state = State()
    rows = []

    for t in range(steps):
        external = step_signal(external, stress=stress)
        lag_buffer.append(external)
        market = lag_buffer[0]

        action, div = maybe_trade(state, cfg, external, market)
        rows.append(
            {
                "t": t,
                "external_px": external,
                "market_px": market,
                "divergence": div,
                "action": action,
                "equity": state.equity,
                "realized_pnl": state.realized_pnl,
                "position": state.position,
                "halted": state.halted,
            }
        )
        if state.halted:
            break

    return pd.DataFrame(rows)


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--steps", type=int, default=5000)
    p.add_argument("--threshold", type=float, default=0.003)
    p.add_argument("--market-lag-ms", type=int, default=120)
    p.add_argument("--decision-interval-ms", type=int, default=50)
    p.add_argument("--output", type=str, default="artifacts/run.csv")
    p.add_argument("--stress", action="store_true")
    a = p.parse_args()

    cfg = Config(
        threshold=a.threshold,
        market_lag_ms=a.market_lag_ms,
        decision_interval_ms=a.decision_interval_ms,
    )
    df = run(a.steps, cfg, stress=a.stress)
    out = Path(a.output)
    out.parent.mkdir(parents=True, exist_ok=True)
    df.to_csv(out, index=False)

    triggers = int((df["action"].isin(["buy", "sell"])) .sum())
    print(f"rows={len(df)} triggers={triggers} final_equity={df['equity'].iloc[-1]:.2f} realized_pnl={df['realized_pnl'].iloc[-1]:.2f}")


if __name__ == "__main__":
    main()
