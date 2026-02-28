#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunLogEventKind {
    TickStarted,
    MarketDataApplied,
    SignalsGenerated,
    OrdersSimulated,
    PortfolioUpdated,
    DecisionLatencyRecorded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLogEvent {
    pub tick: u64,
    pub kind: RunLogEventKind,
    pub decision_latency_micros: Option<u64>,
}

impl RunLogEvent {
    pub fn new(tick: u64, kind: RunLogEventKind, decision_latency_micros: Option<u64>) -> Self {
        Self {
            tick,
            kind,
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
