#[derive(Debug, Clone)]
pub struct AssetState {
    pub symbol: String,
}

impl AssetState {
    pub fn new() -> Self {
        Self {
            symbol: "ACTUSDT".to_string(),
        }
    }
}
