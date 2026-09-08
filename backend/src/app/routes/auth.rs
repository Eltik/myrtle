use axum::{Json, extract::State};
use serde::Deserialize;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::services;
use crate::app::services::auth::parse_server;
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
    let server = parse_server(&body.server)?;
    services::auth::send_code(&state, &body.email, server).await?;
    Ok(ok_status())
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
    let server = parse_server(&body.server)?;
    let result = services::auth::login(&state, &body.email, &body.code, server).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct BilibiliLoginRequest {
    pub username: String,
    pub password: String,
}

pub async fn login_bilibili(
    State(state): State<AppState>,
    Json(body): Json<BilibiliLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result = services::auth::login_bilibili(&state, &body.username, &body.password).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct BilibiliSendSmsRequest {
    pub phone: String,
}

pub async fn send_bilibili_sms(
    State(state): State<AppState>,
    Json(body): Json<BilibiliSendSmsRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    services::auth::send_bilibili_sms(&state, &body.phone).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct BilibiliSmsLoginRequest {
    pub phone: String,
    pub code: String,
}

/// Experimental: see `core::hypergryph::bilibili` module docs. The SMS
/// endpoints themselves are unverified guesses, unlike username/password
/// login.
pub async fn login_bilibili_sms(
    State(state): State<AppState>,
    Json(body): Json<BilibiliSmsLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result = services::auth::login_bilibili_sms(&state, &body.phone, &body.code).await?;
    Ok(Json(result))
}

#[derive(Deserialize)]
pub struct CnSendCodeRequest {
    pub phone: String,
}

pub async fn send_code_cn(
    State(state): State<AppState>,
    Json(body): Json<CnSendCodeRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    services::auth::send_code_cn(&state, &body.phone).await?;
    Ok(ok_status())
}

#[derive(Deserialize)]
pub struct CnLoginRequest {
    pub phone: String,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

/// Experimental: see `core::hypergryph::passport` module docs. Expected to
/// fail until the real game-client appCode is known.
pub async fn login_cn(
    State(state): State<AppState>,
    Json(body): Json<CnLoginRequest>,
) -> Result<Json<services::auth::LoginResponse>, ApiError> {
    let result = services::auth::login_cn(
        &state,
        &body.phone,
        body.password.as_deref(),
        body.code.as_deref(),
    )
    .await?;
    Ok(Json(result))
}

pub async fn verify(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    // AuthUser extractor already validated the token. Re-read the user's
    // role from the DB so admin promotions/demotions take effect on the next
    // `verify` call without requiring the user to log out and back in
    // (JWT claims freeze the role at login time).
    let db_role: Option<String> = if let Ok(uuid) = uuid::Uuid::parse_str(&auth.user_id) {
        sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE id = $1")
            .bind(uuid)
            .fetch_optional(&state.db)
            .await?
    } else {
        None
    };

    let role = db_role.unwrap_or_else(|| auth.role.to_string());

    Ok(Json(serde_json::json!({
        "valid": true,
        "userId": auth.user_id,
        "uid": auth.uid,
        "server": auth.server,
        "role": role,
    })))
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
    let user_id: uuid::Uuid = auth.user_uuid()?;
    services::auth::update_settings(
        &state,
        user_id,
        body.public_profile,
        body.store_gacha,
        body.share_stats,
    )
    .await?;
    // v_user_profile is read through a 10-minute cache; without this, fresh
    // reads after the write would still return the old settings.
    state
        .cache
        .invalidate(&CacheKey::User { uid: &auth.uid })
        .await;
    Ok(ok_status())
}

/// Forget the stored game credentials for the caller's own account.
///
/// The site session stays valid and the already-synced data stays put; what is
/// revoked is our ability to reach Yostar on their behalf. Re-syncing then
/// needs a fresh email code, which is the point — before this, the only way to
/// withdraw that access was to email us and have the whole account deleted.
///
/// Idempotent: disconnecting twice is not an error. `removed` reports whether
/// there was anything stored to remove.
pub async fn disconnect(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id = auth.user_uuid()?;
    let removed = services::game_session::disconnect(&state, &auth.uid, user_id).await?;
    Ok(Json(
        serde_json::json!({ "status": "ok", "removed": removed }),
    ))
}

pub async fn refresh(
    State(state): State<AppState>,
    auth: AuthUser,
) -> Result<Json<serde_json::Value>, ApiError> {
    let server = parse_server(&auth.server)?;
    let data = services::roster::refresh(&state, &auth.uid, server).await?;
    Ok(Json(data))
}
