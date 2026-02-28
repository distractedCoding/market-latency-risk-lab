use crate::events::{RuntimeEvent, RuntimeStage};
use crate::live_runner;

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

        vec![
            RuntimeEvent::new(self.tick, RuntimeStage::TickStarted),
            RuntimeEvent::new(self.tick, RuntimeStage::MarketDataApplied),
            RuntimeEvent::new(self.tick, RuntimeStage::SignalsGenerated),
            RuntimeEvent::new(self.tick, RuntimeStage::OrdersSimulated),
            RuntimeEvent::new(self.tick, RuntimeStage::PortfolioUpdated),
        ]
    }

    pub async fn step_live_once(&mut self) -> Vec<RuntimeEvent> {
        self.tick += 1;
        live_runner::run_paper_live_once(self.tick)
    }
}

#[cfg(test)]
mod tests {
    use super::SimEngine;
    use crate::events::RuntimeStage;

    #[tokio::test]
    async fn live_runner_emits_intent_then_fill_events() {
        let mut engine = SimEngine::for_test_seed(7);
        let out = engine.step_live_once().await;
        assert!(
            out.iter()
                .any(|e| e.stage == RuntimeStage::PaperIntentCreated)
        );
        assert!(
            out.iter()
                .any(|e| e.stage == RuntimeStage::PaperFillRecorded)
        );
    }
}
