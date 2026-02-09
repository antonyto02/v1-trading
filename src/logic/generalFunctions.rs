use crate::state::orderbook::{get_orderbook_state_snapshot, OrderbookLevel};
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};
use crate::state::asset::get_asset_state_snapshot;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::error::Error;
use std::time::{SystemTime, UNIX_EPOCH};

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
    let client = reqwest::Client::new();
    let symbol = "ACTUSDT";
    let api_key = std::env::var("BINANCE_API_KEY").unwrap_or_default();
    let api_secret = std::env::var("BINANCE_API_SECRET").unwrap_or_default();
    let tick_size = get_asset_state_snapshot().tick_size;

    let round_to_tick = |price: f64| -> f64 {
        if tick_size == 0.0 {
            return price;
        }
        let steps = (price / tick_size).round();
        steps * tick_size
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
            .post("https://api.binance.com/api/v3/order")
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
                order.spot.ask_price = Some(round_to_tick(best_bid.price + tick_size));
            }
        }
    }

    set_orders_state(orders_state);
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
        if let Some((lowest_index, _)) = updated_best_bids
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                left.price
                    .partial_cmp(&right.price)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        {
            updated_best_bids.remove(lowest_index);
        }

        if !is_in_best_levels {
            Requeue(&order.spot.sell_order_ids);
        }
    }

    (candidates, updated_best_bids)
}

#[allow(non_snake_case)]
pub fn ProcessActiveBuyOrders(
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
            if let Some((lowest_index, _)) = updated_best_bids
                .iter()
                .enumerate()
                .min_by(|(_, left), (_, right)| {
                    left.price
                        .partial_cmp(&right.price)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            {
                updated_best_bids.remove(lowest_index);
            }
        } else {
            CleanOrder(&order.spot.buy_order_ids);
        }
    }

    (candidates, updated_best_bids)
}

#[allow(non_snake_case)]
pub fn Requeue(_sell_order_ids: &[String]) {}

#[allow(non_snake_case)]
pub fn CleanOrder(_buy_order_ids: &[String]) {}
