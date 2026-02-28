use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BtcMedianTick {
    pub px_median: f64,
    pub px_spread: f64,
    pub venue_count: u32,
    pub ts: u64,
}

impl BtcMedianTick {
    pub fn new(px_median: f64, px_spread: f64, venue_count: u32, ts: u64) -> Self {
        Self {
            px_median,
            px_spread,
            venue_count,
            ts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum LiveIngestEvent {
    #[serde(rename = "btc_median_tick")]
    BtcMedianTick(BtcMedianTick),
}

#[cfg(test)]
mod tests {
    use super::{BtcMedianTick, LiveIngestEvent};
    use serde_json::json;

    #[test]
    fn btc_median_tick_serializes_with_exact_payload_shape() {
        let tick = BtcMedianTick::new(64_000.0, 12.5, 3, 1_735_689_600_000);
        let json = serde_json::to_value(tick).unwrap();

        assert_eq!(
            json,
            json!({
                "px_median": 64_000.0,
                "px_spread": 12.5,
                "venue_count": 3,
                "ts": 1_735_689_600_000_u64,
            })
        );
    }

    #[test]
    fn live_ingest_event_serializes_as_tagged_payload() {
        let event = LiveIngestEvent::BtcMedianTick(BtcMedianTick::new(
            64_000.0,
            12.5,
            3,
            1_735_689_600_000,
        ));
        let json = serde_json::to_value(event).unwrap();

        assert_eq!(
            json,
            json!({
                "event": "btc_median_tick",
                "payload": {
                    "px_median": 64_000.0,
                    "px_spread": 12.5,
                    "venue_count": 3,
                    "ts": 1_735_689_600_000_u64,
                }
            })
        );
    }

    #[test]
    fn btc_median_tick_deserializes_and_round_trips() {
        let json = json!({
            "px_median": 64_000.0,
            "px_spread": 12.5,
            "venue_count": 3,
            "ts": 1_735_689_600_000_u64,
        });

        let tick: BtcMedianTick = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(serde_json::to_value(tick).unwrap(), json);
    }

    #[test]
    fn live_ingest_event_deserializes_and_round_trips() {
        let json = json!({
            "event": "btc_median_tick",
            "payload": {
                "px_median": 64_000.0,
                "px_spread": 12.5,
                "venue_count": 3,
                "ts": 1_735_689_600_000_u64,
            }
        });

        let event: LiveIngestEvent = serde_json::from_value(json.clone()).unwrap();

        assert_eq!(serde_json::to_value(event).unwrap(), json);
    }
}
