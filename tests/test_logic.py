from sim.models import Config
from sim.engine import State, maybe_trade


def test_no_trade_below_threshold():
    s = State()
    cfg = Config(threshold=0.01)
    action, _ = maybe_trade(s, cfg, external_px=100.2, market_px=100.0)
    assert action == "hold"


def test_trade_above_threshold():
    s = State()
    cfg = Config(threshold=0.001)
    action, _ = maybe_trade(s, cfg, external_px=101.0, market_px=100.0)
    assert action in ("buy", "sell")
