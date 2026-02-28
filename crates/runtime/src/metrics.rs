#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyPercentiles {
    pub count: usize,
    pub p50_nanos: u64,
    pub p90_nanos: u64,
    pub p95_nanos: u64,
    pub p99_nanos: u64,
    pub max_nanos: u64,
}

#[derive(Debug, Default, Clone)]
pub struct DecisionLatencyMetrics {
    latencies_nanos: Vec<u64>,
}

impl DecisionLatencyMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_latency_nanos(&mut self, latency_nanos: u64) {
        self.latencies_nanos.push(latency_nanos);
    }

    pub fn percentiles(&self) -> Option<LatencyPercentiles> {
        if self.latencies_nanos.is_empty() {
            return None;
        }

        let mut sorted = self.latencies_nanos.clone();
        sorted.sort_unstable();
        let count = sorted.len();

        Some(LatencyPercentiles {
            count,
            p50_nanos: percentile_nearest_rank(&sorted, 50)?,
            p90_nanos: percentile_nearest_rank(&sorted, 90)?,
            p95_nanos: percentile_nearest_rank(&sorted, 95)?,
            p99_nanos: percentile_nearest_rank(&sorted, 99)?,
            max_nanos: sorted[count - 1],
        })
    }

    pub fn percentile_nanos(&self, percentile: usize) -> Option<u64> {
        if self.latencies_nanos.is_empty() {
            return None;
        }

        let mut sorted = self.latencies_nanos.clone();
        sorted.sort_unstable();
        percentile_nearest_rank(&sorted, percentile)
    }
}

fn percentile_nearest_rank(sorted: &[u64], percentile: usize) -> Option<u64> {
    if sorted.is_empty() || !(1..=100).contains(&percentile) {
        return None;
    }

    let count = sorted.len();
    let rank = (percentile * count).div_ceil(100);
    sorted.get(rank.saturating_sub(1)).copied()
}

#[cfg(test)]
mod tests {
    use super::DecisionLatencyMetrics;

    #[test]
    fn percentiles_returns_none_for_empty_input() {
        let metrics = DecisionLatencyMetrics::new();

        assert_eq!(metrics.percentiles(), None);
    }

    #[test]
    fn single_sample_reports_same_value_for_all_percentiles() {
        let mut metrics = DecisionLatencyMetrics::new();
        metrics.record_latency_nanos(42);

        let report = metrics.percentiles().expect("percentiles should exist");

        assert_eq!(report.p50_nanos, 42);
        assert_eq!(report.p90_nanos, 42);
        assert_eq!(report.p95_nanos, 42);
        assert_eq!(report.p99_nanos, 42);
        assert_eq!(report.max_nanos, 42);
    }

    #[test]
    fn supports_boundary_percentile_queries() {
        let mut metrics = DecisionLatencyMetrics::new();
        metrics.record_latency_nanos(10);
        metrics.record_latency_nanos(20);
        metrics.record_latency_nanos(30);

        assert_eq!(metrics.percentile_nanos(1), Some(10));
        assert_eq!(metrics.percentile_nanos(100), Some(30));
    }

    #[test]
    fn invalid_percentile_queries_return_none() {
        let mut metrics = DecisionLatencyMetrics::new();
        metrics.record_latency_nanos(10);

        assert_eq!(metrics.percentile_nanos(0), None);
        assert_eq!(metrics.percentile_nanos(101), None);
    }
}
