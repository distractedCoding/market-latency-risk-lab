#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatencyPercentiles {
    pub count: usize,
    pub p50_micros: u64,
    pub p90_micros: u64,
    pub p95_micros: u64,
    pub p99_micros: u64,
    pub max_micros: u64,
}

#[derive(Debug, Default, Clone)]
pub struct DecisionLatencyMetrics {
    latencies_micros: Vec<u64>,
}

impl DecisionLatencyMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_latency_micros(&mut self, latency_micros: u64) {
        self.latencies_micros.push(latency_micros);
    }

    pub fn percentiles(&self) -> Option<LatencyPercentiles> {
        if self.latencies_micros.is_empty() {
            return None;
        }

        let mut sorted = self.latencies_micros.clone();
        sorted.sort_unstable();
        let count = sorted.len();

        Some(LatencyPercentiles {
            count,
            p50_micros: percentile_nearest_rank(&sorted, 50),
            p90_micros: percentile_nearest_rank(&sorted, 90),
            p95_micros: percentile_nearest_rank(&sorted, 95),
            p99_micros: percentile_nearest_rank(&sorted, 99),
            max_micros: sorted[count - 1],
        })
    }
}

fn percentile_nearest_rank(sorted: &[u64], percentile: usize) -> u64 {
    let count = sorted.len();
    let rank = (percentile * count).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
