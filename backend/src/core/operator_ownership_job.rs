use crate::app::state::AppState;
use crate::database::queries::operator_ownership::{
    latest_ownership_refresh_at, refresh_operator_ownership,
};
use chrono::Utc;
use std::time::Duration;

const DAY: Duration = Duration::from_hours(24);
const RETRY: Duration = Duration::from_hours(1);

/// How often the loop looks for a roster sync to fold in. A user who refreshes
/// sees the community numbers move within this window.
///
/// 60 s is taken from the measured cost, not guessed: the full recompute is
/// 62.531 ms over 619,706 rows, so the worst case of 1,440 passes a day is 90
/// seconds of database time. At ten times the data that is 900 seconds a day
/// and still comfortable; widen this when it stops being.
const TICK: Duration = Duration::from_secs(60);

/// How long to wait before the first refresh, derived from the persisted time
/// so a restart soon after a refresh does not recompute. Consulted only at
/// startup; the steady-state cadence is paced in memory.
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

/// One pass: recompute, then drop cached responses so the fresh aggregate is
/// served at once rather than after the cache TTL lapses.
///
/// Shared by the loop below and by `core::refresh`, so a forced refresh and a
/// scheduled one cannot drift apart. Reports the row counts because an empty
/// aggregate is a legitimate outcome (nobody has opted into stat sharing) and
/// is indistinguishable from a silent failure without them.
pub async fn refresh_once(state: &AppState) -> anyhow::Result<String> {
    refresh_operator_ownership(&state.db).await?;

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

/// Wrap [`refresh_once`] in the loop's pacing: a success schedules the next
/// unconditional pass a day out, a failure retries in an hour.
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
    // Pace the loop in memory rather than re-reading the persisted time each
    // iteration: the aggregate is legitimately empty until users opt into
    // sharing, and an empty table would otherwise read back as "never refreshed"
    // and spin.
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
        run_loop(state).await;
    });
}
