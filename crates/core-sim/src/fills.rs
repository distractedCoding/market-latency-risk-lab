#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fill {
    pub price: f64,
    pub qty: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FillSummary {
    pub fills: Vec<Fill>,
    pub filled_qty: f64,
    pub avg_price: f64,
    pub remaining_qty: f64,
}

impl Default for FillSummary {
    fn default() -> Self {
        Self {
            fills: Vec::new(),
            filled_qty: 0.0,
            avg_price: 0.0,
            remaining_qty: 0.0,
        }
    }
}
