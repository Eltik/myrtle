use crate::app::state::AppState;
use crate::database::queries::operator_ownership::{
    latest_ownership_refresh_at, refresh_build_stats,
};
use chrono::Utc;
use std::time::Duration;

const DAY: Duration = Duration::from_hours(24);
const RETRY: Duration = Duration::from_hours(1);

/// How often the loop folds in roster syncs; a refreshing user sees community
/// numbers move within this window. Measured: a full recompute is 62.531 ms over
/// 619,706 rows, so 1,440 passes/day is 90 s of DB time. At 10x the data that's
/// 900 s/day, still fine; widen when it isn't.
const TICK: Duration = Duration::from_mins(1);

/// From the persisted time, so a restart right after a refresh doesn't
/// recompute. Startup only; the steady cadence is paced in memory.
async fn initial_delay(state: &AppState) -> Duration {
    match latest_ownership_refresh_at(&state.db).await {
        Ok(Some(last)) => {
            let elapsed = Utc::now()
                .signed_duration_since(last)
                .to_std()
                .unwrap_or(Duration::ZERO);
            DAY.checked_sub(elapsed).unwrap_or(Duration::ZERO)
        }
        Ok(None) => Duration::ZERO,
        Err(e) => {
            tracing::warn!(error = %e, "could not read last ownership refresh time; retrying in 1h");
            RETRY
        }
    }
}

/// Recompute, then drop cached responses so the new aggregate is served now,
/// not after the TTL. Shared with `core::refresh` so forced and scheduled runs
/// can't drift. Reports row counts: an empty aggregate (nobody opted in) is
/// legitimate and otherwise looks like a silent failure.
pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    refresh_build_stats(&state.db).await?;

    state
        .cache
        .invalidate_by_prefix("operators:ownership:")
        .await;
    state
        .cache
        .invalidate_by_prefix("operators:buildstats:")
        .await;

    let (owners, e2, skills, modules): (i64, i64, i64, i64) = sqlx::query_as(
        "SELECT (SELECT COUNT(*) FROM operator_ownership_stats), \
                (SELECT COALESCE(SUM(e2_owners), 0)::BIGINT FROM operator_ownership_stats), \
                (SELECT COUNT(*) FROM operator_skill_choice_stats), \
                (SELECT COUNT(*) FROM operator_module_choice_stats)",
    )
    .fetch_one(&state.db)
    .await?;

    Ok(format!(
        "{owners} ownership rows ({e2} E2 owners), {skills} skill-choice rows, {modules} module-choice rows"
    ))
}

/// Success schedules the next pass a day out; failure retries in an hour.
async fn refresh(state: &AppState, reason: &'static str) -> Duration {
    match refresh_once(state).await {
        Ok(summary) => {
            tracing::info!(reason, %summary, "operator ownership stats refreshed");
            DAY
        }
        Err(e) => {
            tracing::warn!(error = %e, reason, "operator ownership refresh failed; retrying in 1h");
            RETRY
        }
    }
}

async fn run_loop(state: AppState) {
    tracing::info!(
        tick_secs = TICK.as_secs(),
        "operator ownership refresh job started (daily floor, coalesced on sync)"
    );
    // Paced in memory, not from the persisted time: the aggregate is legitimately
    // empty until users opt in, and an empty table would read as "never
    // refreshed" and spin.
    let mut wait = initial_delay(&state).await;
    loop {
        tracing::debug!(
            wait_secs = wait.as_secs(),
            "sleeping until next ownership refresh"
        );

        let mut remaining = wait;
        let mut dirty = false;
        while !remaining.is_zero() {
            let slice = remaining.min(TICK);
            tokio::time::sleep(slice).await;
            remaining -= slice;
            if state.take_ownership_dirty() {
                dirty = true;
                break;
            }
        }

        wait = refresh(&state, if dirty { "roster sync" } else { "daily" }).await;
    }
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        crate::core::jobs::stagger("operator_ownership").await;
        run_loop(state).await;
    });
}
