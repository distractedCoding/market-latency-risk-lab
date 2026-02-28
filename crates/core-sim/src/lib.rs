mod config;
mod generators;
mod state;

pub use config::SimConfig;
pub use generators::{MarketLagGenerator, PriceGenerator};
pub use state::SimState;

pub fn workspace_bootstrap() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{workspace_bootstrap, SimConfig, SimState};

    #[test]
    fn workspace_builds() {
        assert!(workspace_bootstrap());
    }

    #[test]
    fn sim_config_defaults_match_spec() {
        let config = SimConfig::default();
        assert_eq!(config.divergence_threshold, 0.003);
        assert_eq!(config.max_position_pct, 0.005);
        assert_eq!(config.daily_loss_cap_pct, 0.02);
        assert_eq!(config.market_lag_ms, 120);
        assert_eq!(config.decision_interval_ms, 50);
        assert_eq!(config.fee_bps, 2.0);
    }

    #[test]
    fn sim_state_defaults_match_spec() {
        let state = SimState::default();
        assert_eq!(state.equity, 100_000.0);
        assert_eq!(state.cash, 100_000.0);
        assert_eq!(state.position, 0.0);
        assert_eq!(state.avg_price, 0.0);
        assert_eq!(state.realized_pnl, 0.0);
        assert!(!state.halted);
    }
}
