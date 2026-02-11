use std::{error::Error, time::Duration};

use futures_util::StreamExt;
use serde::Deserialize;

use crate::binance::{spot_ws_base, ws_stream_url};
use crate::logic::evaluatelevel::evaluate_level;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

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

pub async fn spawn_bookticker_stream(symbol: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
    let ws_base = spot_ws_base();
    let stream_name = format!("{}@bookTicker", symbol.to_lowercase());
    let ws_url = ws_stream_url(&ws_base, &stream_name);

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
