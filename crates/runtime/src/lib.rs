pub mod engine;
pub mod events;
pub mod logging;
pub mod metrics;
pub mod supervisor;

pub fn module_ready() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use crate::metrics::DecisionLatencyMetrics;

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

    #[test]
    fn latency_percentiles_are_reported() {
        let mut metrics = DecisionLatencyMetrics::new();

        metrics.record_latency_micros(1);
        metrics.record_latency_micros(2);
        metrics.record_latency_micros(3);
        metrics.record_latency_micros(4);
        metrics.record_latency_micros(100);

        let report = metrics.percentiles().expect("percentiles should exist");

        assert_eq!(report.count, 5);
        assert_eq!(report.p50_micros, 3);
        assert_eq!(report.p95_micros, 100);
        assert_eq!(report.p99_micros, 100);
    }
}
