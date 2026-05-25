use crate::app::state::AppState;
use crate::database::queries::tier_lists as queries;
use std::time::Duration;

async fn run_loop(state: AppState, interval_secs: u64, top_n: i64) {
    tracing::info!(interval_secs, top_n, "trending job started");
    let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
    loop {
        tick.tick().await;
        match queries::recompute_trending(&state.db, top_n).await {
            Ok(()) => tracing::debug!("trending recomputed"),
            Err(e) => tracing::warn!(error = %e, "trending recompute failed"),
        }
    }
}

pub fn spawn(state: AppState) {
    let interval_secs: u64 = std::env::var("TRENDING_RECOMPUTE_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(900); // 15 min default
    let top_n: i64 = std::env::var("TRENDING_TOP_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10);

    tokio::spawn(async move {
        run_loop(state, interval_secs, top_n).await;
    });
}
