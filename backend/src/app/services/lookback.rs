//! Account Lookback: the snapshot the roster sync takes once a period, and
//! the timeline reads.
//!
//! Nothing here goes through `cpu::run`: a snapshot is one pass over the
//! account's operator rows (236 on average, 412 at most on the local copy),
//! a few microseconds of counting next to the queries it waits on.

use chrono::Utc;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::core::lookback::{compact, current_period, expand, level_cap, ownership_series};
use crate::database::models::lookback::{LookbackBuild, LookbackOverview};
use crate::database::queries::lookback;

/// The server an account's `users.server_id` names; EN for an unknown index.
fn server_of(server_id: i16) -> Server {
    Server::all()
        .iter()
        .copied()
        .find(|s| s.index() as i16 == server_id)
        .unwrap_or(Server::EN)
}

/// Take `user_id`'s snapshot for the current period unless it has one.
/// Called by the roster sync after the grade, so the summary carries the
/// fresh score when the grade made its budget. Inside a period that has its
/// snapshot this costs one primary-key probe. Returns whether a row was written.
pub async fn record(state: &AppState, user_id: Uuid, server: Server) -> Result<bool, ApiError> {
    let game_data = state.game_data(server);
    let period = current_period(game_data.activities.values(), Utc::now().timestamp());
    if lookback::period_taken(&state.db, user_id, period.start).await? {
        return Ok(false);
    }
    let Some((_, extras)) = lookback::summary_extras(&state.db, user_id).await? else {
        return Ok(false);
    };
    let rows = lookback::build_rows(&state.db, user_id).await?;
    let (summary, build) = compact(&rows, extras, |id, elite| {
        level_cap(&game_data.operators, id, elite)
    });
    Ok(lookback::insert(&state.db, user_id, &period, &summary, &build).await?)
}

/// The timeline: every snapshot's counts and the reconstructed ownership line.
pub async fn overview(state: &AppState, user_id: Uuid) -> Result<LookbackOverview, ApiError> {
    let snapshots = lookback::list(&state.db, user_id).await?;
    let ownership = ownership_series(lookback::obtained_times(&state.db, user_id).await?);
    Ok(LookbackOverview {
        snapshots,
        ownership,
    })
}

/// One stored snapshot, by its key (`period_start` in Unix seconds).
pub async fn snapshot(
    state: &AppState,
    user_id: Uuid,
    id: i64,
) -> Result<Option<LookbackBuild>, ApiError> {
    Ok(lookback::get(&state.db, user_id, id)
        .await?
        .map(|(taken_at, summary, build)| LookbackBuild {
            taken_at,
            summary,
            operators: expand(build),
        }))
}

/// The roster as of its last sync, compacted the way a snapshot would be: the
/// right-hand side of a "since then" diff. `None` when the account is gone.
pub async fn current(state: &AppState, user_id: Uuid) -> Result<Option<LookbackBuild>, ApiError> {
    let Some((server_id, extras)) = lookback::summary_extras(&state.db, user_id).await? else {
        return Ok(None);
    };
    let game_data = state.game_data(server_of(server_id));
    let rows = lookback::build_rows(&state.db, user_id).await?;
    let (summary, build) = compact(&rows, extras, |id, elite| {
        level_cap(&game_data.operators, id, elite)
    });
    Ok(Some(LookbackBuild {
        taken_at: Utc::now(),
        summary,
        operators: expand(build),
    }))
}
