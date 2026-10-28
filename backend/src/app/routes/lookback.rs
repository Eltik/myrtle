use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id_for_tabs;
use crate::app::services::lookback;
use crate::app::state::AppState;
use crate::database::models::lookback::{LookbackBuild, LookbackOverview};
use crate::database::models::profile_layout::ProfileTabId;

/// A snapshot shows the roster as it was, so a visitor needs the Roster tab
/// visible as well as the Lookback tab.
const TABS: &[ProfileTabId] = &[ProfileTabId::Lookback, ProfileTabId::Roster];

#[derive(Deserialize)]
pub struct LookbackParams {
    pub uid: Option<String>,
}

/// The account's Lookback timeline: one snapshot per event launch, each with
/// its headline counts, and the owned-operator count rebuilt from obtain times.
/// Runs the shared privacy gate, then needs the Lookback and Roster tabs
/// visible to a visitor.
#[utoipa::path(
    get,
    path = "/lookback",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Snapshots oldest first, and the reconstructed ownership line.", body = LookbackOverview),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn overview(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<LookbackParams>,
) -> Result<Json<LookbackOverview>, ApiError> {
    let user_id = resolve_user_id_for_tabs(&state, &auth, params.uid.as_deref(), TABS).await?;
    Ok(Json(lookback::overview(&state, user_id).await?))
}

/// One snapshot's whole build: every operator's promotion, level, potential,
/// skill rank, masteries and modules as of that period's first sync. Same gate
/// as `/lookback`.
#[utoipa::path(
    get,
    path = "/lookback/snapshots/{id}",
    tag = "player",
    params(
        ("id" = i64, Path, description = "The snapshot's `id` from `/lookback`: its period start in Unix seconds."),
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The snapshot's counts and operators.", body = LookbackBuild),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn snapshot(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Path(id): Path<i64>,
    Query(params): Query<LookbackParams>,
) -> Result<Json<LookbackBuild>, ApiError> {
    let user_id = resolve_user_id_for_tabs(&state, &auth, params.uid.as_deref(), TABS).await?;
    let build = lookback::snapshot(&state, user_id, id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(build))
}

/// The roster as of its last sync, in a snapshot's shape: what the timeline
/// diffs a snapshot against for "since then". Same gate as `/lookback`.
#[utoipa::path(
    get,
    path = "/lookback/current",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "The live counts and operators.", body = LookbackBuild),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn current(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<LookbackParams>,
) -> Result<Json<LookbackBuild>, ApiError> {
    let user_id = resolve_user_id_for_tabs(&state, &auth, params.uid.as_deref(), TABS).await?;
    let build = lookback::current(&state, user_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(build))
}
