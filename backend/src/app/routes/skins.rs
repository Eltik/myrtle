use std::collections::HashMap;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::Response;
use serde::{Deserialize, Serialize};

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
pub async fn skins_index(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let body = get_skins_index(&state, state.default_server).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/skins/index` - per-server variant.
pub async fn skins_index_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server): Path<Server>,
) -> Result<Response, ApiError> {
    let body = get_skins_index(&state, server).await?;
    Ok(json_response(body, &headers))
}

pub async fn get_owned_skins(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<SkinsParams>,
) -> Result<Json<Vec<OwnedSkin>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let entries = skins::get_owned_skins(&state.db, user_id).await?;
    Ok(Json(entries))
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkinPopularityResponse {
    /// Number of users counted in the denominator - every user that has any
    /// skin record (i.e. has imported their data at least once).
    pub total_users: i64,
    /// Map of `skin_id` → number of owners. Only non-default skins (`skin_id`
    /// containing `@`) are included; absent IDs imply zero owners.
    pub counts: HashMap<String, i64>,
    pub computed_at: String,
}

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
