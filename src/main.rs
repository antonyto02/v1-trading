use std::error::Error;

mod stream;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let asset = state::asset::AssetState::new();
    let symbol = asset.symbol.clone();
    let bookticker_symbol = symbol.clone();
    let aggtrade_symbol = symbol.clone();

    let bookticker_handle = tokio::spawn(async move {
        stream::bookticker_stream::spawn_bookticker_stream(&bookticker_symbol).await
    });
    let aggtrade_handle = tokio::spawn(async move {
        stream::aggtrade_stream::spawn_aggtrade_stream(&aggtrade_symbol).await
    });

    let user_stream_result = stream::user_stream::spawn_user_stream().await;
    bookticker_handle.abort();
    aggtrade_handle.abort();
    user_stream_result?;

    Ok(())
}
