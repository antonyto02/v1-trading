use serde::Deserialize;

use crate::logic::order::add_filled_buy_for_bid_price;
use crate::state::asset::get_asset_state_snapshot;

#[derive(Debug, Deserialize)]
struct ExecutionReportEvent {
    #[serde(rename = "e")]
    event_type: String,
    #[serde(rename = "s")]
    symbol: String,
    #[serde(rename = "S")]
    side: String,
    #[serde(rename = "L")]
    last_executed_price: Option<String>,
    #[serde(rename = "l")]
    last_executed_quantity: Option<String>,
}

pub async fn handle_user_stream_message(message: String) {
    let event: ExecutionReportEvent = match serde_json::from_str(&message) {
        Ok(payload) => payload,
        Err(_) => return,
    };

    if event.event_type != "executionReport" {
        return;
    }

    let asset_symbol = get_asset_state_snapshot().symbol;
    if event.symbol != asset_symbol {
        return;
    }

    if event.side != "BUY" {
        return;
    }

    let price = match event
        .last_executed_price
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
    {
        Some(value) => value,
        None => return,
    };

    let quantity = match event
        .last_executed_quantity
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
    {
        Some(value) => value,
        None => return,
    };

    if quantity <= 0.0 {
        return;
    }

    if let Err(err) = add_filled_buy_for_bid_price(price, quantity).await {
        let _ = err;
    }
}
