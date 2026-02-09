use std::sync::{Mutex, OnceLock};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct AssetState {
    pub symbol: String,
    pub tick_size: f64,
}

impl AssetState {
    pub fn new() -> Self {
        Self {
            symbol: "ACTUSDT".to_string(),
            tick_size: 0.0001,
        }
    }
}

static ASSET_STATE: OnceLock<Mutex<AssetState>> = OnceLock::new();

pub fn set_asset_state(state: AssetState) {
    let lock = ASSET_STATE.get_or_init(|| Mutex::new(AssetState::new()));
    let mut guard = lock.lock().expect("asset state lock poisoned");
    *guard = state;
}

pub fn get_asset_state_snapshot() -> AssetState {
    let lock = ASSET_STATE.get_or_init(|| Mutex::new(AssetState::new()));
    lock.lock()
        .expect("asset state lock poisoned")
        .clone()
}
