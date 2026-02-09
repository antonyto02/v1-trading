use std::{env, error::Error, time::Duration};

use futures_util::StreamExt;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

const DEFAULT_WS_BASE: &str = "wss://stream.binance.com:9443";

pub async fn spawn_bookticker_stream(
    symbol: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_base = env::var("BINANCE_WS_BASE_URL").unwrap_or_else(|_| DEFAULT_WS_BASE.to_string());
    let stream_name = format!("{}@bookTicker", symbol.to_lowercase());
    let ws_url = format!("{}/ws/{}", ws_base.trim_end_matches('/'), stream_name);

    loop {
        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
                while let Some(message) = ws_stream.next().await {
                    match message {
                        Ok(msg) => {
                            if msg.is_text() || msg.is_binary() {
                                let _ = msg;
                            }
                        }
                        Err(err) => {
                            let _ = err;
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                let _ = err;
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}
