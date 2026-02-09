use std::sync::{Mutex, OnceLock};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OrderbookLevel {
    pub price: f64,
    pub qty: f64,
}

impl OrderbookLevel {
    pub fn new(price: f64, qty: f64) -> Self {
        Self { price, qty }
    }
}

#[derive(Debug, Clone, Serialize)]
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

static ORDERBOOK_STATE: OnceLock<Mutex<OrderbookState>> = OnceLock::new();

pub fn set_orderbook_state(state: OrderbookState) {
    let lock = ORDERBOOK_STATE.get_or_init(|| Mutex::new(OrderbookState::new()));
    let mut guard = lock.lock().expect("orderbook state lock poisoned");
    *guard = state;
}

pub fn get_orderbook_state_snapshot() -> OrderbookState {
    let lock = ORDERBOOK_STATE.get_or_init(|| Mutex::new(OrderbookState::new()));
    lock.lock()
        .expect("orderbook state lock poisoned")
        .clone()
}
