from __future__ import annotations
import random
from dataclasses import dataclass
from sim.models import Config

@dataclass
class State:
    equity: float = 100_000.0
    cash: float = 100_000.0
    position: float = 0.0
    avg_price: float = 0.0
    realized_pnl: float = 0.0
    halted: bool = False


def step_signal(prev: float, stress: bool = False) -> float:
    sigma = 0.001 if not stress else 0.003
    return max(1.0, prev * (1 + random.gauss(0, sigma)))


def maybe_trade(state: State, cfg: Config, external_px: float, market_px: float) -> tuple[str, float]:
    if state.halted:
        return ("halted", 0.0)

    divergence = (external_px - market_px) / market_px
    notional_cap = state.equity * cfg.max_position_pct
    qty = notional_cap / market_px
    fee = (cfg.fee_bps / 10_000.0) * (qty * market_px)

    if abs(divergence) < cfg.threshold:
        return ("hold", 0.0)

    side = 1 if divergence > 0 else -1
    # close opposite
    if state.position != 0 and (state.position > 0) != (side > 0):
        pnl = (market_px - state.avg_price) * state.position
        state.cash += state.position * market_px + pnl
        state.realized_pnl += pnl
        state.position = 0
        state.avg_price = 0

    # open micro position
    state.position += side * qty
    state.avg_price = market_px
    state.cash -= side * qty * market_px
    state.cash -= fee

    # mark-to-market
    mtm = state.position * (market_px - state.avg_price)
    state.equity = state.cash + state.position * market_px + mtm

    if (state.realized_pnl / 100_000.0) <= -cfg.daily_loss_cap_pct:
        state.halted = True
        return ("kill_switch", divergence)

    return ("buy" if side > 0 else "sell", divergence)
