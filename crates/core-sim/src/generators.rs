#[derive(Debug, Clone)]
pub struct PriceGenerator {
    state: u64,
    price: f64,
    max_step: f64,
}

impl PriceGenerator {
    pub fn new(seed: u64, start_price: f64, max_step: f64) -> Self {
        assert!(
            start_price.is_finite() && start_price >= 0.0,
            "start_price must be finite and non-negative"
        );
        assert!(
            max_step.is_finite() && max_step >= 0.0,
            "max_step must be finite and non-negative"
        );

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

        let min = self.base_lag_ms.saturating_sub(self.jitter_ms);
        let max = self.base_lag_ms.saturating_add(self.jitter_ms);
        let width = max - min;

        if width == u64::MAX {
            return next_u64(&mut self.state);
        }

        let span = width + 1;
        let offset = next_u64(&mut self.state) % span;
        min + offset
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

    #[test]
    fn lag_stays_within_expected_bounds_when_base_is_less_than_jitter() {
        let base = 10_u64;
        let jitter = 50_u64;
        let min = base.saturating_sub(jitter);
        let max = base.saturating_add(jitter);
        let mut lag = MarketLagGenerator::new(7, base, jitter);

        for _ in 0..1_000 {
            let sample = lag.next_lag_ms();
            assert!((min..=max).contains(&sample));
        }
    }

    #[test]
    fn lag_near_u64_max_stays_within_bounds() {
        let base = u64::MAX - 4;
        let jitter = 10_u64;
        let min = base.saturating_sub(jitter);
        let max = base.saturating_add(jitter);
        let mut lag = MarketLagGenerator::new(99, base, jitter);

        for _ in 0..1_000 {
            let sample = lag.next_lag_ms();
            assert!((min..=max).contains(&sample));
        }
    }

    #[test]
    #[should_panic(expected = "start_price must be finite and non-negative")]
    fn price_generator_rejects_invalid_start_price() {
        let _ = PriceGenerator::new(1, f64::NAN, 1.0);
    }

    #[test]
    #[should_panic(expected = "max_step must be finite and non-negative")]
    fn price_generator_rejects_invalid_max_step() {
        let _ = PriceGenerator::new(1, 100.0, -1.0);
    }
}
