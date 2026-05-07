use crate::app::state::AppState;
use crate::database::queries::score as queries;
use std::time::Duration;

async fn run_loop(state: AppState, interval_secs: u64) {
    tracing::info!(interval_secs, "leaderboard snapshot job started");
    let mut tick = tokio::time::interval(Duration::from_secs(interval_secs));
    tick.tick().await;
    loop {
        tick.tick().await;
        match queries::take_leaderboard_snapshot(&state.db).await {
            Ok(id) => tracing::debug!(snapshot_id = id, "leaderboard snapshot taken"),
            Err(e) => tracing::warn!(error = %e, "leaderboard snapshot failed"),
        }
    }
}

pub fn spawn(state: AppState) {
    let interval_secs: u64 = std::env::var("LEADERBOARD_SNAPSHOT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3600); // 1h default — gives "this week" 168 snapshots
    tokio::spawn(async move {
        run_loop(state, interval_secs).await;
    });
}
