from dataclasses import dataclass

@dataclass
class Config:
    threshold: float = 0.003
    max_position_pct: float = 0.005
    daily_loss_cap_pct: float = 0.02
    market_lag_ms: int = 120
    decision_interval_ms: int = 50
    fee_bps: float = 2.0
