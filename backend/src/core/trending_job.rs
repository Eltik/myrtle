use crate::app::state::AppState;
use crate::database::queries::tier_lists::recompute_trending;
use std::time::Duration;

/// How many entries the trending window ranks. Read here rather than at spawn
/// so the one-shot path and the loop agree without the caller passing it in.
fn top_n_from_env() -> i64 {
    std::env::var("TRENDING_TOP_N")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10)
}

/// One pass, shared by the loop and by `core::refresh`. See that module for why
/// every job exposes exactly this shape.
pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    let top_n = top_n_from_env();
    recompute_trending(&state.db, top_n).await?;
    Ok(format!(
        "recomputed trending windows (top {top_n} per list)"
    ))
}

async fn run_loop(state: AppState, interval_secs: u64, top_n: i64) {
    tracing::info!(interval_secs, top_n, "trending job started");
    let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
    loop {
        tick.tick().await;
        match recompute_trending(&state.db, top_n).await {
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
