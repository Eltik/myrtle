use crate::app::state::AppState;
use crate::database::queries::medal_ownership::{
    latest_medal_ownership_refresh_at, refresh_medal_ownership,
};
use chrono::Utc;
use std::time::Duration;

const DAY: Duration = Duration::from_hours(24);
const RETRY: Duration = Duration::from_hours(1);

/// From the persisted time, so a restart right after a refresh doesn't
/// recompute. Startup only; the steady cadence is paced in memory.
async fn initial_delay(state: &AppState) -> Duration {
    match latest_medal_ownership_refresh_at(&state.db).await {
        Ok(Some(last)) => {
            let elapsed = Utc::now()
                .signed_duration_since(last)
                .to_std()
                .unwrap_or(Duration::ZERO);
            DAY.checked_sub(elapsed).unwrap_or(Duration::ZERO)
        }
        Ok(None) => Duration::ZERO,
        Err(e) => {
            tracing::warn!(error = %e, "could not read last medal ownership refresh time; retrying in 1h");
            RETRY
        }
    }
}

/// Shared by the loop and `core::refresh`. Reports the row count: an empty
/// aggregate (nobody sharing) is legitimate and otherwise looks like a silent
/// failure.
pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    refresh_medal_ownership(&state.db).await?;
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM medal_ownership_stats")
        .fetch_one(&state.db)
        .await?;
    Ok(format!("{rows} medal ownership rows"))
}

async fn run_loop(state: AppState) {
    tracing::info!("medal ownership refresh job started (daily cadence)");
    // Paced in memory, not from the persisted time: the aggregate is legitimately
    // empty until users opt in, and an empty table would read as "never
    // refreshed" and spin.
    let mut wait = initial_delay(&state).await;
    loop {
        tracing::debug!(
            wait_secs = wait.as_secs(),
            "sleeping until next medal ownership refresh"
        );
        tokio::time::sleep(wait).await;
        wait = match refresh_medal_ownership(&state.db).await {
            Ok(()) => {
                tracing::info!("medal ownership stats refreshed");
                DAY
            }
            Err(e) => {
                tracing::warn!(error = %e, "medal ownership refresh failed; retrying in 1h");
                RETRY
            }
        };
    }
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        crate::core::jobs::stagger("medal_ownership").await;
        run_loop(state).await;
    });
}
