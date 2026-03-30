use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;
use crate::database::models::roster::RosterEntry;
use crate::database::queries::roster;

pub async fn get_roster(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<Vec<RosterEntry>>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let entries = roster::get_roster(&state.db, user_id).await?;
    Ok(Json(entries))
}

pub async fn get_operator(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(operator_id): Path<String>,
) -> Result<Json<RosterEntry>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let entry = roster::get_operator(&state.db, user_id, &operator_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(entry))
}
