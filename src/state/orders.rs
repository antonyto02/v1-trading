#[derive(Debug, Clone)]
pub struct SpotState {
    pub bid_price: Option<f64>,
    pub ask_price: Option<f64>,
    pub buy_order_ids: Vec<String>,
    pub sell_order_ids: Vec<String>,
    pub filled_buy: f64,
    pub filled_sell: f64,
}

impl SpotState {
    pub fn new() -> Self {
        Self {
            bid_price: None,
            ask_price: None,
            buy_order_ids: Vec::new(),
            sell_order_ids: Vec::new(),
            filled_buy: 0.0,
            filled_sell: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveOrder {
    pub amount_target: f64,
    pub spot: SpotState,
}

impl ActiveOrder {
    pub fn new() -> Self {
        Self {
            amount_target: 1000.0,
            spot: SpotState::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrdersState {
    pub orders: Vec<ActiveOrder>,
}

impl OrdersState {
    pub fn new() -> Self {
        Self {
            orders: (0..4).map(|_| ActiveOrder::new()).collect(),
        }
    }
}
