use crate::events::{RuntimeEvent, RuntimeStage};

pub struct SimEngine {
    _seed: u64,
    tick: u64,
}

impl SimEngine {
    pub fn for_test_seed(seed: u64) -> Self {
        Self {
            _seed: seed,
            tick: 0,
        }
    }

    pub async fn step_once(&mut self) -> Vec<RuntimeEvent> {
        self.tick += 1;
        tokio::task::yield_now().await;

        vec![
            RuntimeEvent::new(self.tick, RuntimeStage::TickStarted),
            RuntimeEvent::new(self.tick, RuntimeStage::MarketDataApplied),
            RuntimeEvent::new(self.tick, RuntimeStage::SignalsGenerated),
            RuntimeEvent::new(self.tick, RuntimeStage::OrdersSimulated),
            RuntimeEvent::new(self.tick, RuntimeStage::PortfolioUpdated),
        ]
    }
}
