#[derive(Debug, Clone)]
pub struct PriceGenerator {
    state: u64,
    price: f64,
    max_step: f64,
}

impl PriceGenerator {
    pub fn new(seed: u64, start_price: f64, max_step: f64) -> Self {
        Self {
            state: seed,
            price: start_price,
            max_step,
        }
    }

    pub fn next_price(&mut self) -> f64 {
        let unit = next_unit(&mut self.state);
        let delta = (unit * 2.0 - 1.0) * self.max_step;
        self.price = (self.price + delta).max(0.0);
        self.price
    }
}

#[derive(Debug, Clone)]
pub struct MarketLagGenerator {
    state: u64,
    base_lag_ms: u64,
    jitter_ms: u64,
}

impl MarketLagGenerator {
    pub fn new(seed: u64, base_lag_ms: u64, jitter_ms: u64) -> Self {
        Self {
            state: seed,
            base_lag_ms,
            jitter_ms,
        }
    }

    pub fn next_lag_ms(&mut self) -> u64 {
        if self.jitter_ms == 0 {
            return self.base_lag_ms;
        }

        let span = self.jitter_ms.saturating_mul(2).saturating_add(1);
        let offset = next_u64(&mut self.state) % span;
        self.base_lag_ms
            .saturating_sub(self.jitter_ms)
            .saturating_add(offset)
    }
}

fn next_u64(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

fn next_unit(state: &mut u64) -> f64 {
    let value = next_u64(state);
    (value as f64) / (u64::MAX as f64)
}

#[cfg(test)]
mod tests {
    use super::{MarketLagGenerator, PriceGenerator};

    #[test]
    fn seeded_generators_are_deterministic() {
        let mut price_a = PriceGenerator::new(42, 100.0, 0.5);
        let mut price_b = PriceGenerator::new(42, 100.0, 0.5);

        let mut lag_a = MarketLagGenerator::new(42, 100, 40);
        let mut lag_b = MarketLagGenerator::new(42, 100, 40);

        let ticks_a: Vec<(f64, u64)> = (0..10)
            .map(|_| (price_a.next_price(), lag_a.next_lag_ms()))
            .collect();

        let ticks_b: Vec<(f64, u64)> = (0..10)
            .map(|_| (price_b.next_price(), lag_b.next_lag_ms()))
            .collect();

        assert_eq!(ticks_a, ticks_b);
    }
}
