use crate::state::asset::get_asset_state_snapshot;
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};
use chrono::Local;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

pub async fn add_filled_buy_for_bid_price(price: f64, quantity: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
    log(&format!(
        "Execution report BUY: buscando orden con bid_price={price} para sumar filled_buy += {quantity}."
    ));
    let mut orders_state = get_orders_state_snapshot();
    let mut updated = None;

    for (index, order) in orders_state.orders.iter_mut().enumerate() {
        let bid_price = match order.spot.bid_price {
            Some(value) => value,
            None => continue,
        };

        if (bid_price - price).abs() < 1e-9 {
            order.spot.filled_buy += quantity;
            updated = Some(index);
            log(&format!(
                "BUY match encontrado en index={index}. filled_buy actualizado, preparando orden SELL."
            ));
            break;
        }
    }

    let order_index = match updated {
        Some(index) => index,
        None => {
            log("BUY: no se encontró orden con bid_price igual al precio del trade.");
            return Ok(());
        }
    };

    let ask_price = match orders_state.orders[order_index].spot.ask_price {
        Some(value) => value,
        None => {
            log(&format!(
                "BUY: order index={order_index} no tiene ask_price, no se crea orden SELL."
            ));
            set_orders_state(orders_state);
            return Ok(());
        }
    };

    log(&format!(
        "BUY: creando orden SELL por quantity={quantity} a price={ask_price}."
    ));
    let client = reqwest::Client::new();
    let symbol = get_asset_state_snapshot().symbol;
    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("BINANCE_API_SECRET").unwrap_or_default();
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let query = format!(
        "symbol={symbol}&side=SELL&type=LIMIT&timeInForce=GTC&quantity={quantity}&price={price}&timestamp={timestamp}",
        symbol = symbol,
        quantity = quantity,
        price = ask_price,
        timestamp = timestamp
    );

    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())?;
    mac.update(query.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    let response = client
        .post("https://api.binance.com/api/v3/order")
        .header("X-MBX-APIKEY", &api_key)
        .query(&[("signature", signature)])
        .body(query)
        .send()
        .await?;

    if response.status().is_success() {
        let payload: serde_json::Value = response.json().await?;
        if let Some(order_id) = payload.get("orderId").and_then(|value| value.as_i64()) {
            orders_state.orders[order_index]
                .spot
                .sell_order_ids
                .push(order_id.to_string());
            log(&format!(
                "BUY: orden SELL creada. order_id={order_id} agregado a sell_order_ids."
            ));
        } else {
            log("BUY: Binance respondió OK pero sin orderId; no se agregó sell_order_id.");
        }
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        log(&format!(
            "BUY: fallo al crear orden SELL. status={status} body={body}"
        ));
    }

    set_orders_state(orders_state);
    log("BUY: estado de órdenes persistido.");
    Ok(())
}

pub fn add_filled_sell_for_ask_price(price: f64, quantity: f64) -> Option<usize> {
    log(&format!(
        "Execution report SELL: buscando orden con ask_price={price} para sumar filled_sell += {quantity}."
    ));
    let mut orders_state = get_orders_state_snapshot();
    let mut matched_index = None;

    for (index, order) in orders_state.orders.iter_mut().enumerate() {
        let ask_price = match order.spot.ask_price {
            Some(value) => value,
            None => continue,
        };

        if (ask_price - price).abs() < 1e-9 {
            order.spot.filled_sell += quantity;
            matched_index = Some(index);
            log(&format!(
                "SELL match encontrado en index={index}. filled_sell actualizado."
            ));
            break;
        }
    }

    if matched_index.is_some() {
        set_orders_state(orders_state);
        log("SELL: estado de órdenes persistido.");
    } else {
        log("SELL: no se encontró orden con ask_price igual al precio del trade.");
    }

    matched_index
}
