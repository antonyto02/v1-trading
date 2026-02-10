use crate::logic::futures::closeshort::closeshort;
use crate::logic::futures::open_shory::openshort;
use crate::state::market_price::{get_market_price_state_snapshot, set_market_price_state};

pub fn evaluate_level(best_bid: f64) {
    let mut market_price_state = get_market_price_state_snapshot();

    match market_price_state.last_best_bid {
        Some(last_best_bid) if (best_bid - last_best_bid).abs() < f64::EPSILON => {
            return;
        }
        Some(last_best_bid) if best_bid > last_best_bid => {
            closeshort(best_bid);
        }
        Some(_) => {
            openshort(best_bid);
        }
        None => {}
    }

    market_price_state.last_best_bid = Some(best_bid);
    set_market_price_state(market_price_state);
}
