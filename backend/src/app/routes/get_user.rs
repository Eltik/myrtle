use crate::app::{error::ApiError, state::AppState};
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;

use crate::database::models::user::User;

#[derive(Deserialize)]
pub struct GetUserQuery {
    pub uid: Option<String>,
}

// /get-user?uid={uid}
pub async fn get_user_by_query(
    State(state): State<AppState>,
    Query(params): Query<GetUserQuery>,
) -> Result<Json<User>, ApiError> {
    let uid = params
        .uid
        .ok_or(ApiError::BadRequest("Missing 'uid' parameter.".into()))?;
    find_user(&state, &uid).await
}

// /get-user/{uid}
pub async fn get_user_by_path(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Json<User>, ApiError> {
    if uid.is_empty() {
        return Err(ApiError::BadRequest("UID cannot be empty.".into()));
    }
    find_user(&state, &uid).await
}

async fn find_user(state: &AppState, uid: &str) -> Result<Json<User>, ApiError> {
    User::find_by_uid(&state.db, uid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            ApiError::Internal("Internal server error.".into())
        })?
        .map(Json)
        .ok_or(ApiError::NotFound("No user found.".into()))
}
