use crate::app::{error::ApiError, state::AppState};
use crate::core::authentication::jwt;
use crate::database::models::user::User;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub token: String,
    pub settings: Value,
}

#[derive(Serialize)]
pub struct UpdateSettingsResponse {
    pub success: bool,
    pub settings: Value,
}

/// POST /settings/update
/// Updates user settings (visibility, preferences, etc.)
/// Requires JWT token for authentication
pub async fn update_settings(
    State(state): State<AppState>,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<UpdateSettingsResponse>, ApiError> {
    // 1. Verify JWT token
    let claims = jwt::verify_token(&state.jwt_secret, &body.token)
        .map_err(|_| ApiError::BadRequest("Invalid or expired token.".into()))?;

    // 2. Find user by uid from token claims
    let user = User::find_by_uid(&state.db, &claims.uid)
        .await
        .map_err(|e| {
            eprintln!("Database error: {e:?}");
            ApiError::Internal("Internal server error.".into())
        })?
        .ok_or(ApiError::NotFound("User not found.".into()))?;

    // 3. Merge new settings with existing settings
    let merged_settings = merge_settings(user.settings.clone(), body.settings);

    // 4. Update settings in database
    let updated_user = User::update_settings(&state.db, user.id, merged_settings)
        .await
        .map_err(|e| {
            eprintln!("Failed to update settings: {e:?}");
            ApiError::Internal("Failed to update settings.".into())
        })?;

    Ok(Json(UpdateSettingsResponse {
        success: true,
        settings: updated_user.settings,
    }))
}

/// Merge new settings into existing settings
/// New values override existing ones, but unspecified keys are preserved
fn merge_settings(mut existing: Value, new: Value) -> Value {
    if let (Value::Object(existing_map), Value::Object(new_map)) = (&mut existing, new) {
        for (key, value) in new_map {
            existing_map.insert(key, value);
        }
    }
    existing
}
