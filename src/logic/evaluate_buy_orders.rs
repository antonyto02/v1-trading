use crate::logic::generalFunctions;

#[allow(non_snake_case)]
pub async fn EvaluateBuyOrders() {
    let mut candidates = vec![0, 1, 2, 3];
    let _ = &mut candidates;

    let (best_bids, best_asks) = generalFunctions::get_best_bid_and_ask();

    let (candidates, best_bids) =
        generalFunctions::ProcessFrozenBlocks(candidates, &best_bids, &best_asks);
    let (candidates, best_bids) =
        generalFunctions::ProcessActiveBuyOrders(candidates, &best_bids).await;
    if let Err(error) = generalFunctions::FillMissingBestBids(&candidates, &best_bids).await {
        let _ = error;
    }
}
