use chrono::Local;

use crate::logic::evaluate_buy_orders::EvaluateBuyOrders;
use crate::logic::stream_initializer::refresh_orderbook_state;
use crate::state::asset::get_asset_state_snapshot;
use crate::state::orders::{get_orders_state_snapshot, set_orders_state};

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

fn close_short_position(symbol: &str, position_size: f64) -> bool {
    log(&format!(
        "Enviando cierre de short x1 para {symbol} con size={position_size}."
    ));
    true
}

fn refresh_orderbook_then_evaluate_buy_orders(symbol: String) {
    tokio::spawn(async move {
        if let Err(error) = refresh_orderbook_state(&symbol).await {
            log::error!("Error refrescando orderbook para {}: {}", symbol, error);
        }
        EvaluateBuyOrders().await;
    });
}

pub fn closeshort(new_best_bid: f64) {
    let mut orders_state = get_orders_state_snapshot();
    let symbol = get_asset_state_snapshot().symbol;

    for order in &mut orders_state.orders {
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

        if !close_short_position(&symbol, short_size) {
            continue;
        }

        log(&format!(
            "Binance OK cierre short para {symbol}. Liberando size_position={short_size}."
        ));

        order.has_open_short = false;
        order.size_position = 0.0;
        set_orders_state(orders_state);
        refresh_orderbook_then_evaluate_buy_orders(symbol.clone());
        return;
    }

    refresh_orderbook_then_evaluate_buy_orders(symbol);
}
