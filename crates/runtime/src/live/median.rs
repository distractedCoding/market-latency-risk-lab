use std::collections::HashMap;

use crate::live::{BtcMedianTick, NormalizedBtcTick};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MedianAggregatorConfigError {
    /// `staleness_ms` must be greater than zero.
    InvalidStalenessMs,
    /// `outlier_bps` must be finite and non-negative.
    InvalidOutlierBps,
}

#[derive(Debug, Clone)]
pub struct MedianAggregator {
    staleness_ms: u64,
    outlier_bps: f64,
    latest_by_venue: HashMap<String, NormalizedBtcTick>,
}

impl MedianAggregator {
    /// Creates a median aggregator with validated runtime parameters.
    ///
    /// - `staleness_ms`: max age (milliseconds) from the freshest venue tick.
    /// - `outlier_bps`: outlier band in basis points around the baseline median.
    ///
    /// Returns an error when `staleness_ms == 0`, or when `outlier_bps` is not
    /// finite or negative.
    pub fn new(staleness_ms: u64, outlier_bps: f64) -> Result<Self, MedianAggregatorConfigError> {
        if staleness_ms == 0 {
            return Err(MedianAggregatorConfigError::InvalidStalenessMs);
        }
        if !outlier_bps.is_finite() || outlier_bps < 0.0 {
            return Err(MedianAggregatorConfigError::InvalidOutlierBps);
        }

        Ok(Self {
            staleness_ms,
            outlier_bps,
            latest_by_venue: HashMap::new(),
        })
    }

    /// Ingests a normalized venue tick into the latest-per-venue cache.
    ///
    /// Ticks with invalid prices (`NaN`, infinite, or `<= 0.0`) are silently
    /// ignored and do not update aggregator state.
    ///
    /// For valid prices, only the newest tick per venue is retained.
    pub fn ingest(&mut self, tick: NormalizedBtcTick) {
        if !tick.px.is_finite() || tick.px <= 0.0 {
            return;
        }

        match self.latest_by_venue.get(&tick.venue) {
            Some(existing) if existing.ts > tick.ts => {}
            _ => {
                self.latest_by_venue.insert(tick.venue.clone(), tick);
            }
        }
    }

    /// Computes a robust median snapshot across currently tracked venues.
    ///
    /// The aggregator starts from the latest tick per venue, removes stale ticks
    /// relative to the freshest timestamp, computes a baseline median, then drops
    /// outliers outside the configured basis-point band.
    ///
    /// Returns `Some(BtcMedianTick)` only when at least two venues survive all
    /// filtering steps.
    ///
    /// Returns `None` when no ticks are available, all ticks are stale, or fewer
    /// than two venues survive staleness/outlier filtering.
    pub fn compute(&self) -> Option<BtcMedianTick> {
        let latest_ts = self.latest_by_venue.values().map(|tick| tick.ts).max()?;

        let fresh_ticks: Vec<&NormalizedBtcTick> = self
            .latest_by_venue
            .values()
            .filter(|tick| latest_ts.saturating_sub(tick.ts) <= self.staleness_ms)
            .collect();

        if fresh_ticks.is_empty() {
            return None;
        }

        let baseline_median = median_price(&fresh_ticks)?;
        let threshold = baseline_median * (self.outlier_bps / 10_000.0);

        let filtered_ticks: Vec<&NormalizedBtcTick> = fresh_ticks
            .into_iter()
            .filter(|tick| (tick.px - baseline_median).abs() <= threshold)
            .collect();

        if filtered_ticks.len() < 2 {
            return None;
        }

        let px_median = median_price(&filtered_ticks)?;
        let min_px = filtered_ticks
            .iter()
            .map(|tick| tick.px)
            .fold(f64::INFINITY, f64::min);
        let max_px = filtered_ticks
            .iter()
            .map(|tick| tick.px)
            .fold(f64::NEG_INFINITY, f64::max);
        let ts = filtered_ticks.iter().map(|tick| tick.ts).max()?;

        Some(BtcMedianTick::new(
            px_median,
            max_px - min_px,
            filtered_ticks.len() as u32,
            ts,
        ))
    }
}

fn median_price(ticks: &[&NormalizedBtcTick]) -> Option<f64> {
    if ticks.is_empty() {
        return None;
    }

    let mut prices: Vec<f64> = ticks.iter().map(|tick| tick.px).collect();
    prices.sort_by(|a, b| a.total_cmp(b));

    let mid = prices.len() / 2;
    if prices.len() % 2 == 0 {
        Some((prices[mid - 1] + prices[mid]) / 2.0)
    } else {
        Some(prices[mid])
    }
}

#[cfg(test)]
mod tests {
    use super::MedianAggregator;
    use crate::live::NormalizedBtcTick;

    #[test]
    fn median_ignores_stale_and_outlier_ticks() {
        let mut agg = MedianAggregator::new(2_000, 200.0).unwrap();

        agg.ingest(tick("binance", 60_000.0, 10_000));
        agg.ingest(tick("coinbase", 60_050.0, 10_500));
        agg.ingest(tick("kraken", 59_980.0, 10_300));

        agg.ingest(tick("old-feed", 60_040.0, 8_000));
        agg.ingest(tick("bad-feed", 70_000.0, 10_400));

        // fresh ticks + one stale + one outlier
        let out = agg.compute().unwrap();
        assert_eq!(out.venue_count, 3);
        assert!(out.px_median > 0.0);
    }

    #[test]
    fn ingest_keeps_latest_tick_per_venue() {
        let mut agg = MedianAggregator::new(5_000, 500.0).unwrap();
        agg.ingest(tick("binance", 61_000.0, 10_100));
        agg.ingest(tick("binance", 60_500.0, 10_000));
        agg.ingest(tick("coinbase", 61_100.0, 10_100));

        let out = agg.compute().unwrap();
        assert_eq!(out.venue_count, 2);
        assert_eq!(out.px_median, 61_050.0);
    }

    #[test]
    fn compute_requires_at_least_two_surviving_venues() {
        let mut agg = MedianAggregator::new(5_000, 0.0).unwrap();
        agg.ingest(tick("binance", 60_000.0, 10_000));
        agg.ingest(tick("coinbase", 60_000.0, 10_100));
        agg.ingest(tick("kraken", 60_100.0, 10_200));

        let out = agg.compute().unwrap();
        assert_eq!(out.venue_count, 2);

        agg.ingest(tick("coinbase", 60_010.0, 10_300));
        assert!(agg.compute().is_none());
    }

    #[test]
    fn ingest_rejects_non_finite_and_non_positive_prices() {
        let mut agg = MedianAggregator::new(5_000, 500.0).unwrap();
        agg.ingest(tick("binance", 61_000.0, 10_100));
        agg.ingest(tick("coinbase", 61_100.0, 10_100));

        let baseline = agg.compute().unwrap();
        assert_eq!(baseline.venue_count, 2);

        agg.ingest(tick("bad-nan", f64::NAN, 10_200));
        agg.ingest(tick("bad-inf", f64::INFINITY, 10_200));
        agg.ingest(tick("bad-zero", 0.0, 10_200));
        agg.ingest(tick("bad-neg", -1.0, 10_200));

        let out = agg.compute().unwrap();
        assert_eq!(out.venue_count, 2);
        assert_eq!(out.px_median, baseline.px_median);
    }

    #[test]
    fn new_rejects_invalid_constructor_params() {
        assert!(MedianAggregator::new(0, 100.0).is_err());
        assert!(MedianAggregator::new(5_000, f64::NAN).is_err());
        assert!(MedianAggregator::new(5_000, f64::INFINITY).is_err());
        assert!(MedianAggregator::new(5_000, -0.1).is_err());
    }

    #[test]
    fn new_accepts_boundary_constructor_params() {
        assert!(MedianAggregator::new(1, 0.0).is_ok());
    }

    fn tick(venue: &str, px: f64, ts: u64) -> NormalizedBtcTick {
        NormalizedBtcTick {
            venue: venue.to_string(),
            px,
            size: 1.0,
            ts,
        }
    }
}
