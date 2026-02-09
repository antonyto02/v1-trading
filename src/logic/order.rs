use crate::state::asset::get_asset_state_snapshot;
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn add_filled_buy_for_bid_price(price: f64, quantity: f64) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            break;
        }
    }

    let order_index = match updated {
        Some(index) => index,
        None => return Ok(()),
    };

    let ask_price = match orders_state.orders[order_index].spot.ask_price {
        Some(value) => value,
        None => {
            set_orders_state(orders_state);
            return Ok(());
        }
    };

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
        }
    }

    set_orders_state(orders_state);
    Ok(())
}

pub fn add_filled_sell_for_ask_price(price: f64, quantity: f64) -> Option<usize> {
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
            break;
        }
    }

    if matched_index.is_some() {
        set_orders_state(orders_state);
    }

    matched_index
}
