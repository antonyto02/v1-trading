use std::error::Error;

use crate::stream;

pub async fn start_streams(
    bookticker_symbol: String,
    aggtrade_symbol: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
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
