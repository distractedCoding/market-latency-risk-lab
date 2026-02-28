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

        let cap_amount = self.starting_equity * self.daily_loss_cap_pct;
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
}
