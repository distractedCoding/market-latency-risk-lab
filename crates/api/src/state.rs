use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Clone, Debug, Default)]
pub struct AppState {
    next_run_id: Arc<AtomicU64>,
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_run(&self) -> u64 {
        self.next_run_id.fetch_add(1, Ordering::Relaxed) + 1
    }
}
