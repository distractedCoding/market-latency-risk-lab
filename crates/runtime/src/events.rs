#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeStage {
    TickStarted,
    MarketDataApplied,
    SignalsGenerated,
    OrdersSimulated,
    PortfolioUpdated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeEvent {
    pub tick: u64,
    pub stage: RuntimeStage,
}

impl RuntimeEvent {
    pub fn new(tick: u64, stage: RuntimeStage) -> Self {
        Self { tick, stage }
    }
}
