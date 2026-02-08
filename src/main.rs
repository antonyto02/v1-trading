use std::error::Error;

mod stream;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let asset = state::asset::AssetState::new();

    let bookticker_handle = tokio::spawn(stream::bookticker_stream::spawn_bookticker_stream(
        &asset.symbol,
    ));
    let aggtrade_handle = tokio::spawn(stream::aggtrade_stream::spawn_aggtrade_stream(
        &asset.symbol,
    ));

    let user_stream_result = stream::user_stream::spawn_user_stream().await;
    bookticker_handle.abort();
    aggtrade_handle.abort();
    user_stream_result?;

    Ok(())
}
