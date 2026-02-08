use std::{env, error::Error, time::Duration};

use futures_util::StreamExt;
use tokio::time::sleep;
use tokio_tungstenite::connect_async;

const DEFAULT_REST_BASE: &str = "https://api.binance.com";
const DEFAULT_WS_BASE: &str = "wss://stream.binance.com:9443";

pub async fn spawn_user_stream() -> Result<(), Box<dyn Error>> {
    let api_key = env::var("BINANCE_API_KEY")?;
    let _api_secret = env::var("BINANCE_API_SECRET")?;
    let rest_base = env::var("BINANCE_REST_BASE_URL").unwrap_or_else(|_| DEFAULT_REST_BASE.to_string());
    let ws_base = env::var("BINANCE_WS_BASE_URL").unwrap_or_else(|_| DEFAULT_WS_BASE.to_string());

    let client = reqwest::Client::new();

    loop {
        let listen_key = match create_listen_key(&client, &rest_base, &api_key).await {
            Ok(key) => key,
            Err(err) => {
                eprintln!("Failed to create listenKey: {err}");
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let ws_url = format!(
            "{}/ws/{}",
            ws_base.trim_end_matches('/'),
            listen_key
        );

        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
                println!("Connected to user stream: {ws_url}");
                while let Some(message) = ws_stream.next().await {
                    match message {
                        Ok(msg) => {
                            if msg.is_text() || msg.is_binary() {
                                println!("{}", msg);
                            }
                        }
                        Err(err) => {
                            eprintln!("WebSocket error: {err}");
                            break;
                        }
                    }
                }
                eprintln!("WebSocket disconnected. Reconnecting...");
            }
            Err(err) => {
                eprintln!("Failed to connect WebSocket: {err}");
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn create_listen_key(
    client: &reqwest::Client,
    rest_base: &str,
    api_key: &str,
) -> Result<String, Box<dyn Error>> {
    let url = format!("{}/api/v3/userDataStream", rest_base.trim_end_matches('/'));
    let response = client
        .post(url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("listenKey request failed: {status} {body}").into());
    }

    let payload: serde_json::Value = response.json().await?;
    let listen_key = payload
        .get("listenKey")
        .and_then(|value| value.as_str())
        .ok_or("listenKey missing in response")?;

    Ok(listen_key.to_string())
}
