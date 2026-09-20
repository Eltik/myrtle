use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_user_id;
use crate::app::routes::static_data::json_response;
use crate::app::services::static_data::get_skins_index;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
use crate::database::queries::skins;
use crate::database::queries::skins::OwnedSkin;

#[derive(Deserialize)]
pub struct SkinsParams {
    pub uid: Option<String>,
}

/// `GET /skins/index` - slim `skinId -> {charId, displaySkin{...}}` map over all
/// skins (default server), for the profile Stats tab's skin count + browser.
/// Replaces the full `/static/skins` fetch on that tab.
#[utoipa::path(
    get,
    path = "/skins/index",
    tag = "gamedata",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The skin index. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn skins_index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let body = get_skins_index(&state, state.default_server).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/skins/index` - per-server variant.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/skins/index",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The skin index. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn skins_index_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server): Path<Server>,
) -> Result<Response, ApiError> {
    let body = get_skins_index(&state, server).await?;
    Ok(json_response(body, &headers))
}

/// The skins a player owns.
/// Runs the shared privacy gate: another player's data is readable only when
/// their profile is public, and a player always sees their own.
#[utoipa::path(
    get,
    path = "/user-skins",
    tag = "player",
    params(
        ("uid" = Option<String>, Query, description = "Player to read. Omitted means the caller's own account, which then requires a token.")
    ),
    security(("bearer_auth" = []), ()),
    responses(
        (status = 200, description = "Owned skins.", body = Vec<OwnedSkin>),
        (status = 401, response = crate::app::openapi::responses::Unauthorized),
        (status = 403, response = crate::app::openapi::responses::Forbidden),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_owned_skins(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<SkinsParams>,
) -> Result<Json<Vec<OwnedSkin>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = skins::get_owned_skins(&state.db, user_id).await?;
    Ok(Json(entries))
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinPopularityResponse {
    /// Number of users counted in the denominator - every user that has any
    /// skin record (i.e. has imported their data at least once).
    #[ts(type = "number")]
    pub total_users: i64,
    /// Map of `skin_id` -> number of owners. Only non-default skins (`skin_id`
    /// containing `@`) are included; absent IDs imply zero owners.
    #[ts(type = "Record<string, number>")]
    pub counts: HashMap<String, i64>,
    pub computed_at: String,
}

/// How many tracked players own each non-default skin.
#[utoipa::path(
    get,
    path = "/skins/popularity",
    tag = "gamedata",
    responses(
        (status = 200, description = "Owner counts by skin id.", body = SkinPopularityResponse),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_skin_popularity(
    State(state): State<AppState>,
) -> Result<Json<SkinPopularityResponse>, ApiError> {
    let key = CacheKey::SkinPopularity;
    if let Some(cached) = state.cache.get::<SkinPopularityResponse>(&key).await {
        return Ok(Json(cached));
    }

    let (total_users, rows) = skins::get_skin_popularity(&state.db).await?;
    let counts: HashMap<String, i64> = rows.into_iter().map(|r| (r.skin_id, r.owners)).collect();

    let response = SkinPopularityResponse {
        total_users,
        counts,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(Json(response))
}
