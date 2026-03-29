use axum::{
    Json,
    extract::{Query, State},
};
use serde::Deserialize;

use crate::app::{error::ApiError, services, state::AppState};
use crate::database::models::user::UserProfile;

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
