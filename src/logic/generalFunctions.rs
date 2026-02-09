use crate::state::orderbook::{get_orderbook_state_snapshot, OrderbookLevel};

pub fn get_best_bid_and_ask() -> (Vec<OrderbookLevel>, Vec<OrderbookLevel>) {
    let orderbook = get_orderbook_state_snapshot();
    let best_bids = orderbook.bids.into_iter().take(4).collect();
    let best_asks = orderbook.asks.into_iter().take(4).collect();
    (best_bids, best_asks)
}
