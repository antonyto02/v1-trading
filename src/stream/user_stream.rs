use std::{env, error::Error, time::Duration};

use futures_util::StreamExt;
use tokio::time::{interval, sleep};
use tokio_tungstenite::connect_async;

use crate::binance::{spot_rest_base, spot_ws_base, ws_stream_url};
use crate::logic::user_stream_handler::handle_user_stream_message;

pub async fn spawn_user_stream() -> Result<(), Box<dyn Error + Send + Sync>> {
    let api_key = env::var("BINANCE_API_KEY")?;
    let _api_secret = env::var("BINANCE_API_SECRET")?;
    let rest_base = spot_rest_base();
    let ws_base = spot_ws_base();

    let client = reqwest::Client::new();

    loop {
        let listen_key = match create_listen_key(&client, &rest_base, &api_key).await {
            Ok(key) => key,
            Err(err) => {
                let _ = err;
                sleep(Duration::from_secs(5)).await;
                continue;
            }
        };

        let ws_url = ws_stream_url(&ws_base, &listen_key);

        match connect_async(&ws_url).await {
            Ok((mut ws_stream, _)) => {
                let keepalive_handle = tokio::spawn(keepalive_listen_key(
                    client.clone(),
                    rest_base.clone(),
                    api_key.clone(),
                    listen_key.clone(),
                ));
                while let Some(message) = ws_stream.next().await {
                    match message {
                        Ok(msg) => {
                            if msg.is_text() || msg.is_binary() {
                                if let Ok(payload) = msg.to_text() {
                                    let payload = payload.to_string();
                                    tokio::spawn(async move {
                                        handle_user_stream_message(payload).await;
                                    });
                                } else if msg.is_binary() {
                                    if let Ok(payload) = String::from_utf8(msg.into_data()) {
                                        tokio::spawn(async move {
                                            handle_user_stream_message(payload).await;
                                        });
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            let _ = err;
                            break;
                        }
                    }
                }
                keepalive_handle.abort();
            }
            Err(err) => {
                let _ = err;
            }
        }

        sleep(Duration::from_secs(5)).await;
    }
}

async fn create_listen_key(
    client: &reqwest::Client,
    rest_base: &str,
    api_key: &str,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    let url = format!("{}/v3/userDataStream", rest_base.trim_end_matches('/'));
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

async fn keepalive_listen_key(
    client: reqwest::Client,
    rest_base: String,
    api_key: String,
    listen_key: String,
) {
    let mut ticker = interval(Duration::from_secs(30 * 60));
    loop {
        ticker.tick().await;
        if let Err(err) = renew_listen_key(&client, &rest_base, &api_key, &listen_key).await {
            let _ = err;
        }
    }
}

async fn renew_listen_key(
    client: &reqwest::Client,
    rest_base: &str,
    api_key: &str,
    listen_key: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let url = format!(
        "{}/v3/userDataStream?listenKey={}",
        rest_base.trim_end_matches('/'),
        listen_key
    );
    let response = client
        .put(url)
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("listenKey keepalive failed: {status} {body}").into());
    }

    Ok(())
}
