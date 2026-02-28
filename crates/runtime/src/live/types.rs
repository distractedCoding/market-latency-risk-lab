use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BtcMedianTick {
    pub px_median: f64,
    pub spread_bps: f64,
    pub venue_count: u32,
}

impl BtcMedianTick {
    pub fn new(px_median: f64, spread_bps: f64, venue_count: u32) -> Self {
        Self {
            px_median,
            spread_bps,
            venue_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload", rename_all = "snake_case")]
pub enum LiveIngestEvent {
    BtcMedianTick(BtcMedianTick),
}

#[cfg(test)]
mod tests {
    use super::{BtcMedianTick, LiveIngestEvent};

    #[test]
    fn btc_median_tick_serializes_with_required_fields() {
        let tick = BtcMedianTick::new(64_000.0, 12.5, 3);
        let json = serde_json::to_value(tick).unwrap();
        assert!(json.get("px_median").is_some());
        assert!(json.get("venue_count").is_some());
    }

    #[test]
    fn live_ingest_event_serializes_as_tagged_payload() {
        let event = LiveIngestEvent::BtcMedianTick(BtcMedianTick::new(64_000.0, 12.5, 3));
        let json = serde_json::to_value(event).unwrap();

        assert_eq!(
            json.get("event").and_then(|value| value.as_str()),
            Some("btc_median_tick")
        );
        assert!(json.get("payload").is_some());
    }
}
