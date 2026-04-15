use axum::{Json, extract::State};
use serde::Deserialize;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services;
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct SendCodeRequest {
    pub email: String,
    pub server: String,
}

pub async fn send_code(
    State(state): State<AppState>,
    Json(body): Json<SendCodeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = services::auth::parse_server(&body.server)?;
    services::auth::send_code(&state, &body.email, server).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub code: String,
    pub server: String,
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let server = services::auth::parse_server(&body.server)?;
    let result = services::auth::login(&state, &body.email, &body.code, server).await?;
    Ok(Json(result))
}

pub async fn verify(auth: AuthUser) -> Json<serde_json::Value> {
    // AuthUser extractor already validated the token.
    // `role` is surfaced so frontend API routes can make permission decisions
    // without a second round-trip (e.g. deciding whether a user may create an
    // official tier list vs a community one).
    Json(serde_json::json!({
        "valid": true,
        "userId": auth.user_id,
        "uid": auth.uid,
        "server": auth.server,
        "role": auth.role.to_string(),
    }))
}

#[derive(Deserialize)]
pub struct UpdateSettingsRequest {
    pub public_profile: bool,
    pub store_gacha: bool,
    pub share_stats: bool,
}

pub async fn update_settings(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(body): Json<UpdateSettingsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: uuid::Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    services::auth::update_settings(
        &state,
        user_id,
        body.public_profile,
        body.store_gacha,
        body.share_stats,
    )
    .await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn refresh(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = services::auth::parse_server(&auth.server)?;
    let data = services::roster::refresh(&state, &auth.uid, server).await?;
    Ok(Json(data))
}
