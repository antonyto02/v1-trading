use crate::binance::spot_rest_url;
use crate::state::asset::get_asset_state_snapshot;
use crate::state::orderbook::{OrderbookLevel, get_orderbook_state_snapshot};
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};
use chrono::Local;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

pub fn get_best_bid_and_ask() -> (Vec<OrderbookLevel>, Vec<OrderbookLevel>) {
    let orderbook = get_orderbook_state_snapshot();
    let best_bids = orderbook.bids.into_iter().take(4).collect();
    let best_asks = orderbook.asks.into_iter().take(4).collect();
    (best_bids, best_asks)
}

#[allow(non_snake_case)]
pub async fn FillMissingBestBids(
    candidates: &[usize],
    best_bids: &[OrderbookLevel],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut orders_state = get_orders_state_snapshot();
    let orderbook_state = get_orderbook_state_snapshot();
    let client = reqwest::Client::new();
    let symbol = "ACTUSDT";
    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("BINANCE_API_SECRET").unwrap_or_default();

    let next_ask_price = |bid_price: f64| -> Option<f64> {
        orderbook_state
            .bids
            .iter()
            .chain(orderbook_state.asks.iter())
            .filter(|level| level.price > bid_price)
            .map(|level| level.price)
            .min_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
    };

    for (candidate_index, best_bid) in candidates.iter().zip(best_bids.iter()) {
        let order = match orders_state.orders.get_mut(*candidate_index) {
            Some(order) => order,
            None => continue,
        };

        let amount_target = order.amount_target;
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let query = format!(
            "symbol={symbol}&side=BUY&type=LIMIT&timeInForce=GTC&quantity={amount_target}&price={price}&timestamp={timestamp}",
            symbol = symbol,
            amount_target = amount_target,
            price = best_bid.price,
            timestamp = timestamp
        );

        let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())?;
        mac.update(query.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let response = client
            .post(spot_rest_url("v3/order"))
            .header("X-MBX-APIKEY", &api_key)
            .query(&[("signature", signature)])
            .body(query)
            .send()
            .await?;

        if response.status().is_success() {
            let payload: serde_json::Value = response.json().await?;
            if let Some(order_id) = payload.get("orderId").and_then(|value| value.as_i64()) {
                order.spot.buy_order_ids.push(order_id.to_string());
                order.spot.bid_price = Some(best_bid.price);
                order.spot.ask_price = next_ask_price(best_bid.price);
            }
        }
    }

    set_orders_state(orders_state);
    Ok(())
}

async fn cancel_spot_orders(
    symbol: &str,
    order_ids: &[String],
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if order_ids.is_empty() {
        return Ok(());
    }

    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("BINANCE_API_SECRET").unwrap_or_default();

    if api_key.is_empty() || api_secret.is_empty() {
        return Err("missing BINANCE_API_KEY / BINANCE_API_SECRET".into());
    }

    let client = reqwest::Client::new();

    for order_id in order_ids {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        let query = format!(
            "symbol={symbol}&orderId={order_id}&timestamp={timestamp}",
            symbol = symbol,
            order_id = order_id,
            timestamp = timestamp
        );

        let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())?;
        mac.update(query.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let response = client
            .delete(spot_rest_url("v3/order"))
            .header("X-MBX-APIKEY", &api_key)
            .query(&[
                ("symbol", symbol),
                ("orderId", order_id.as_str()),
                ("timestamp", &timestamp.to_string()),
                ("signature", &signature),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!(
                "error canceling order_id={order_id} in Binance. status={status}, body={body}"
            )
            .into());
        }
    }

    Ok(())
}

#[allow(non_snake_case)]
pub fn ProcessFrozenBlocks(
    mut candidates: Vec<usize>,
    best_bids: &[OrderbookLevel],
    best_asks: &[OrderbookLevel],
) -> (Vec<usize>, Vec<OrderbookLevel>) {
    let orders_state = get_orders_state_snapshot();
    let mut updated_best_bids = best_bids.to_vec();

    for (index, order) in orders_state.orders.iter().enumerate() {
        let bid_price = match order.spot.bid_price {
            Some(price) => price,
            None => continue,
        };

        if order.spot.sell_order_ids.is_empty() {
            continue;
        }

        let is_in_best_levels = best_bids
            .iter()
            .chain(best_asks.iter())
            .any(|level| level.price == bid_price);

        candidates.retain(|candidate| *candidate != index);
        if let Some(matching_index) = updated_best_bids
            .iter()
            .position(|level| (level.price - bid_price).abs() < 1e-9)
        {
            updated_best_bids.remove(matching_index);
        }

        if !is_in_best_levels {
            Requeue(&order.spot.sell_order_ids);
        }
    }

    (candidates, updated_best_bids)
}

#[allow(non_snake_case)]
pub async fn ProcessActiveBuyOrders(
    mut candidates: Vec<usize>,
    best_bids: &[OrderbookLevel],
) -> (Vec<usize>, Vec<OrderbookLevel>) {
    let orders_state = get_orders_state_snapshot();
    let mut updated_best_bids = best_bids.to_vec();

    for &index in candidates.clone().iter() {
        let order = match orders_state.orders.get(index) {
            Some(order) => order,
            None => continue,
        };

        let bid_price = match order.spot.bid_price {
            Some(price) => price,
            None => continue,
        };

        let is_in_best_bids = updated_best_bids
            .iter()
            .any(|level| level.price == bid_price);

        if is_in_best_bids {
            candidates.retain(|candidate| *candidate != index);
            if let Some(matching_index) = updated_best_bids
                .iter()
                .position(|level| (level.price - bid_price).abs() < 1e-9)
            {
                updated_best_bids.remove(matching_index);
            }
        } else {
            CleanOrder(index).await;
        }
    }

    (candidates, updated_best_bids)
}

#[allow(non_snake_case)]
pub fn Requeue(_sell_order_ids: &[String]) {}

#[allow(non_snake_case)]
pub async fn CleanOrder(index: usize) {
    let orders_snapshot = get_orders_state_snapshot();
    let order = match orders_snapshot.orders.get(index) {
        Some(order) => order,
        None => {
            log(&format!(
                "CleanOrder: índice inválido {index}, no se encontró orden."
            ));
            return;
        }
    };

    let symbol = get_asset_state_snapshot().symbol;
    let mut order_ids_to_cancel = order.spot.buy_order_ids.clone();
    order_ids_to_cancel.extend(order.spot.sell_order_ids.clone());

    if let Err(error) = cancel_spot_orders(&symbol, &order_ids_to_cancel).await {
        log(&format!(
            "CleanOrder: no se pudo cancelar en Binance index={index}. error={error}"
        ));
        return;
    }

    let mut latest_orders_state = get_orders_state_snapshot();
    if let Some(order) = latest_orders_state.orders.get_mut(index) {
        log(&format!(
            "CleanOrder: órdenes canceladas en Binance, reseteando index={index}."
        ));
        order.spot.bid_price = None;
        order.spot.ask_price = None;
        order.spot.buy_order_ids.clear();
        order.spot.sell_order_ids.clear();
        order.spot.filled_buy = 0.0;
        order.spot.filled_sell = 0.0;
        set_orders_state(latest_orders_state);
    }
}
