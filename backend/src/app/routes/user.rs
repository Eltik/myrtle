use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::{error::ApiError, services, state::AppState};
use crate::database::models::user::{UserCheckin, UserProfile};
use crate::database::queries::score::get_score_by_uid;
use crate::database::queries::users::get_checkin_by_uid;

#[derive(Deserialize)]
pub struct GetUserParams {
    pub uid: String,
}

/// A player's profile. `UserProfile` carries currency, sanity, subscription
/// expiry, last-online time and the account's role, so the `uid` goes through
/// the shared privacy gate: the caller's own profile or a public one, 403
/// otherwise.
pub async fn get_user(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let profile = services::user::get_user(&state, &uid).await?;
    Ok(Json(profile))
}

pub async fn get_user_score(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Return the full user_scores row (all category scores + grade + timestamp)
    // for the Score tab's detailed breakdown.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let score = get_score_by_uid(&state.db, &uid).await?;
    let body = match score {
        Some(s) => serde_json::to_value(&s).map_err(|e| ApiError::Internal(e.into()))?,
        None => serde_json::Value::Null,
    };
    Ok(Json(body))
}

pub async fn get_user_checkin(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<GetUserParams>,
) -> Result<Json<Option<UserCheckin>>, ApiError> {
    // Daily sign-in state: the month's claim count and per-claim monthly-card
    // flags, the lifetime total, and the active series' progress. `null` when
    // the user has never synced.
    let uid = resolve_uid(&state, &auth, Some(&params.uid)).await?;
    let checkin = get_checkin_by_uid(&state.db, &uid).await?;
    Ok(Json(checkin))
}
