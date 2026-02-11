use std::env;

const DEFAULT_SPOT_REST_BASE: &str = "https://api.binance.com/api";
const DEFAULT_SPOT_WS_BASE: &str = "wss://stream.binance.com:9443";
const DEFAULT_FUTURES_REST_BASE: &str = "https://fapi.binance.com";
const DEFAULT_FUTURES_WS_BASE: &str = "wss://fstream.binance.com";

pub fn spot_rest_base() -> String {
    env::var("BINANCE_REST_BASE_URL").unwrap_or_else(|_| DEFAULT_SPOT_REST_BASE.to_string())
}

pub fn spot_ws_base() -> String {
    env::var("BINANCE_WS_BASE_URL").unwrap_or_else(|_| DEFAULT_SPOT_WS_BASE.to_string())
}

pub fn futures_rest_base() -> String {
    env::var("BINANCE_FUTURES_REST_BASE_URL")
        .unwrap_or_else(|_| DEFAULT_FUTURES_REST_BASE.to_string())
}

pub fn futures_ws_base() -> String {
    env::var("BINANCE_FUTURES_WS_BASE_URL").unwrap_or_else(|_| DEFAULT_FUTURES_WS_BASE.to_string())
}

pub fn spot_rest_url(path: &str) -> String {
    let base = spot_rest_base();
    let normalized_base = base.trim_end_matches('/');
    let normalized_path = path.trim_start_matches('/');

    if normalized_base.ends_with("/api") {
        format!("{normalized_base}/{normalized_path}")
    } else {
        format!("{normalized_base}/api/{normalized_path}")
    }
}

pub fn futures_rest_url(path: &str) -> String {
    let base = futures_rest_base();
    format!(
        "{}/{}",
        base.trim_end_matches('/'),
        path.trim_start_matches('/')
    )
}

pub fn ws_stream_url(ws_base: &str, stream_name: &str) -> String {
    format!(
        "{}/ws/{}",
        ws_base.trim_end_matches('/'),
        stream_name.trim_start_matches('/')
    )
}
