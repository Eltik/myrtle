use crate::app::{error::ApiError, state::AppState};
use crate::core::user::get::format_user;
use crate::core::user::types::User as UserData;
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
    let mut user = User::find_by_uid(&state.db, uid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {:?}", e);
            ApiError::Internal("Internal server error.".into())
        })?
        .ok_or(ApiError::NotFound("No user found.".into()))?;

    // Re-enrich user data with fresh game_data to ensure static fields are up-to-date
    if let Ok(mut user_data) = serde_json::from_value::<UserData>(user.data.clone()) {
        format_user(&mut user_data, &state.game_data);
        if let Ok(enriched_json) = serde_json::to_value(&user_data) {
            user.data = enriched_json;
        }
    }

    Ok(Json(user))
}
