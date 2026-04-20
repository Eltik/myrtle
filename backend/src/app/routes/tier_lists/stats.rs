use axum::http::HeaderMap;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::models::tier_list::{TierListFlair, TierListStats};
use crate::database::queries::tier_lists as queries;

fn session_hash(headers: &HeaderMap) -> Option<String> {
    let ua = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ip = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if ip.is_empty() && ua.is_empty() {
        return None;
    }
    let salt = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut h = Sha256::new();
    h.update(ip);
    h.update(ua);
    h.update(salt);
    let digest = h.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest.iter() {
        use std::fmt::Write;
        let _ = write!(hex, "{byte:02x}");
    }
    Some(hex)
}

pub async fn record_view(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    let user_id = auth
        .0
        .as_ref()
        .and_then(|a| Uuid::parse_str(&a.user_id).ok());
    let sh = if user_id.is_none() {
        session_hash(&headers)
    } else {
        None
    };
    let unique = queries::record_view(&state.db, list.id, user_id, sh.as_deref()).await?;
    Ok(Json(serde_json::json!({ "unique": unique })))
}

pub async fn get_stats(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<TierListStats>, ApiError> {
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    let stats = queries::get_stats(&state.db, list.id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(stats))
}

pub async fn toggle_favorite(
    State(state): State<AppState>,
    auth: crate::app::extractors::auth::AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    let favorited = if queries::is_favorited(&state.db, list.id, user_id).await? {
        queries::remove_favorite(&state.db, list.id, user_id).await?;
        false
    } else {
        queries::add_favorite(&state.db, list.id, user_id).await?;
        true
    };
    Ok(Json(serde_json::json!({ "favorited": favorited })))
}

#[derive(Deserialize)]
pub struct SetFlairRequest {
    pub flair_id: Option<i16>,
}

pub async fn set_flair(
    State(state): State<AppState>,
    auth: crate::app::extractors::auth::AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<SetFlairRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    use crate::core::auth::permissions::Permission;
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    crate::app::services::tier_list::check_permission(
        &state,
        &list,
        user_id,
        auth.role,
        Permission::Edit,
    )
    .await?;
    queries::set_flair(&state.db, list.id, body.flair_id).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub async fn list_flairs(
    State(state): State<AppState>,
) -> Result<Json<Vec<TierListFlair>>, ApiError> {
    Ok(Json(queries::list_flairs(&state.db, true).await?))
}

#[derive(Deserialize)]
pub struct CreateFlairRequest {
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: Option<i16>,
}

pub async fn create_flair(
    State(state): State<AppState>,
    auth: crate::app::extractors::auth::AuthUser,
    Json(body): Json<CreateFlairRequest>,
) -> Result<Json<TierListFlair>, ApiError> {
    if !auth.role.is_tier_list_admin() {
        return Err(ApiError::Forbidden);
    }
    let flair = queries::create_flair(
        &state.db,
        &body.code,
        &body.label,
        body.color.as_deref(),
        body.display_order.unwrap_or(0),
    )
    .await?;
    Ok(Json(flair))
}
