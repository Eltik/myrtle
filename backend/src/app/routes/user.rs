use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::{error::ApiError, services, state::AppState};
use crate::database::models::user::UserProfile;
use crate::database::queries::score as score_queries;

#[derive(Deserialize)]
pub struct GetUserParams {
    pub uid: String,
}

pub async fn get_user(
    State(state): State<AppState>,
    Query(params): Query<GetUserParams>,
) -> Result<Json<UserProfile>, ApiError> {
    let profile = services::user::get_user(&state, &params.uid).await?;
    Ok(Json(profile))
}

pub async fn get_user_score(
    State(state): State<AppState>,
    Query(params): Query<GetUserParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Return the full user_scores row (all category scores + grade + timestamp)
    // for the Score tab's detailed breakdown.
    let score = score_queries::get_score_by_uid(&state.db, &params.uid).await?;
    let body = match score {
        Some(s) => serde_json::to_value(&s).map_err(|e| ApiError::Internal(e.into()))?,
        None => serde_json::Value::Null,
    };
    Ok(Json(body))
}
