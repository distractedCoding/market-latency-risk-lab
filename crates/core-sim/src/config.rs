#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimConfig {
    pub threshold: f64,
    pub max_position_pct: f64,
    pub daily_loss_cap_pct: f64,
    pub market_lag_ms: u64,
    pub decision_interval_ms: u64,
    pub fee_bps: f64,
}

impl Default for SimConfig {
    fn default() -> Self {
        Self {
            threshold: 0.003,
            max_position_pct: 0.005,
            daily_loss_cap_pct: 0.02,
            market_lag_ms: 120,
            decision_interval_ms: 50,
            fee_bps: 2.0,
        }
    }
}
