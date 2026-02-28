use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tokio::sync::broadcast;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FeedMode {
    PaperLive,
    Sim,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct SourceCount {
    pub source: String,
    pub count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct FeedHealthResponse {
    pub mode: FeedMode,
    pub source_counts: Vec<SourceCount>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct DiscoveredMarket {
    pub source: String,
    pub market_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct DiscoveredMarketsResponse {
    pub markets: Vec<DiscoveredMarket>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartRunError {
    RunIdOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaperOrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Debug, serde::Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum RuntimeEvent {
    Connected {
        run_id: Option<u64>,
    },
    RunStarted {
        run_id: u64,
    },
    PaperIntent {
        market_id: String,
        side: PaperOrderSide,
        qty: f64,
        limit_px: f64,
    },
    PaperFill {
        market_id: String,
        side: PaperOrderSide,
        qty: f64,
        fill_px: f64,
    },
    RiskReject {
        market_id: String,
        reason: String,
        requested_qty: f64,
    },
}

impl RuntimeEvent {
    pub fn connected() -> Self {
        Self::Connected { run_id: None }
    }

    pub fn run_started(run_id: u64) -> Self {
        Self::RunStarted { run_id }
    }

    pub fn paper_intent(market_id: impl Into<String>, side: PaperOrderSide, qty: f64, limit_px: f64) -> Self {
        Self::PaperIntent {
            market_id: market_id.into(),
            side,
            qty,
            limit_px,
        }
    }

    pub fn paper_fill(market_id: impl Into<String>, side: PaperOrderSide, qty: f64, fill_px: f64) -> Self {
        Self::PaperFill {
            market_id: market_id.into(),
            side,
            qty,
            fill_px,
        }
    }

    pub fn risk_reject(market_id: impl Into<String>, reason: impl Into<String>, requested_qty: f64) -> Self {
        Self::RiskReject {
            market_id: market_id.into(),
            reason: reason.into(),
            requested_qty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    next_run_id: Arc<AtomicU64>,
    events_tx: broadcast::Sender<RuntimeEvent>,
    feed_mode: FeedMode,
}

impl Default for AppState {
    fn default() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
            feed_mode: FeedMode::PaperLive,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(&self) -> Result<u64, StartRunError> {
        let previous = self
            .next_run_id
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                current.checked_add(1)
            })
            .map_err(|_| StartRunError::RunIdOverflow)?;

        Ok(previous + 1)
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<RuntimeEvent> {
        self.events_tx.subscribe()
    }

    pub fn publish_event(
        &self,
        event: RuntimeEvent,
    ) -> Result<usize, broadcast::error::SendError<RuntimeEvent>> {
        self.events_tx.send(event)
    }

    pub fn feed_health(&self) -> FeedHealthResponse {
        FeedHealthResponse {
            mode: self.feed_mode,
            source_counts: Vec::new(),
        }
    }

    pub fn discovered_markets(&self) -> DiscoveredMarketsResponse {
        DiscoveredMarketsResponse {
            markets: Vec::new(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_next_run_id_for_test(next_run_id: u64) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(next_run_id)),
            events_tx,
            feed_mode: FeedMode::PaperLive,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_feed_mode_for_test(feed_mode: FeedMode) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
            feed_mode,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::{AppState, FeedMode};

    #[test]
    fn start_run_returns_overflow_error_at_u64_max() {
        let state = AppState::new();
        state.next_run_id.store(u64::MAX, Ordering::Relaxed);

        assert!(state.start_run().is_err());
    }

    #[test]
    fn feed_health_returns_configured_mode() {
        let state = AppState::with_feed_mode_for_test(FeedMode::Sim);

        assert_eq!(state.feed_health().mode, FeedMode::Sim);
    }
}
