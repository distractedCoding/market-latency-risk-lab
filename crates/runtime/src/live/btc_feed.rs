#[derive(Debug, Clone, PartialEq)]
pub struct NormalizedBtcTick {
    pub venue: String,
    pub px: f64,
    pub size: f64,
    pub ts: String,
}
