#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLogEvent {
    pub tick: u64,
    pub event: String,
    pub decision_latency_micros: Option<u64>,
}

impl RunLogEvent {
    pub fn new(tick: u64, event: impl Into<String>, decision_latency_micros: Option<u64>) -> Self {
        Self {
            tick,
            event: event.into(),
            decision_latency_micros,
        }
    }
}

pub trait RunLogWriter {
    fn write(&mut self, event: RunLogEvent);
}

#[derive(Debug, Default)]
pub struct InMemoryRunLogWriter {
    events: Vec<RunLogEvent>,
}

impl InMemoryRunLogWriter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> &[RunLogEvent] {
        &self.events
    }
}

impl RunLogWriter for InMemoryRunLogWriter {
    fn write(&mut self, event: RunLogEvent) {
        self.events.push(event);
    }
}
