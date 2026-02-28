use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use tokio::sync::broadcast;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StartRunError {
    RunIdOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeEventType {
    Connected,
    RunStarted,
}

#[derive(Clone, Debug, serde::Serialize)]
pub struct RuntimeEvent {
    pub event_type: RuntimeEventType,
    pub run_id: Option<u64>,
}

impl RuntimeEvent {
    pub fn connected() -> Self {
        Self {
            event_type: RuntimeEventType::Connected,
            run_id: None,
        }
    }

    pub fn run_started(run_id: u64) -> Self {
        Self {
            event_type: RuntimeEventType::RunStarted,
            run_id: Some(run_id),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppState {
    next_run_id: Arc<AtomicU64>,
    events_tx: broadcast::Sender<RuntimeEvent>,
}

impl Default for AppState {
    fn default() -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(0)),
            events_tx,
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

    #[cfg(test)]
    pub(crate) fn with_next_run_id_for_test(next_run_id: u64) -> Self {
        let (events_tx, _) = broadcast::channel(256);
        Self {
            next_run_id: Arc::new(AtomicU64::new(next_run_id)),
            events_tx,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::AppState;

    #[test]
    fn start_run_returns_overflow_error_at_u64_max() {
        let state = AppState::new();
        state.next_run_id.store(u64::MAX, Ordering::Relaxed);

        assert!(state.start_run().is_err());
    }
}
