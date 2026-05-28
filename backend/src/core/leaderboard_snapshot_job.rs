use crate::app::state::AppState;
use crate::database::queries::score as queries;
use chrono::Utc;
use std::time::Duration;

const DAY: Duration = Duration::from_hours(24);

async fn delay_until_due(state: &AppState) -> Duration {
    match queries::latest_leaderboard_snapshot_at(&state.db).await {
        Ok(Some(last)) => {
            let elapsed = Utc::now().signed_duration_since(last);
            let elapsed = elapsed.to_std().unwrap_or(Duration::ZERO);
            DAY.checked_sub(elapsed).unwrap_or(Duration::ZERO)
        }
        Ok(None) => Duration::ZERO,
        Err(e) => {
            tracing::warn!(error = %e, "could not read last snapshot time; retrying in 1h");
            Duration::from_hours(1)
        }
    }
}

async fn run_loop(state: AppState) {
    tracing::info!("leaderboard snapshot job started (daily cadence)");
    loop {
        let wait = delay_until_due(&state).await;
        tracing::debug!(wait_secs = wait.as_secs(), "sleeping until next snapshot");
        tokio::time::sleep(wait).await;
        match queries::take_leaderboard_snapshot(&state.db).await {
            Ok(id) => tracing::info!(snapshot_id = id, "leaderboard snapshot taken"),
            Err(e) => {
                tracing::warn!(error = %e, "leaderboard snapshot failed; retrying in 1h");
                tokio::time::sleep(Duration::from_hours(1)).await;
            }
        }
    }
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        run_loop(state).await;
    });
}
