use crate::state::orderbook::{get_orderbook_state_snapshot, OrderbookLevel};
use crate::state::orders::get_orders_state_snapshot;

pub fn get_best_bid_and_ask() -> (Vec<OrderbookLevel>, Vec<OrderbookLevel>) {
    let orderbook = get_orderbook_state_snapshot();
    let best_bids = orderbook.bids.into_iter().take(4).collect();
    let best_asks = orderbook.asks.into_iter().take(4).collect();
    (best_bids, best_asks)
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
