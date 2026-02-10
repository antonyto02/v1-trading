use chrono::Local;

use crate::logic::generalFunctions::Requeue;
use crate::logic::stream_initializer::refresh_orderbook_state_for_current_asset;
use crate::state::asset::get_asset_state_snapshot;
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

fn open_short_position(symbol: &str, position_size: f64) -> bool {
    log(&format!(
        "Enviando apertura de short x1 para {symbol} con size={position_size}."
    ));
    true
}

pub fn openshort(new_best_bid: f64) {
    let mut orders_state = get_orders_state_snapshot();
    let symbol = get_asset_state_snapshot().symbol;

    for order in &mut orders_state.orders {
        let order_bid_price = match order.spot.bid_price {
            Some(value) => value,
            None => continue,
        };

        if order.has_open_short || new_best_bid >= order_bid_price {
            continue;
        }

        let short_size = order.spot.filled_buy - order.spot.filled_sell;
        if short_size <= 0.0 {
            continue;
        }

        if !open_short_position(&symbol, short_size) {
            continue;
        }

        log(&format!(
            "Binance OK apertura short para {symbol}. Guardando size_position={short_size}."
        ));

        let sell_order_ids = order.spot.sell_order_ids.clone();
        order.has_open_short = true;
        order.size_position = short_size;
        set_orders_state(orders_state);
        refresh_orderbook_state_for_current_asset();
        Requeue(&sell_order_ids);
        return;
    }

    refresh_orderbook_state_for_current_asset();
    Requeue(&[]);
}
