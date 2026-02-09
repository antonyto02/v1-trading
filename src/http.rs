use std::error::Error;

use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::asset::{get_asset_state_snapshot, AssetState};
use crate::state::orderbook::{get_orderbook_state_snapshot, OrderbookState};
use crate::state::orders::{get_orders_state_snapshot, OrdersState};

#[derive(Debug, Serialize)]
struct OrdersResponse {
    asset: AssetState,
    orderbook: OrderbookState,
    orders: OrdersState,
}

pub async fn run_server() -> Result<(), Box<dyn Error + Send + Sync>> {
    let app = Router::new().route("/orders", get(get_orders));
    let bind_addr = std::env::var("HTTP_BIND").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn get_orders() -> Json<OrdersResponse> {
    let response = OrdersResponse {
        asset: get_asset_state_snapshot(),
        orderbook: get_orderbook_state_snapshot(),
        orders: get_orders_state_snapshot(),
    };
    Json(response)
}
