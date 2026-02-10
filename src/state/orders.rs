use std::sync::{Mutex, OnceLock};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
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

#[derive(Debug, Clone, Serialize)]
pub struct ActiveOrder {
    pub amount_target: f64,
    pub has_open_short: bool,
    pub size_position: f64,
    pub spot: SpotState,
}

impl ActiveOrder {
    pub fn new() -> Self {
        Self {
            amount_target: 500.0,
            has_open_short: false,
            size_position: 0.0,
            spot: SpotState::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
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

static ORDERS_STATE: OnceLock<Mutex<OrdersState>> = OnceLock::new();

pub fn set_orders_state(state: OrdersState) {
    let lock = ORDERS_STATE.get_or_init(|| Mutex::new(OrdersState::new()));
    let mut guard = lock.lock().expect("orders state lock poisoned");
    *guard = state;
}

pub fn get_orders_state_snapshot() -> OrdersState {
    let lock = ORDERS_STATE.get_or_init(|| Mutex::new(OrdersState::new()));
    lock.lock().expect("orders state lock poisoned").clone()
}
