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
use crate::app::extractors::auth::AuthUser;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::{StatusOk, ok_status};
use crate::app::services::tier_list::check_permission;
use crate::app::services::tier_list::invalidate_detail;
use crate::app::state::AppState;
use crate::app::validation::validate_hex_color;
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::{TierListFlair, TierListStats};
use crate::database::queries::tier_lists as queries;
use crate::database::queries::tier_lists::add_favorite;
use crate::database::queries::tier_lists::is_favorited;
use crate::database::queries::tier_lists::remove_favorite;
use ts_rs::TS;

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
const VIEW_RATE_WINDOW: Duration = Duration::from_mins(1);
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

/// Whether a `POST /tier-lists/{slug}/view` counted as a new view.
///
/// Serialized directly by the handler, so this declaration is the wire format,
/// the `OpenAPI` schema and the generated TypeScript at once.
#[derive(serde::Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct ViewRecorded {
    /// False when the same user or session already counted for this list.
    pub unique: bool,
}

/// Whether the caller has this tier list favourited.
#[derive(serde::Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct FavoriteState {
    pub favorited: bool,
}

/// Whether a tier list appears in the public index.
///
/// Hiding a list does not make it private: a direct link still resolves.
#[derive(serde::Serialize, TS, utoipa::ToSchema)]
#[ts(export)]
pub struct VisibilityState {
    pub is_listed: bool,
}

/// Count one view of a tier list.
///
/// Deduplicated server-side, so a client may call it on every page load.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/view",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Whether this call counted as a new view.", body = ViewRecorded),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn record_view(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    headers: HeaderMap,
    Path(slug): Path<String>,
) -> Result<Json<ViewRecorded>, ApiError> {
    let list = super::load_tier_list(&state, &slug).await?;
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

    // Deliberately no invalidate_detail: views are high-frequency and a briefly
    // stale view count in the cached detail is an acceptable trade for not
    // churning the cache on every page load.
    let unique = queries::record_view(&state.db, list.id, user_id, session_hash.as_deref()).await?;
    Ok(Json(ViewRecorded { unique }))
}

/// View and favourite counts for a tier list.
#[utoipa::path(
    get,
    path = "/tier-lists/{slug}/stats",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    responses(
        (status = 200, description = "Engagement counters.", body = TierListStats),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_stats(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<TierListStats>, ApiError> {
    let list = super::load_tier_list(&state, &slug).await?;
    let stats = queries::get_stats(&state.db, list.id)
        .await?
        .ok_or(ApiError::NotFound)?;
    Ok(Json(stats))
}

/// Favourite or unfavourite a tier list.
#[utoipa::path(
    post,
    path = "/tier-lists/{slug}/favorite",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The favourite state after the toggle.", body = FavoriteState),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn toggle_favorite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<FavoriteState>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    let favorited = if is_favorited(&state.db, list.id, user_id).await? {
        remove_favorite(&state.db, list.id, user_id).await?;
        false
    } else {
        add_favorite(&state.db, list.id, user_id).await?;
        true
    };
    invalidate_detail(&state, &slug).await;
    Ok(Json(FavoriteState { favorited }))
}

/// Whether the caller has favourited this list.
#[utoipa::path(
    get,
    path = "/tier-lists/{slug}/favorite",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The caller's favourite state.", body = FavoriteState),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_favorite(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
) -> Result<Json<FavoriteState>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    let favorited = is_favorited(&state.db, list.id, user_id).await?;
    Ok(Json(FavoriteState { favorited }))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SetFlairRequest {
    pub flair_id: Option<i16>,
}

/// Attach a flair to a tier list, or clear it with a null id.
#[utoipa::path(
    put,
    path = "/tier-lists/{slug}/flair",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = SetFlairRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The flair was set.", body = crate::app::routes::StatusOk),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn set_flair(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<SetFlairRequest>,
) -> Result<Json<StatusOk>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    check_permission(&state, &list, user_id, auth.role, Permission::Edit).await?;
    queries::set_flair(&state.db, list.id, body.flair_id).await?;
    invalidate_detail(&state, &slug).await;
    Ok(ok_status())
}

/// Every flair a tier list can carry.
#[utoipa::path(
    get,
    path = "/tier-list-flairs",
    tag = "tier-lists",
    responses(
        (status = 200, description = "The flair catalog.", body = Vec<TierListFlair>),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn list_flairs(
    State(state): State<AppState>,
) -> Result<Json<Vec<TierListFlair>>, ApiError> {
    Ok(Json(queries::list_flairs(&state.db, true).await?))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct SetVisibilityRequest {
    pub is_listed: bool,
}

/// Show or hide a tier list in the public index.
///
/// Hiding does not make it private; a direct link still resolves.
#[utoipa::path(
    put,
    path = "/tier-lists/{slug}/visibility",
    tag = "tier-lists",
    params(
        ("slug" = String, Path, description = "Tier list slug, as it appears in its URL.")
    ),
    request_body = SetVisibilityRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The visibility as stored.", body = VisibilityState),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn set_visibility(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<SetVisibilityRequest>,
) -> Result<Json<VisibilityState>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = super::load_tier_list(&state, &slug).await?;
    check_permission(&state, &list, user_id, auth.role, Permission::Edit).await?;
    queries::set_visibility(&state.db, list.id, body.is_listed).await?;
    invalidate_detail(&state, &slug).await;
    Ok(Json(VisibilityState {
        is_listed: body.is_listed,
    }))
}

#[derive(Deserialize, utoipa::ToSchema)]
pub struct CreateFlairRequest {
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub display_order: Option<i16>,
}

/// Add a flair to the catalog.
#[utoipa::path(
    post,
    path = "/tier-list-flairs",
    tag = "tier-lists",
    request_body = CreateFlairRequest,
    security(("bearer_auth" = []), ("service_key" = [])),
    responses(
        (status = 200, description = "The created flair.", body = TierListFlair),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 409, response = crate::app::openapi::responses::Conflict),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn create_flair(
    State(state): State<AppState>,
    auth: AuthUser,
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
