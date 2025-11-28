use crate::app::{error::ApiError, state::AppState};
use axum::{
    Json,
    extract::{Path, State},
};

use crate::database::models::user::User;

pub async fn get_user(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<User>, ApiError> {
    User::find_by_uid(&state.db, &uid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            ApiError::Internal("Internal server error.".into())
        })?
        .map(Json)
        .ok_or(ApiError::NotFound("No user found.".into()))
}
