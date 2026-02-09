use std::error::Error;

use crate::logic::evaluate_buy_orders::EvaluateBuyOrders;
use crate::state::orderbook::{set_orderbook_state, OrderbookLevel, OrderbookState};
use crate::state::orders::{set_orders_state, OrdersState};
use crate::stream;
use serde_json::Value;

const BINANCE_REST_BASE: &str = "https://api.binance.com";

pub async fn start_streams(
    bookticker_symbol: String,
    aggtrade_symbol: String,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let orderbook_state = initialize_orderbook(&bookticker_symbol).await?;
    println!("Orderbook state initialized: {:?}", orderbook_state);
    set_orderbook_state(orderbook_state);
    let orders_state = OrdersState::new();
    println!("Orders state initialized: {:?}", orders_state);
    set_orders_state(orders_state);
    EvaluateBuyOrders().await;

    let bookticker_handle = tokio::spawn(async move {
        stream::bookticker_stream::spawn_bookticker_stream(&bookticker_symbol).await
    });
    let aggtrade_handle = tokio::spawn(async move {
        stream::aggtrade_stream::spawn_aggtrade_stream(&aggtrade_symbol).await
    });

    let user_stream_result = stream::user_stream::spawn_user_stream().await;
    bookticker_handle.abort();
    aggtrade_handle.abort();
    user_stream_result?;

    Ok(())
}

async fn initialize_orderbook(
    symbol: &str,
) -> Result<OrderbookState, Box<dyn Error + Send + Sync>> {
    let url = format!(
        "{}/api/v3/depth?symbol={}&limit=10",
        BINANCE_REST_BASE,
        symbol.to_uppercase()
    );
    let response = reqwest::get(url).await?;
    let payload: Value = response.json().await?;

    let bids = parse_levels(payload.get("bids"))?;
    let asks = parse_levels(payload.get("asks"))?;

    Ok(OrderbookState { bids, asks })
}

fn parse_levels(source: Option<&Value>) -> Result<Vec<OrderbookLevel>, Box<dyn Error + Send + Sync>> {
    let levels = source
        .and_then(|value| value.as_array())
        .ok_or("missing orderbook levels")?;

    let mut parsed = Vec::with_capacity(10);
    for level in levels.iter().take(10) {
        let entries = level
            .as_array()
            .ok_or("invalid orderbook level format")?;
        let price = entries
            .get(0)
            .and_then(|value| value.as_str())
            .ok_or("missing price")?
            .parse::<f64>()?;
        let qty = entries
            .get(1)
            .and_then(|value| value.as_str())
            .ok_or("missing quantity")?
            .parse::<f64>()?;
        parsed.push(OrderbookLevel::new(price, qty));
    }

    Ok(parsed)
}
