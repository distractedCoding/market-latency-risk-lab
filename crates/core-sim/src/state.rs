#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimState {
    pub equity: f64,
    pub cash: f64,
    pub position: f64,
    pub avg_price: f64,
    pub realized_pnl: f64,
    pub halted: bool,
}

impl Default for SimState {
    fn default() -> Self {
        Self {
            equity: 100_000.0,
            cash: 100_000.0,
            position: 0.0,
            avg_price: 0.0,
            realized_pnl: 0.0,
            halted: false,
        }
    }
}
