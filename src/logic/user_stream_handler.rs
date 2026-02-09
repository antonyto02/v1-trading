use serde::Deserialize;

use chrono::Local;
use crate::logic::evaluate_buy_orders::EvaluateBuyOrders;
use crate::logic::generalFunctions::CleanOrder;
use crate::logic::order::{add_filled_buy_for_bid_price, add_filled_sell_for_ask_price};
use crate::state::asset::get_asset_state_snapshot;

fn log(message: &str) {
    println!("[{}] {message}", Local::now().format("%Y-%m-%d %H:%M:%S"));
}

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
    #[serde(rename = "X")]
    order_status: String,
}

pub async fn handle_user_stream_message(message: String) {
    log("Recibido mensaje de user stream. Iniciando parseo.");
    let event: ExecutionReportEvent = match serde_json::from_str(&message) {
        Ok(payload) => payload,
        Err(err) => {
            log(&format!("Error parseando JSON de user stream: {err}"));
            return;
        }
    };

    if event.event_type != "executionReport" {
        log(&format!(
            "Evento ignorado: event_type={} (esperado executionReport).",
            event.event_type
        ));
        return;
    }

    let asset_symbol = get_asset_state_snapshot().symbol;
    if event.symbol != asset_symbol {
        log(&format!(
            "Execution report ignorado: symbol={} no coincide con asset={asset_symbol}.",
            event.symbol
        ));
        return;
    }

    let price = match event
        .last_executed_price
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
    {
        Some(value) => value,
        None => {
            log("Execution report sin last_executed_price válido; se omite.");
            return;
        }
    };

    let quantity = match event
        .last_executed_quantity
        .as_deref()
        .and_then(|value| value.parse::<f64>().ok())
    {
        Some(value) => value,
        None => {
            log("Execution report sin last_executed_quantity válido; se omite.");
            return;
        }
    };

    if quantity <= 0.0 {
        log("Execution report con quantity <= 0; se omite.");
        return;
    }

    log(&format!(
        "Procesando execution report: side={}, status={}, price={}, quantity={}.",
        event.side, event.order_status, price, quantity
    ));

    match event.side.as_str() {
        "BUY" => {
            if let Err(err) = add_filled_buy_for_bid_price(price, quantity).await {
                log(&format!("Error en flujo BUY: {err}"));
            }
        }
        "SELL" => {
            match event.order_status.as_str() {
                "PARTIALLY_FILLED" => {
                    log("SELL PARTIALLY_FILLED: sumando filled_sell.");
                    add_filled_sell_for_ask_price(price, quantity);
                }
                "FILLED" => {
                    if let Some(order_index) = add_filled_sell_for_ask_price(price, quantity) {
                        log(&format!(
                            "SELL FILLED: limpiando orden index={order_index} y reevaluando buy orders."
                        ));
                        // CleanOrder(order_index);
                        // EvaluateBuyOrders().await;
                    } else {
                        log("SELL FILLED: no se encontró orden para limpiar.");
                    }
                }
                other => {
                    log(&format!("SELL con status no manejado: {other}."));
                }
            }
        }
        other => {
            log(&format!("Side no manejado en execution report: {other}."));
        }
    }
}
