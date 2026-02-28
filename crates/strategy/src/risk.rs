use crate::divergence::StrategyError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RiskState {
    starting_equity: f64,
    realized_pnl: f64,
    daily_loss_cap_pct: f64,
    halted: bool,
}

impl RiskState {
    pub fn new(starting_equity: f64, daily_loss_cap_pct: f64) -> Result<Self, StrategyError> {
        if !starting_equity.is_finite() || starting_equity <= 0.0 {
            return Err(StrategyError::InvalidStartingEquity);
        }
        if !daily_loss_cap_pct.is_finite() || !(0.0..=1.0).contains(&daily_loss_cap_pct) {
            return Err(StrategyError::InvalidDailyLossCapPct);
        }

        Ok(Self {
            starting_equity,
            realized_pnl: 0.0,
            daily_loss_cap_pct,
            halted: false,
        })
    }

    pub fn apply_realized_pnl(&mut self, pnl_delta: f64) -> Result<(), StrategyError> {
        if !pnl_delta.is_finite() {
            return Err(StrategyError::NonFinitePnl);
        }

        self.realized_pnl += pnl_delta;

        let cap_amount = self.exposure_cap_amount();
        if self.realized_pnl <= -cap_amount {
            self.halted = true;
        }

        Ok(())
    }

    pub fn trigger_kill_switch(&mut self) {
        self.halted = true;
    }

    pub fn is_halted(&self) -> bool {
        self.halted
    }

    pub fn check_market_exposure(
        &self,
        market_id: &str,
        current_market_exposure: f64,
        intent_notional: f64,
    ) -> Result<(), StrategyError> {
        if market_id.trim().is_empty() {
            return Err(StrategyError::InvalidMarketId);
        }
        if !current_market_exposure.is_finite() || !intent_notional.is_finite() {
            return Err(StrategyError::NonFiniteMarketExposure);
        }
        if current_market_exposure < 0.0 || intent_notional < 0.0 {
            return Err(StrategyError::InvalidMarketExposure);
        }

        let projected_market_exposure = current_market_exposure + intent_notional;
        if projected_market_exposure > self.exposure_cap_amount() {
            return Err(StrategyError::MarketExposureCapExceeded);
        }

        Ok(())
    }

    fn exposure_cap_amount(&self) -> f64 {
        self.starting_equity * self.daily_loss_cap_pct
    }
}

#[cfg(test)]
mod tests {
    use super::RiskState;
    use crate::divergence::StrategyError;

    #[test]
    fn rejects_intent_when_market_exposure_cap_exceeded() {
        let risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        let decision = risk.check_market_exposure("btc-up", 10_000.0, 2_000.0);

        assert!(decision.is_err());
    }

    #[test]
    fn allows_intent_when_market_exposure_is_within_cap() {
        let risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        let decision = risk.check_market_exposure("btc-up", 1_000.0, 900.0);

        assert_eq!(decision, Ok(()));
    }

    #[test]
    fn rejects_invalid_market_exposure_inputs() {
        let risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        assert_eq!(
            risk.check_market_exposure("", 1_000.0, 500.0),
            Err(StrategyError::InvalidMarketId)
        );
        assert_eq!(
            risk.check_market_exposure("btc-up", f64::NAN, 500.0),
            Err(StrategyError::NonFiniteMarketExposure)
        );
        assert_eq!(
            risk.check_market_exposure("btc-up", 1_000.0, f64::INFINITY),
            Err(StrategyError::NonFiniteMarketExposure)
        );
        assert_eq!(
            risk.check_market_exposure("btc-up", -1.0, 500.0),
            Err(StrategyError::InvalidMarketExposure)
        );
        assert_eq!(
            risk.check_market_exposure("btc-up", 1_000.0, -1.0),
            Err(StrategyError::InvalidMarketExposure)
        );
    }

    #[test]
    fn halts_when_daily_loss_cap_is_breached() {
        let mut risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        risk.apply_realized_pnl(-2_001.0).expect("valid pnl update");

        assert!(risk.is_halted());
    }

    #[test]
    fn halts_when_daily_loss_reaches_exact_cap_boundary() {
        let mut risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        risk.apply_realized_pnl(-2_000.0).expect("valid pnl update");

        assert!(risk.is_halted());
    }

    #[test]
    fn allows_manual_kill_switch_trigger() {
        let mut risk = RiskState::new(100_000.0, 0.02).expect("valid risk state");

        risk.trigger_kill_switch();

        assert!(risk.is_halted());
    }

    #[test]
    fn rejects_invalid_daily_loss_cap_fraction_values() {
        assert_eq!(
            RiskState::new(100_000.0, -0.01),
            Err(StrategyError::InvalidDailyLossCapPct)
        );
        assert_eq!(
            RiskState::new(100_000.0, 1.01),
            Err(StrategyError::InvalidDailyLossCapPct)
        );
        assert_eq!(
            RiskState::new(100_000.0, f64::NAN),
            Err(StrategyError::InvalidDailyLossCapPct)
        );
        assert_eq!(
            RiskState::new(100_000.0, f64::INFINITY),
            Err(StrategyError::InvalidDailyLossCapPct)
        );
    }
}
