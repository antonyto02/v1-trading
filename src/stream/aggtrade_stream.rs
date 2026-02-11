use std::{error::Error, time::Duration};

use crate::binance::{spot_ws_base, ws_stream_url};
use futures_util::StreamExt;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

pub async fn spawn_aggtrade_stream(symbol: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_base = spot_ws_base();
    let stream_name = format!("{}@aggTrade", symbol.to_lowercase());
    let ws_url = ws_stream_url(&ws_base, &stream_name);

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
