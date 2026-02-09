use std::error::Error;

mod stream;
mod logic;
mod state;
mod http;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let asset = state::asset::AssetState::new();
    state::asset::set_asset_state(asset.clone());
    let symbol = asset.symbol.clone();
    let bookticker_symbol = symbol.clone();
    let aggtrade_symbol = symbol.clone();

    let server_handle = tokio::spawn(async {
        if let Err(error) = http::run_server().await {
            log::error!("HTTP server error: {}", error);
        }
    });

    logic::stream_initializer::start_streams(bookticker_symbol, aggtrade_symbol).await?;
    server_handle.abort();

    Ok(())
}
