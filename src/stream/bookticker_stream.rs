use std::{env, error::Error, time::Duration};

use futures_util::StreamExt;
use serde::Deserialize;

use crate::logic::evaluatelevel::evaluate_level;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

const DEFAULT_WS_BASE: &str = "wss://stream.binance.com:9443";

#[derive(Debug, Deserialize)]
struct BookTickerEvent {
    #[serde(rename = "b")]
    best_bid: String,
}

fn handle_bookticker_message(payload: &str) {
    let event: BookTickerEvent = match serde_json::from_str(payload) {
        Ok(value) => value,
        Err(_) => return,
    };

    let best_bid = match event.best_bid.parse::<f64>() {
        Ok(value) => value,
        Err(_) => return,
    };

    evaluate_level(best_bid);
}


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
                            if let Ok(text) = msg.to_text() {
                                handle_bookticker_message(text);
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
