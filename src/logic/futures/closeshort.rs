use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Local;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::logic::evaluate_buy_orders::EvaluateBuyOrders;
use crate::logic::stream_initializer::refresh_orderbook_state;
use crate::state::asset::get_asset_state_snapshot;
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};

const BINANCE_FUTURES_BASE: &str = "https://fapi.binance.com";

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

fn format_qty(value: f64) -> String {
    let mut qty = format!("{value:.8}");
    while qty.contains('.') && qty.ends_with('0') {
        qty.pop();
    }
    if qty.ends_with('.') {
        qty.pop();
    }
    qty
}

async fn set_leverage_x1(
    symbol: &str,
    client: &reqwest::Client,
    api_key: &str,
    api_secret: &str,
) -> bool {
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_millis(),
        Err(err) => {
            log(&format!(
                "No se pudo obtener timestamp para leverage: {err}"
            ));
            return false;
        }
    };

    let query = format!("symbol={symbol}&leverage=1&timestamp={timestamp}");
    let mut mac = match Hmac::<Sha256>::new_from_slice(api_secret.as_bytes()) {
        Ok(value) => value,
        Err(err) => {
            log(&format!("Error creando firma HMAC para leverage: {err}"));
            return false;
        }
    };
    mac.update(query.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    let signed_query = format!("{query}&signature={signature}");

    let response = match client
        .post(format!(
            "{BINANCE_FUTURES_BASE}/fapi/v1/leverage?{signed_query}"
        ))
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await
    {
        Ok(value) => value,
        Err(err) => {
            log(&format!("Error HTTP seteando leverage x1: {err}"));
            return false;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log(&format!(
            "Binance FUTURES rechazó leverage x1. status={status}, body={body}"
        ));
        return false;
    }

    log(&format!("Leverage x1 configurado para {symbol}."));
    true
}

async fn close_short_position(symbol: &str, position_size: f64) -> bool {
    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("BINANCE_API_SECRET").unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        log("No hay BINANCE_API_KEY / BINANCE_API_SECRET para cerrar short.");
        return false;
    }

    let client = reqwest::Client::new();
    if !set_leverage_x1(symbol, &client, &api_key, &api_secret).await {
        return false;
    }

    let qty = format_qty(position_size);
    let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_millis(),
        Err(err) => {
            log(&format!(
                "No se pudo obtener timestamp para cierre short: {err}"
            ));
            return false;
        }
    };

    let query = format!(
        "symbol={symbol}&side=BUY&type=MARKET&reduceOnly=true&quantity={qty}&timestamp={timestamp}"
    );

    let mut mac = match Hmac::<Sha256>::new_from_slice(api_secret.as_bytes()) {
        Ok(value) => value,
        Err(err) => {
            log(&format!(
                "Error creando firma HMAC para cierre short: {err}"
            ));
            return false;
        }
    };
    mac.update(query.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());
    let signed_query = format!("{query}&signature={signature}");

    log(&format!(
        "Cerrando short MARKET x1 para {symbol}, qty={qty} (taker)."
    ));

    let response = match client
        .post(format!(
            "{BINANCE_FUTURES_BASE}/fapi/v1/order?{signed_query}"
        ))
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await
    {
        Ok(value) => value,
        Err(err) => {
            log(&format!("Error HTTP cerrando short: {err}"));
            return false;
        }
    };

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log(&format!(
            "Fallo cierre short en Binance FUTURES. status={status}, body={body}"
        ));
        return false;
    }

    let payload: serde_json::Value = response.json().await.unwrap_or_default();
    log(&format!("Cierre short OK en Binance: {payload}"));
    true
}

pub fn closeshort(new_best_bid: f64) {
    let orders_state = get_orders_state_snapshot();
    let symbol = get_asset_state_snapshot().symbol;

    let mut selected: Option<(usize, f64)> = None;
    for (index, order) in orders_state.orders.iter().enumerate() {
        let order_bid_price = match order.spot.bid_price {
            Some(value) => value,
            None => continue,
        };

        if !order.has_open_short || (new_best_bid - order_bid_price).abs() >= f64::EPSILON {
            continue;
        }

        let short_size = order.size_position;
        if short_size <= 0.0 {
            continue;
        }

        selected = Some((index, short_size));
        break;
    }

    tokio::spawn(async move {
        if let Some((order_index, short_size)) = selected {
            if close_short_position(&symbol, short_size).await {
                let mut latest_orders_state = get_orders_state_snapshot();
                if let Some(order) = latest_orders_state.orders.get_mut(order_index) {
                    log(&format!(
                        "Binance OK cierre short para {symbol}. Liberando size_position={short_size}."
                    ));
                    order.has_open_short = false;
                    order.size_position = 0.0;
                    set_orders_state(latest_orders_state);
                }
            }
        }

        if let Err(error) = refresh_orderbook_state(&symbol).await {
            log(&format!(
                "Error refrescando orderbook tras closeshort: {error}"
            ));
        }
        EvaluateBuyOrders().await;
    });
}
