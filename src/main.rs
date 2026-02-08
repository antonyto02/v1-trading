use std::error::Error;

mod stream;
mod logic;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let asset = state::asset::AssetState::new();
    let symbol = asset.symbol.clone();
    let bookticker_symbol = symbol.clone();
    let aggtrade_symbol = symbol.clone();

    logic::stream_initializer::start_streams(bookticker_symbol, aggtrade_symbol).await?;

    Ok(())
}
