#[derive(Debug, Clone)]
pub struct OrderbookLevel {
    pub price: f64,
    pub qty: f64,
}

impl OrderbookLevel {
    pub fn new(price: f64, qty: f64) -> Self {
        Self { price, qty }
    }
}

#[derive(Debug, Clone)]
pub struct OrderbookState {
    pub bids: Vec<OrderbookLevel>,
    pub asks: Vec<OrderbookLevel>,
}

impl OrderbookState {
    pub fn new() -> Self {
        Self {
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }
}
