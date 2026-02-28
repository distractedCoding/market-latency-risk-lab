use crate::events::{RuntimeEvent, RuntimeStage};
use crate::live_runner::{self, JoinedLiveInputs};

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

    pub async fn step_live_once(&mut self, joined: JoinedLiveInputs) -> Vec<RuntimeEvent> {
        self.tick += 1;
        live_runner::run_paper_live_once(self.tick, &joined)
    }
}

#[cfg(test)]
mod tests {
    use super::SimEngine;
    use crate::events::RuntimeStage;
    use crate::live::{BtcMedianTick, PolymarketQuoteTick};
    use crate::live_runner::JoinedLiveInputs;

    #[tokio::test]
    async fn live_runner_emits_intent_then_fill_events() {
        let mut engine = SimEngine::for_test_seed(7);
        let out = engine.step_live_once(synthetic_joined_live_inputs(1)).await;

        assert_eq!(out.len(), 2);
        assert_eq!(out[0].stage, RuntimeStage::PaperIntentCreated);
        assert_eq!(out[1].stage, RuntimeStage::PaperFillRecorded);
    }

    fn synthetic_joined_live_inputs(tick: u64) -> JoinedLiveInputs {
        JoinedLiveInputs {
            btc_tick: BtcMedianTick::new(64_000.0, 8.0, 3, tick),
            quote_tick: PolymarketQuoteTick {
                market_slug: "btc-up-down".to_string(),
                best_yes_bid: 0.48,
                best_yes_ask: 0.52,
                mid_yes: 0.50,
                ts: tick,
            },
        }
    }
}
