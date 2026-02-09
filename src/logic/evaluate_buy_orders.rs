use crate::logic::generalFunctions;

#[allow(non_snake_case)]
pub fn EvaluateBuyOrders() {
    let mut candidates = vec![0, 1, 2, 3];
    let _ = &mut candidates;

    let best_prices = generalFunctions::get_best_bid_and_ask();
    println!("Best bids/asks: {:?}", best_prices);
}
