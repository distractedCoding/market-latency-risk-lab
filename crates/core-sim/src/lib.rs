mod config;
mod state;

pub use config::SimConfig;
pub use state::SimState;

pub fn workspace_bootstrap() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::{workspace_bootstrap, SimConfig};

    #[test]
    fn workspace_builds() {
        assert!(workspace_bootstrap());
    }

    #[test]
    fn default_risk_limits_match_spec() {
        let config = SimConfig::default();
        assert_eq!(config.max_position_pct, 0.005);
        assert_eq!(config.daily_loss_cap_pct, 0.02);
    }
}
