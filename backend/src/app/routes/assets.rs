use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::UNIX_EPOCH;

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::hypergryph::constants::Server;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "svg", "mp3", "ogg", "wav", "m4a", "mp4", "webm", "skel",
    "atlas", "json", "txt",
];
static CANONICAL_BASES: OnceLock<RwLock<HashMap<PathBuf, PathBuf>>> = OnceLock::new();

fn canonical_base_dir(base_dir: &Path) -> Result<PathBuf, ApiError> {
    let cache = CANONICAL_BASES.get_or_init(|| RwLock::new(HashMap::new()));
    let cached = cache
        .read()
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("asset path cache poisoned")))?
        .get(base_dir)
        .cloned();
    if let Some(canonical) = cached {
        return Ok(canonical);
    }

    let canonical = base_dir
        .canonicalize()
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("assets dir missing")))?;
    cache
        .write()
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("asset path cache poisoned")))?
        .insert(base_dir.to_path_buf(), canonical.clone());
    Ok(canonical)
}

/// Serve an asset from `server`, falling back to the default server when the
/// requested server has no data for it (a missing index entry, or the server
/// isn't loaded at all). `resolve` maps an asset index to a relative path and is
/// run against each server's index in turn.
async fn serve_resolved<F>(
    state: &AppState,
    server: Server,
    headers: &HeaderMap,
    resolve: F,
) -> Result<Response, ApiError>
where
    F: Fn(&AssetIndex) -> Option<String>,
{
    if let Some(sd) = state.try_server_data(server)
        && let Some(rel) = resolve(&sd.asset_index.load())
        && let Ok(resp) = serve_file(&sd.assets_dir, &rel, headers).await
    {
        return Ok(resp);
    }
    if server != state.default_server
        && let Some(sd) = state.try_server_data(state.default_server)
        && let Some(rel) = resolve(&sd.asset_index.load())
    {
        return serve_file(&sd.assets_dir, &rel, headers).await;
    }
    Err(ApiError::NotFound)
}

async fn portrait_impl(
    state: &AppState,
    server: Server,
    char_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.portrait_path(char_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Operator portrait art.
#[utoipa::path(
    get,
    path = "/portrait/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Operator id, e.g. `char_002_amiya`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn portrait(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(char_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    portrait_impl(&state, state.default_server, &char_id, &headers).await
}

/// Operator portrait art. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/portrait/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id, e.g. `char_002_amiya`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn portrait_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, char_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    portrait_impl(&state, server, &char_id, &headers).await
}

async fn avatar_impl(
    state: &AppState,
    server: Server,
    avatar_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    // A handful of operators (e.g. Medic Amiya `char_1037_amiya3`, Closure)
    // ship only the `_2` avatar variant - no bare-id file exists. Fall back
    // to the E2 then E1 suffix so plain char-id lookups still resolve.
    serve_resolved(state, server, headers, |idx| {
        idx.path(AssetKind::Avatar, avatar_id)
            .or_else(|| idx.path(AssetKind::Avatar, &format!("{avatar_id}_2")))
            .or_else(|| idx.path(AssetKind::Avatar, &format!("{avatar_id}_1")))
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Small square operator avatar.
#[utoipa::path(
    get,
    path = "/avatar/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Operator or skin id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(avatar_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    avatar_impl(&state, state.default_server, &avatar_id, &headers).await
}

/// Small square operator avatar. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/avatar/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator or skin id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn avatar_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, avatar_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    avatar_impl(&state, server, &avatar_id, &headers).await
}

async fn skill_icon_impl(
    state: &AppState,
    server: Server,
    skill_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.skill_icon_path(skill_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Skill icon.
#[utoipa::path(
    get,
    path = "/skill-icon/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Skill icon id from `skill_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn skill_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(skill_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    skill_icon_impl(&state, state.default_server, &skill_id, &headers).await
}

/// Skill icon. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/skill-icon/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Skill icon id from `skill_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn skill_icon_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, skill_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    skill_icon_impl(&state, server, &skill_id, &headers).await
}

async fn module_icon_impl(
    state: &AppState,
    server: Server,
    equip_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.module_icon_path(equip_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Small module icon.
#[utoipa::path(
    get,
    path = "/module-icon/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Module (equip) id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn module_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(equip_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    module_icon_impl(&state, state.default_server, &equip_id, &headers).await
}

/// Small module icon. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/module-icon/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Module (equip) id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn module_icon_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, equip_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    module_icon_impl(&state, server, &equip_id, &headers).await
}

async fn module_big_impl(
    state: &AppState,
    server: Server,
    equip_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.module_big_path(equip_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Full-size module art.
#[utoipa::path(
    get,
    path = "/module-big/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Module (equip) id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn module_big(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(equip_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    module_big_impl(&state, state.default_server, &equip_id, &headers).await
}

/// Full-size module art. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/module-big/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Module (equip) id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn module_big_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, equip_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    module_big_impl(&state, server, &equip_id, &headers).await
}

async fn enemy_icon_impl(
    state: &AppState,
    server: Server,
    enemy_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.path(AssetKind::EnemyIcon, enemy_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Enemy icon.
#[utoipa::path(
    get,
    path = "/enemy-icon/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Enemy id from `enemy_database`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn enemy_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(enemy_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    enemy_icon_impl(&state, state.default_server, &enemy_id, &headers).await
}

/// Enemy icon. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/enemy-icon/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Enemy id from `enemy_database`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn enemy_icon_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, enemy_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    enemy_icon_impl(&state, server, &enemy_id, &headers).await
}

async fn item_icon_impl(
    state: &AppState,
    server: Server,
    item_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.path(AssetKind::ItemIcon, item_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Inventory item icon.
#[utoipa::path(
    get,
    path = "/item-icon/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Item id from `item_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn item_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(item_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    item_icon_impl(&state, state.default_server, &item_id, &headers).await
}

/// Inventory item icon. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/item-icon/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Item id from `item_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn item_icon_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, item_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    item_icon_impl(&state, server, &item_id, &headers).await
}

async fn medal_icon_impl(
    state: &AppState,
    server: Server,
    medal_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.path(AssetKind::MedalIcon, medal_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Medal icon.
#[utoipa::path(
    get,
    path = "/medal-icon/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Medal id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn medal_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(medal_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    medal_icon_impl(&state, state.default_server, &medal_id, &headers).await
}

/// Medal icon. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/medal-icon/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Medal id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn medal_icon_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, medal_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    medal_icon_impl(&state, server, &medal_id, &headers).await
}

async fn skin_portrait_impl(
    state: &AppState,
    server: Server,
    skin_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| {
        idx.path(AssetKind::SkinPortrait, skin_id)
            .map(std::borrow::ToOwned::to_owned)
    })
    .await
}

/// Skin portrait art.
#[utoipa::path(
    get,
    path = "/skin-portrait/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Skin id from `skin_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn skin_portrait(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(skin_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    skin_portrait_impl(&state, state.default_server, &skin_id, &headers).await
}

/// Skin portrait art. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/skin-portrait/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Skin id from `skin_table`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn skin_portrait_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, skin_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    skin_portrait_impl(&state, server, &skin_id, &headers).await
}

/// Defines one asset endpoint pair, plain and per-server, plus the `OpenAPI`
/// operation for each.
///
/// The paths are passed in rather than derived so the `#[utoipa::path]`
/// annotation and the route carry the SAME literal: `routes!()` reads the path
/// off the annotation, so a path written here is the path that gets served and
/// the path that gets documented, with no third place to keep in agreement.
macro_rules! indexed_asset_routes {
    ($impl_name:ident, $plain:ident, $srv:ident, $resolver:ident, $path:literal, $srv_path:literal, $id:literal, $what:literal) => {
        async fn $impl_name(
            state: &AppState,
            server: Server,
            id: &str,
            headers: &HeaderMap,
        ) -> Result<Response, ApiError> {
            serve_resolved(state, server, headers, |idx| {
                idx.$resolver(id).map(std::borrow::ToOwned::to_owned)
            })
            .await
        }

        #[doc = $what]
        #[utoipa::path(
            get,
            path = $path,
            tag = "assets",
            params(
                ($id = String, Path, description = "Asset id."),
                ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
                ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
            ),
            responses(
                (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
                (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
                (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
                (status = 404, response = crate::app::openapi::responses::NotFound),
                (status = 429, response = crate::app::openapi::responses::RateLimited),
                (status = 500, response = crate::app::openapi::responses::InternalError)
            )
        )]
        pub async fn $plain(
            State(state): State<AppState>,
            headers: HeaderMap,
            AxumPath(id): AxumPath<String>,
        ) -> Result<Response, ApiError> {
            $impl_name(&state, state.default_server, &id, &headers).await
        }

        #[doc = $what]
        #[utoipa::path(
            get,
            path = $srv_path,
            tag = "assets",
            params(
                ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
                ($id = String, Path, description = "Asset id."),
                ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
                ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
            ),
            responses(
                (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
                (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
                (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
                (status = 404, response = crate::app::openapi::responses::NotFound),
                (status = 429, response = crate::app::openapi::responses::RateLimited),
                (status = 500, response = crate::app::openapi::responses::InternalError)
            )
        )]
        pub async fn $srv(
            State(state): State<AppState>,
            headers: HeaderMap,
            AxumPath((server, id)): AxumPath<(Server, String)>,
        ) -> Result<Response, ApiError> {
            $impl_name(&state, server, &id, &headers).await
        }
    };
}

indexed_asset_routes!(
    banner_image_impl,
    banner_image,
    banner_image_srv,
    gacha_banner_path,
    "/banner-image/{pool_id}",
    "/{server}/banner-image/{pool_id}",
    "pool_id",
    " Gacha banner art, by gacha pool id."
);
indexed_asset_routes!(
    event_image_impl,
    event_image,
    event_image_srv,
    event_banner_path,
    "/event-image/{act_id}",
    "/{server}/event-image/{act_id}",
    "act_id",
    " Event art, by activity id."
);
indexed_asset_routes!(
    brand_kv_impl,
    brand_kv,
    brand_kv_srv,
    brand_kv_path,
    "/brand-kv/{kv_id}",
    "/{server}/brand-kv/{kv_id}",
    "kv_id",
    " Skin brand key visual."
);
indexed_asset_routes!(
    brand_logo_impl,
    brand_logo,
    brand_logo_srv,
    brand_logo_path,
    "/brand-logo/{brand_id}",
    "/{server}/brand-logo/{brand_id}",
    "brand_id",
    " Skin brand logo."
);

async fn charart_impl(
    state: &AppState,
    server: Server,
    char_id: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_resolved(state, server, headers, |idx| idx.charart_path(char_id)).await
}

/// Full character illustration.
#[utoipa::path(
    get,
    path = "/charart/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Operator id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn charart(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(char_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    charart_impl(&state, state.default_server, &char_id, &headers).await
}

/// Full character illustration. Served from `server`, falling back to the
/// default server when that server has no entry for the id.
#[utoipa::path(
    get,
    path = "/{server}/charart/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Operator id."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn charart_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, char_id)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    charart_impl(&state, server, &char_id, &headers).await
}

async fn generic_impl(
    state: &AppState,
    server: Server,
    asset_path: &str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(sd) = state.try_server_data(server)
        && let Ok(resp) = serve_file(&sd.assets_dir, asset_path, headers).await
    {
        return Ok(resp);
    }
    if server != state.default_server
        && let Some(sd) = state.try_server_data(state.default_server)
    {
        return serve_file(&sd.assets_dir, asset_path, headers).await;
    }
    Err(ApiError::NotFound)
}

/// Any asset by its path under the server's assets directory.
///
/// The path is validated against directory traversal and restricted to a
/// fixed extension allowlist; anything else is a 400.
#[utoipa::path(
    get,
    path = "/assets/{*path}",
    tag = "assets",
    params(
        ("path" = String, Path, description = "Asset path relative to the server's assets root."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn generic(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(asset_path): AxumPath<String>,
) -> Result<Response, ApiError> {
    generic_impl(&state, state.default_server, &asset_path, &headers).await
}

/// Any asset by its path under the server's assets directory.
///
/// The path is validated against directory traversal and restricted to a
/// fixed extension allowlist; anything else is a 400.
#[utoipa::path(
    get,
    path = "/{server}/assets/{*path}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("path" = String, Path, description = "Asset path relative to the server's assets root."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes."),
        ("Range" = Option<String>, Header, description = "Byte range. Honoured for the large media assets, answered with 206.")
    ),
    responses(
        (status = 200, description = "The asset bytes, with an `ETag` and `Cache-Control: public, max-age=604800`."),
        (status = 206, description = "Partial content, when the request carried a satisfiable `Range`."),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 400, response = crate::app::openapi::responses::BadRequest),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn generic_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, asset_path)): AxumPath<(Server, String)>,
) -> Result<Response, ApiError> {
    generic_impl(&state, server, &asset_path, &headers).await
}

fn validate_asset_path(base_dir: &Path, requested: &str) -> Result<PathBuf, ApiError> {
    if requested.contains('\0') {
        return Err(ApiError::BadRequest("invalid path".into()));
    }

    let requested = Path::new(requested.trim_start_matches('/'));
    for component in requested.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ApiError::BadRequest("invalid path".into()));
            }
        }
    }

    let canonical_base = canonical_base_dir(base_dir)?;
    // Assets are generated by our unpacker; the tree is assumed not to contain
    // hostile symlinks, so lexical validation plus the cached canonical base is sufficient here.
    let full = canonical_base.join(requested);

    let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(ApiError::Forbidden);
    }

    Ok(full)
}

async fn serve_file(
    assets_dir: &str,
    rel_path: &str,
    request_headers: &HeaderMap,
) -> Result<Response, ApiError> {
    let base = Path::new(assets_dir);
    let full_path = validate_asset_path(base, rel_path)?;

    let metadata = tokio::fs::metadata(&full_path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());

    let size = metadata.len();
    let etag = format!("\"{size}-{mtime}\"");

    if let Some(inm) = request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        && inm == etag
    {
        return Ok((
            StatusCode::NOT_MODIFIED,
            [
                (header::ETAG, etag),
                (header::CACHE_CONTROL, "public, max-age=604800".into()),
            ],
        )
            .into_response());
    }

    let mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    let range = request_headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| parse_range(v, size));

    let mut file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| ApiError::NotFound)?;

    if let Some((start, end)) = range {
        if start >= size {
            return Ok((
                StatusCode::RANGE_NOT_SATISFIABLE,
                [(header::CONTENT_RANGE, format!("bytes */{size}"))],
            )
                .into_response());
        }
        let len = end - start + 1;
        file.seek(std::io::SeekFrom::Start(start))
            .await
            .map_err(|_| ApiError::NotFound)?;
        let stream = ReaderStream::new(file.take(len));
        let body = axum::body::Body::from_stream(stream);

        return Ok((
            StatusCode::PARTIAL_CONTENT,
            [
                (header::CONTENT_TYPE, mime),
                (header::CONTENT_LENGTH, len.to_string()),
                (header::ACCEPT_RANGES, "bytes".into()),
                (header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}")),
                (header::ETAG, etag),
                (header::CACHE_CONTROL, "public, max-age=604800".into()),
            ],
            body,
        )
            .into_response());
    }

    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CONTENT_LENGTH, size.to_string()),
            (header::ACCEPT_RANGES, "bytes".into()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, "public, max-age=604800".into()),
        ],
        body,
    )
        .into_response())
}

/// Parse a single-range `Range: bytes=<start>-<end>` header.
/// Returns inclusive `(start, end)` byte offsets. `start` may be `>= size` when
/// the request is unsatisfiable; the caller is responsible for emitting 416.
/// Returns `None` only when the header is unparseable (in which case the
/// caller falls back to a full 200 response, per RFC 7233).
fn parse_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let spec = value.strip_prefix("bytes=")?.trim();
    if spec.contains(',') {
        return None;
    }
    let (start_s, end_s) = spec.split_once('-')?;
    let start_s = start_s.trim();
    let end_s = end_s.trim();

    if start_s.is_empty() {
        // Suffix range: last N bytes
        let n: u64 = end_s.parse().ok()?;
        if n == 0 || size == 0 {
            return Some((size, size));
        }
        let n = n.min(size);
        return Some((size - n, size - 1));
    }

    let start: u64 = start_s.parse().ok()?;
    let end = if end_s.is_empty() {
        size.saturating_sub(1)
    } else {
        end_s.parse::<u64>().ok()?.min(size.saturating_sub(1))
    };
    // Reject reversed/inverted ranges; per RFC 7233 these are unsatisfiable
    // and the server may ignore the header (we fall back to a full 200).
    if start > end {
        return None;
    }
    Some((start, end))
}
