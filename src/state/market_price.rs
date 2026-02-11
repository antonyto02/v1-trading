use std::sync::{Mutex, OnceLock};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct MarketPriceState {
    pub last_best_bid: Option<f64>,
}

impl MarketPriceState {
    pub fn new() -> Self {
        Self {
            last_best_bid: None,
        }
    }
}

static MARKET_PRICE_STATE: OnceLock<Mutex<MarketPriceState>> = OnceLock::new();

pub fn set_market_price_state(state: MarketPriceState) {
    let lock = MARKET_PRICE_STATE.get_or_init(|| Mutex::new(MarketPriceState::new()));
    let mut guard = lock.lock().expect("market price state lock poisoned");
    *guard = state;
}

pub fn get_market_price_state_snapshot() -> MarketPriceState {
    let lock = MARKET_PRICE_STATE.get_or_init(|| Mutex::new(MarketPriceState::new()));
    lock.lock()
        .expect("market price state lock poisoned")
        .clone()
}
