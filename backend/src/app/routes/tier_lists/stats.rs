use std::sync::LazyLock;
use std::time::{Duration, Instant};

use axum::http::HeaderMap;
use axum::{
    Json,
    extract::{Path, State},
};
use dashmap::DashMap;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::app::validation::validate_hex_color;
use crate::database::models::tier_list::{TierListFlair, TierListStats};
use crate::database::queries::tier_lists as queries;

type HmacSha256 = Hmac<Sha256>;

/// Derive the stored anonymous-dedupe key from a client-supplied session id.
/// The DB column is `CHAR(64)`, matching the hex-encoded HMAC-SHA256 length.
/// Keying with `jwt_secret` (treated here as a server-side pepper) ensures a
/// DB dump alone can't be cross-referenced with a stolen `mtl_sid` cookie.
fn hash_session_id(secret: &str, session_id: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any length");
    mac.update(session_id.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

const SESSION_COOKIE: &str = "mtl_sid";
const SESSION_ID_LEN: usize = 64;
const VIEW_RATE_WINDOW: Duration = Duration::from_secs(60);
const VIEW_RATE_LIMIT: u32 = 10;

fn valid_session_id(value: &str) -> bool {
    value.len() == SESSION_ID_LEN && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn extract_session_id(headers: &HeaderMap) -> Option<String> {
    if let Some(v) = headers.get("x-session-id").and_then(|v| v.to_str().ok())
        && valid_session_id(v)
    {
        return Some(v.to_owned());
    }
    let cookies = headers.get("cookie").and_then(|v| v.to_str().ok())?;
    for part in cookies.split(';') {
        if let Some(value) = part.trim().strip_prefix(&format!("{SESSION_COOKIE}="))
            && valid_session_id(value)
        {
            return Some(value.to_owned());
        }
    }
    None
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    let raw = headers
        .get("x-forwarded-for")
        .or_else(|| headers.get("x-real-ip"))
        .and_then(|v| v.to_str().ok())?;
    let first = raw.split(',').next()?.trim();
    (!first.is_empty()).then(|| first.to_owned())
}

static VIEW_RATE_BUCKETS: LazyLock<DashMap<String, (Instant, u32)>> = LazyLock::new(DashMap::new);

fn check_view_rate(ip: &str) -> bool {
    let now = Instant::now();
    let mut entry = VIEW_RATE_BUCKETS.entry(ip.to_owned()).or_insert((now, 0));
    let (window_start, count) = *entry;
    if now.duration_since(window_start) >= VIEW_RATE_WINDOW {
        *entry = (now, 1);
        true
    } else if count >= VIEW_RATE_LIMIT {
        false
    } else {
        entry.1 = count + 1;
        true
    }
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

    if let Some(ip) = client_ip(&headers)
        && !check_view_rate(&ip)
    {
        return Err(ApiError::RateLimited);
    }

    let session_hash = if user_id.is_some() {
        None
    } else {
        extract_session_id(&headers).map(|sid| hash_session_id(&state.config.jwt_secret, &sid))
    };

    let unique = queries::record_view(&state.db, list.id, user_id, session_hash.as_deref()).await?;
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

pub async fn get_favorite(
    State(state): State<AppState>,
    auth: crate::app::extractors::auth::AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| ApiError::Unauthorized)?;
    let list = queries::find_by_slug(&state.db, &slug)
        .await?
        .ok_or(ApiError::NotFound)?;
    let favorited = queries::is_favorited(&state.db, list.id, user_id).await?;
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
    validate_hex_color(body.color.as_deref())?;
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
