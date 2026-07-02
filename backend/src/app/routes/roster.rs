use axum::Json;
use axum::extract::{Path, Query, State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id;
use crate::app::state::AppState;
use crate::database::models::roster::{RosterEntry, SupportUnit};
use crate::database::queries::roster;

#[derive(Deserialize)]
pub struct RosterParams {
    pub uid: Option<String>,
}

pub async fn get_roster(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<RosterParams>,
) -> Result<Json<Vec<RosterEntry>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = roster::get_roster(&state.db, user_id).await?;
    Ok(Json(entries))
}

pub async fn get_operator(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Path(operator_id): Path<String>,
    Query(params): Query<RosterParams>,
) -> Result<Json<RosterEntry>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entry = roster::get_operator(&state.db, user_id, &operator_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(entry))
}

pub async fn get_supports(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<RosterParams>,
) -> Result<Json<Vec<SupportUnit>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = roster::get_supports(&state.db, user_id).await?;
    Ok(Json(entries))
}
