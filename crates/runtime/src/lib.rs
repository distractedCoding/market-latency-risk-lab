pub mod engine;
pub mod events;
pub mod supervisor;

pub fn module_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    #[tokio::test(flavor = "current_thread")]
    async fn engine_emits_events_in_expected_order() {
        let mut engine = crate::engine::SimEngine::for_test_seed(7);
        let events = engine.step_once().await;

        assert_eq!(events.len(), 5);
        assert_eq!(events[0].stage, crate::events::RuntimeStage::TickStarted);
        assert_eq!(events[1].stage, crate::events::RuntimeStage::MarketDataApplied);
        assert_eq!(events[2].stage, crate::events::RuntimeStage::SignalsGenerated);
        assert_eq!(events[3].stage, crate::events::RuntimeStage::OrdersSimulated);
        assert_eq!(events[4].stage, crate::events::RuntimeStage::PortfolioUpdated);
    }
}
