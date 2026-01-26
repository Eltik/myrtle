use crate::app::{error::ApiError, state::AppState};
use crate::core::user::get::format_user;
use crate::core::user::types::User as UserData;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use serde::Deserialize;

use crate::database::models::user::User;

#[derive(Deserialize)]
pub struct GetUserQuery {
    pub uid: Option<String>,
    /// If true, bypass cache and re-enrich from database
    pub refresh: Option<bool>,
}

// Cache TTL for enriched user data (10 minutes)
const USER_CACHE_TTL: u64 = 600;

struct JsonResponse(String);

impl IntoResponse for JsonResponse {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/json")],
            self.0,
        )
            .into_response()
    }
}

// /get-user?uid={uid}
pub async fn get_user_by_query(
    State(state): State<AppState>,
    Query(params): Query<GetUserQuery>,
) -> Result<Response, ApiError> {
    let uid = params
        .uid
        .ok_or(ApiError::BadRequest("Missing 'uid' parameter.".into()))?;
    let bypass_cache = params.refresh.unwrap_or(false);
    find_user(&state, &uid, bypass_cache).await
}

// /get-user/{uid}
pub async fn get_user_by_path(
    State(state): State<AppState>,
    Path(uid): Path<String>,
) -> Result<Response, ApiError> {
    if uid.is_empty() {
        return Err(ApiError::BadRequest("UID cannot be empty.".into()));
    }
    find_user(&state, &uid, false).await
}

async fn find_user(state: &AppState, uid: &str, bypass_cache: bool) -> Result<Response, ApiError> {
    let cache_key = format!("user:enriched:{uid}");
    let mut redis = state.redis.clone();

    // Check Redis cache first (unless bypassing)
    if !bypass_cache
        && let Ok(cached) = redis.get::<_, String>(&cache_key).await
        && !cached.is_empty()
    {
        return Ok(JsonResponse(cached).into_response());
    }

    // Cache miss - fetch from database
    let user = User::find_by_uid(&state.db, uid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Internal server error.".into())
        })?
        .ok_or(ApiError::NotFound("No user found.".into()))?;

    // Enrich user data with static game data
    let enriched_data =
        if let Ok(mut user_data) = serde_json::from_value::<UserData>(user.data.clone()) {
            format_user(&mut user_data, &state.game_data);
            serde_json::to_value(&user_data).unwrap_or(user.data.clone())
        } else {
            // If parsing fails, return raw data
            user.data.clone()
        };

    // Build response with user metadata + enriched data
    let response = serde_json::json!({
        "id": user.id,
        "uid": user.uid,
        "server": user.server,
        "data": enriched_data,
        "settings": user.settings,
        "role": user.role,
        "score": user.score,
        "created_at": user.created_at,
        "updated_at": user.updated_at,
    });

    let json_str = serde_json::to_string(&response).map_err(|e| {
        eprintln!("JSON serialization error: {e:?}");
        ApiError::Internal("Failed to serialize response.".into())
    })?;

    // Cache the serialized JSON
    let _: Result<(), _> = redis.set_ex(&cache_key, &json_str, USER_CACHE_TTL).await;

    Ok(JsonResponse(json_str).into_response())
}
