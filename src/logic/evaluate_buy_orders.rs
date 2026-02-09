use crate::logic::generalFunctions;

#[allow(non_snake_case)]
pub fn EvaluateBuyOrders() {
    let mut candidates = vec![0, 1, 2, 3];
    let _ = &mut candidates;

    let (best_bids, best_asks) = generalFunctions::get_best_bid_and_ask();
    println!("Best bids/asks: {:?}", (best_bids.clone(), best_asks.clone()));

    let (candidates, best_bids) =
        generalFunctions::ProcessFrozenBlocks(candidates, &best_bids, &best_asks);
    let (_candidates, _best_bids) =
        generalFunctions::ProcessActiveBuyOrders(candidates, &best_bids);
}
