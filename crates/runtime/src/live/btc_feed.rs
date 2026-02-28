use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NormalizedBtcTick {
    pub venue: String,
    pub px: f64,
    pub size: f64,
    pub ts: u64,
}
