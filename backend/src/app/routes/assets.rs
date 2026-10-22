use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::UNIX_EPOCH;

use axum::extract::{Path as AxumPath, Query, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use myrtle_storage::{DeliveryMode, GetOptions, ObjectStore, StorageConfig, StorageError, key};

use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::assets::{AssetIndex, AssetKind};
use crate::core::hypergryph::constants::Server;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "svg", "mp3", "ogg", "wav", "m4a", "mp4", "webm", "skel",
    "atlas", "json", "txt",
];
static CANONICAL_BASES: OnceLock<RwLock<HashMap<PathBuf, PathBuf>>> = OnceLock::new();

/// Top-level dirs `/assets/{*path}` serves; every path built for this route starts
/// with one. Each assets root also holds `gamedata/` (raw excel, `activity_table.json`
/// alone is 2.3 GB) and `derived/` (job sidecars), and both used to be downloadable,
/// brotli-encoded per request. Indexed routes resolve their own paths, unfiltered.
const PUBLIC_ASSET_ROOTS: &[&str] = &["textures", "spine", "audio", "video", "portraits"];

/// `ASSETS_ALLOW_ALL_DIRS=1` serves every directory again, exactly as before the
/// allowlist. Unset or any other value keeps the allowlist.
fn allow_all_dirs() -> bool {
    static ALLOW_ALL: OnceLock<bool> = OnceLock::new();
    *ALLOW_ALL.get_or_init(|| {
        parse_allow_all_dirs(std::env::var("ASSETS_ALLOW_ALL_DIRS").ok().as_deref())
    })
}

fn parse_allow_all_dirs(raw: Option<&str>) -> bool {
    match raw {
        None => false,
        Some(v) => v.trim() == "1",
    }
}

/// True when `requested` lies under a [`PUBLIC_ASSET_ROOTS`] entry. Rejects any
/// non-plain component itself rather than trusting the later traversal check.
fn is_public_asset_path(requested: &str) -> bool {
    let rel = Path::new(requested.trim_start_matches('/'));
    if !rel
        .components()
        .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
    {
        return false;
    }
    match rel.components().find(|c| !matches!(c, Component::CurDir)) {
        Some(Component::Normal(first)) => first
            .to_str()
            .is_some_and(|f| PUBLIC_ASSET_ROOTS.contains(&f)),
        _ => false,
    }
}

/// Read size per streamed chunk. The default reader stream reads 4 KiB at a time,
/// one blocking-pool round trip each, so a 4 MB image took about 1,024 of them.
/// `ASSET_STREAM_CHUNK_BYTES=4096` restores the old size exactly.
const DEFAULT_STREAM_CHUNK_BYTES: usize = 64 * 1024;

fn stream_chunk_bytes() -> usize {
    static CHUNK: OnceLock<usize> = OnceLock::new();
    *CHUNK.get_or_init(|| {
        parse_stream_chunk_bytes(std::env::var("ASSET_STREAM_CHUNK_BYTES").ok().as_deref())
    })
}

fn parse_stream_chunk_bytes(raw: Option<&str>) -> usize {
    match raw.map(|v| v.trim().parse::<usize>()) {
        Some(Ok(n)) if n > 0 => n,
        None | Some(_) => DEFAULT_STREAM_CHUNK_BYTES,
    }
}

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

/// Serves from the first server in [`AppState::asset_servers`] order (`server`,
/// the default, then CN) whose index has an entry and whose file serves.
/// `resolve` maps an index to a relative path, tried per server. The last
/// server's error is returned, else a 404.
async fn serve_resolved<F>(
    state: &AppState,
    server: Server,
    headers: &HeaderMap,
    resolve: F,
) -> Result<Response, ApiError>
where
    F: Fn(&AssetIndex) -> Option<String>,
{
    let mut last = ApiError::NotFound;
    for srv in state.asset_servers(server) {
        let Some(sd) = state.try_server_data(srv) else {
            continue;
        };
        let Some(rel) = resolve(&sd.asset_index.load()) else {
            continue;
        };
        match deliver_file(state, &sd.art_dir, &rel, headers).await {
            Ok(resp) => return Ok(resp),
            Err(e) => last = e,
        }
    }
    Err(last)
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
    // Some operators (Medic Amiya `char_1037_amiya3`, Closure) ship only the `_2`
    // avatar with no bare-id file, so fall back to the E2 then E1 suffix.
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

/// One asset endpoint pair, plain and per-server, with an `OpenAPI` operation each.
///
/// Paths are passed in so the `#[utoipa::path]` and the route share the SAME
/// literal (`routes!()` reads the path off the annotation).
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

/// `?format=` on the story sprite thumbnail routes.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StorySpriteThumbFormat {
    #[default]
    Webp,
    Png,
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct StorySpriteThumbParams {
    #[serde(default)]
    pub format: StorySpriteThumbFormat,
}

async fn story_sprite_thumb_impl(
    state: &AppState,
    server: Server,
    sprite_id: &str,
    format: StorySpriteThumbFormat,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    use crate::app::services::story::gallery::with_default_fallback;
    use crate::app::services::story_sprite_thumb::{ThumbFormat, ensure};
    let (format, etag_prefix) = match format {
        StorySpriteThumbFormat::Webp => (ThumbFormat::Webp, ""),
        // A PNG and a WebP of one id could share a size and mtime, so the
        // PNG's ETag carries its own prefix and never revalidates the WebP.
        StorySpriteThumbFormat::Png => (ThumbFormat::Png, "png-"),
    };
    for srv in with_default_fallback(state, server) {
        let Some(sd) = state.try_server_data(srv) else {
            continue;
        };
        let found = sd
            .asset_index
            .load()
            .story_sprites()
            .get(sprite_id)
            .map(|s| (s.body_path(), s.face_center));
        let Some((body, face)) = found else {
            continue;
        };
        // The index is built from the art tree, so its body path, and the
        // thumb cropped from it, live under `art_dir`.
        let rel = ensure(&sd.art_dir, sprite_id, &body, face, format).await?;
        return serve_file_tagged(&sd.art_dir, &rel, headers, etag_prefix).await;
    }
    Err(ApiError::NotFound)
}

/// A story character's head-and-shoulders thumbnail, 160 px square, lossless
/// WebP (or PNG with `format=png`), cropped from the default body on first
/// request and cached on disk.
#[utoipa::path(
    get,
    path = "/story-sprite-thumb/{id}",
    tag = "assets",
    params(
        ("id" = String, Path, description = "Story sprite id, e.g. `avg_npc_935`."),
        ("format" = Option<String>, Query, description = "`webp` (the default) or `png`. PNG carries the same pixels, for renderers that cannot decode WebP; each format is cached and tagged separately."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes.")
    ),
    responses(
        (status = 200, description = "The thumbnail, with an `ETag` and `Cache-Control: public, max-age=604800`.", content(("image/webp"), ("image/png"))),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn story_sprite_thumb(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(id): AxumPath<String>,
    Query(params): Query<StorySpriteThumbParams>,
) -> Result<Response, ApiError> {
    story_sprite_thumb_impl(&state, state.default_server, &id, params.format, &headers).await
}

/// A story character's head-and-shoulders thumbnail, from one server's extract.
#[utoipa::path(
    get,
    path = "/{server}/story-sprite-thumb/{id}",
    tag = "assets",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("id" = String, Path, description = "Story sprite id, e.g. `avg_npc_935`."),
        ("format" = Option<String>, Query, description = "`webp` (the default) or `png`. PNG carries the same pixels, for renderers that cannot decode WebP; each format is cached and tagged separately."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the bytes.")
    ),
    responses(
        (status = 200, description = "The thumbnail, with an `ETag` and `Cache-Control: public, max-age=604800`.", content(("image/webp"), ("image/png"))),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError)
    )
)]
pub async fn story_sprite_thumb_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath((server, id)): AxumPath<(Server, String)>,
    Query(params): Query<StorySpriteThumbParams>,
) -> Result<Response, ApiError> {
    story_sprite_thumb_impl(&state, server, &id, params.format, &headers).await
}

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
    // 404, not 403: a refusal must not confirm the file exists.
    if !allow_all_dirs() && !is_public_asset_path(asset_path) {
        return Err(ApiError::NotFound);
    }
    // `server`'s own tree, then the default's, then CN's: see `asset_servers`.
    let mut last = ApiError::NotFound;
    for srv in state.asset_servers(server) {
        let Some(sd) = state.try_server_data(srv) else {
            continue;
        };
        match deliver_file(state, &sd.assets_dir, asset_path, headers).await {
            Ok(resp) => return Ok(resp),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// Any asset by its path under the server's assets directory.
///
/// The path must lie under `textures`, `spine`, `audio`, `video` or `portraits`;
/// anything else, traversal included, is a 404. Other extensions are a 403.
///
/// With object storage configured (`S3_TYPE`) and `ASSET_DELIVERY=redirect`,
/// a file under the `S3_ROOTS` answers `302 Found` to its public URL in the
/// bucket instead of the bytes.
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
        (status = 302, description = "Delivery is `redirect`: `Location` is the object's public URL in the bucket, with `Cache-Control: public, max-age=3600` and no body."),
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
/// The path must lie under `textures`, `spine`, `audio`, `video` or `portraits`;
/// anything else, traversal included, is a 404. Other extensions are a 403.
///
/// With object storage configured (`S3_TYPE`) and `ASSET_DELIVERY=redirect`,
/// a file under the `S3_ROOTS` answers `302 Found` to its public URL in the
/// bucket instead of the bytes.
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
        (status = 302, description = "Delivery is `redirect`: `Location` is the object's public URL in the bucket, with `Cache-Control: public, max-age=3600` and no body."),
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
    // The unpacker writes this tree, assumed free of hostile symlinks, so lexical
    // validation plus the cached canonical base is enough.
    let full = canonical_base.join(requested);

    let ext = full.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(ApiError::Forbidden);
    }

    Ok(full)
}

/// Where a request for a mirrored asset is answered from, when not from disk.
#[derive(Debug, PartialEq, Eq)]
enum StoreTarget {
    /// `302 Found` to this public URL.
    Redirect(String),
    /// Stream this object key from the store.
    Proxy(String),
}

/// The store answer for `rel` under `base_dir`, or `None` for disk.
///
/// `None` when storage is disabled, delivery is `local`, `rel` fails the
/// lexical path rules, or `rel` is not under `S3_ROOTS`. The key's server
/// segment is `base_dir`'s last component, the directory the file was synced
/// from: a fallback that lands on `cn/` redirects to the `cn/` key, and an
/// art tree borrowed from the default server (`art_dir_for`) to that server's.
fn store_target(
    storage: Option<&StorageConfig>,
    mode: DeliveryMode,
    base_dir: &str,
    rel: &str,
) -> Option<StoreTarget> {
    let storage = storage?;
    if mode == DeliveryMode::Local {
        return None;
    }
    let rel = key::normalize_rel(rel)?;
    if !key::is_under_roots(&rel, &storage.roots) {
        return None;
    }
    let server = Path::new(base_dir).file_name()?.to_str()?;
    let object = key::object_key(&storage.prefix, server, &rel);
    match mode {
        DeliveryMode::Redirect => Some(StoreTarget::Redirect(key::public_url(
            storage.public_url.as_deref()?,
            &object,
        ))),
        DeliveryMode::Proxy => Some(StoreTarget::Proxy(object)),
        DeliveryMode::Local => None,
    }
}

/// [`serve_file`], or the object store when `rel` is mirrored there.
///
/// Every asset route answers through this. With `S3_TYPE` unset it is a
/// direct call to [`serve_file`].
pub(crate) async fn deliver_file(
    state: &AppState,
    base_dir: &str,
    rel: &str,
    request_headers: &HeaderMap,
) -> Result<Response, ApiError> {
    deliver_with(
        state.config.storage.as_ref(),
        state.config.asset_delivery,
        state.object_store.as_deref(),
        base_dir,
        rel,
        request_headers,
    )
    .await
}

async fn deliver_with(
    storage: Option<&StorageConfig>,
    mode: DeliveryMode,
    store: Option<&dyn ObjectStore>,
    base_dir: &str,
    rel: &str,
    request_headers: &HeaderMap,
) -> Result<Response, ApiError> {
    let (Some(target), Some(store)) = (store_target(storage, mode, base_dir, rel), store) else {
        return serve_file(base_dir, rel, request_headers).await;
    };
    // The bucket mirrors this tree, so the local file is the existence
    // oracle: a 404 here moves the caller on to the next server exactly as
    // `serve_file` would, and no request is spent on the store.
    let full_path = validate_asset_path(Path::new(base_dir), rel)?;
    let local_len = match tokio::fs::metadata(&full_path).await {
        Ok(m) if m.is_file() => m.len(),
        _ => return Err(ApiError::NotFound),
    };
    match target {
        // An hour, not the week the bytes get: the object keeps its own long
        // cache, and a week-cached redirect would keep browsers on the bucket
        // for a week after storage is switched off.
        StoreTarget::Redirect(url) => Ok((
            StatusCode::FOUND,
            [
                (header::LOCATION, url),
                (header::CACHE_CONTROL, "public, max-age=3600".into()),
            ],
        )
            .into_response()),
        StoreTarget::Proxy(object) => match proxy_object(store, &object, request_headers).await {
            Ok(resp) => Ok(resp),
            // The file is on disk, so disk can still answer: a store outage
            // or an object the sync has not reached yet costs bandwidth, not
            // a broken asset. Except for an empty file: it may be an evicted
            // stub (`S3_EVICT_LOCAL`) whose bytes live only in the store, and
            // serving it would answer 200 with an empty body.
            Err(e) if local_len == 0 => {
                tracing::warn!(key = %object, error = %e, "object store failed for an empty local file (an evicted stub?), not serving it");
                Err(match e {
                    StorageError::NotFound(_) => ApiError::NotFound,
                    _ => ApiError::ServiceUnavailable,
                })
            }
            Err(e) => {
                tracing::warn!(key = %object, error = %e, "object store failed, serving from disk");
                serve_file(base_dir, rel, request_headers).await
            }
        },
    }
}

/// Streams `key` from the store with the caller's `Range` and
/// `If-None-Match`, answering with the store's status and validators.
async fn proxy_object(
    store: &dyn ObjectStore,
    key: &str,
    request_headers: &HeaderMap,
) -> Result<Response, StorageError> {
    let header_str = |name: header::HeaderName| {
        request_headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned)
    };
    let got = store
        .get(
            key,
            GetOptions {
                range: header_str(header::RANGE),
                if_none_match: header_str(header::IF_NONE_MATCH),
            },
        )
        .await?;
    let status = StatusCode::from_u16(got.status)
        .map_err(|e| StorageError::Request(format!("store status {}: {e}", got.status)))?;
    let mut builder = axum::http::Response::builder()
        .status(status)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, "public, max-age=604800");
    if let Some(v) = got.etag {
        builder = builder.header(header::ETAG, v);
    }
    if let Some(v) = got.content_type {
        builder = builder.header(header::CONTENT_TYPE, v);
    }
    if let Some(v) = got.content_length {
        builder = builder.header(header::CONTENT_LENGTH, v);
    }
    if let Some(v) = got.content_range {
        builder = builder.header(header::CONTENT_RANGE, v);
    }
    let body = if matches!(got.status, 304 | 416) {
        axum::body::Body::empty()
    } else {
        axum::body::Body::from_stream(got.body)
    };
    builder
        .body(body)
        .map_err(|e| StorageError::Request(e.to_string()))
}

pub(crate) async fn serve_file(
    assets_dir: &str,
    rel_path: &str,
    request_headers: &HeaderMap,
) -> Result<Response, ApiError> {
    serve_file_tagged(assets_dir, rel_path, request_headers, "").await
}

/// [`serve_file`] with `etag_prefix` inside the `ETag`'s quotes, for two
/// files that may share a size and mtime but must never revalidate each other.
async fn serve_file_tagged(
    assets_dir: &str,
    rel_path: &str,
    request_headers: &HeaderMap,
    etag_prefix: &str,
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
    let etag = format!("\"{etag_prefix}{size}-{mtime}\"");

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

    // mime_guess 2.0.5 maps `.m4a` to `audio/m4a`, which is not a registered type; the music
    // `.m4a` fallbacks (AAC in MP4, for browsers without Vorbis) go out as `audio/mp4`.
    let mime = if full_path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("m4a"))
    {
        "audio/mp4".to_string()
    } else {
        mime_guess::from_path(&full_path)
            .first_or_octet_stream()
            .to_string()
    };

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
        let stream = ReaderStream::with_capacity(file.take(len), stream_chunk_bytes());
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

    let stream = ReaderStream::with_capacity(file, stream_chunk_bytes());
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

/// Parses a single-range `Range: bytes=<start>-<end>` into inclusive `(start, end)`.
/// `start >= size` means unsatisfiable (caller emits 416); `None` means unparseable
/// (caller falls back to a full 200, per RFC 7233).
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
    // Reversed range: RFC 7233 lets the server ignore the header.
    if start > end {
        return None;
    }
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spine_and_texture_paths_are_public() {
        assert!(is_public_asset_path(
            "/spine/char_002_amiya/char_002_amiya/char_002_amiya.skel"
        ));
        assert!(is_public_asset_path("spine/x/y[scene].json"));
        assert!(is_public_asset_path("./textures/chararts/a/a_2.png"));
        assert!(is_public_asset_path("/audio/audio/sound_beta_2/x.ogg"));
        assert!(is_public_asset_path(
            "/audio/audio/sound_beta_2/music/a/m_a/m_a_loop.m4a"
        ));
        assert!(is_public_asset_path("/video/main_10/main_10_enter.mp4"));
        assert!(is_public_asset_path("/portraits/char_002_amiya_1.png"));
    }

    #[test]
    fn data_dirs_are_not_public() {
        assert!(!is_public_asset_path("/gamedata/excel/activity_table.json"));
        assert!(!is_public_asset_path(
            "gamedata/levels/obt/main/level_main_00-01.json"
        ));
        assert!(!is_public_asset_path("/derived/gacha_pool_details.json"));
        assert!(!is_public_asset_path("/Textures/x.png"));
        assert!(!is_public_asset_path("/texturesx/x.png"));
        assert!(!is_public_asset_path(""));
        assert!(!is_public_asset_path("/"));
    }

    #[test]
    fn traversal_is_rejected_by_both_checks() {
        for p in [
            "/textures/../gamedata/excel/activity_table.json",
            "../textures/x.png",
            "textures/../../etc/passwd.txt",
        ] {
            assert!(!is_public_asset_path(p), "{p}");
            assert!(
                matches!(
                    validate_asset_path(Path::new("."), p),
                    Err(ApiError::BadRequest(_))
                ),
                "{p}"
            );
        }
    }

    #[test]
    fn allow_all_switch_needs_an_explicit_one() {
        assert!(!parse_allow_all_dirs(None));
        assert!(!parse_allow_all_dirs(Some("")));
        assert!(!parse_allow_all_dirs(Some("0")));
        assert!(!parse_allow_all_dirs(Some("true")));
        assert!(parse_allow_all_dirs(Some("1")));
        assert!(parse_allow_all_dirs(Some(" 1 ")));
    }

    fn storage(public: Option<&str>) -> StorageConfig {
        let vars: Vec<(&str, &str)> = [
            ("S3_TYPE", "local"),
            ("S3_LOCAL_DIR", "/nonexistent-bucket"),
            ("S3_PREFIX", "assets"),
        ]
        .into_iter()
        .chain(public.map(|p| ("S3_PUBLIC_URL", p)))
        .collect();
        StorageConfig::from_lookup(&|k| {
            vars.iter()
                .find(|(name, _)| *name == k)
                .map(|(_, v)| (*v).to_owned())
        })
        .expect("valid storage config")
        .expect("storage enabled")
    }

    #[test]
    fn storage_disabled_never_targets_the_store() {
        for mode in [
            DeliveryMode::Local,
            DeliveryMode::Redirect,
            DeliveryMode::Proxy,
        ] {
            assert_eq!(
                store_target(None, mode, "../assets/output/en", "spine/x/[scene].json"),
                None
            );
        }
        let cfg = storage(Some("https://cdn.example"));
        assert_eq!(
            store_target(
                Some(&cfg),
                DeliveryMode::Local,
                "../assets/output/en",
                "spine/x/a.json"
            ),
            None
        );
    }

    #[test]
    fn redirect_location_is_the_encoded_public_url() {
        let cfg = storage(Some("https://cdn.example"));
        assert_eq!(
            store_target(
                Some(&cfg),
                DeliveryMode::Redirect,
                "../assets/output/en",
                "spine/DynIllust/x/[scene].json"
            ),
            Some(StoreTarget::Redirect(
                "https://cdn.example/assets/en/spine/DynIllust/x/%5Bscene%5D.json".into()
            ))
        );
        // The CN fallback tree redirects to the CN key.
        assert_eq!(
            store_target(
                Some(&cfg),
                DeliveryMode::Redirect,
                "/srv/output/cn/",
                "/textures/a b.png"
            ),
            Some(StoreTarget::Redirect(
                "https://cdn.example/assets/cn/textures/a%20b.png".into()
            ))
        );
    }

    #[test]
    fn only_rels_under_the_roots_target_the_store() {
        let cfg = storage(None);
        assert_eq!(
            store_target(Some(&cfg), DeliveryMode::Proxy, "out/en", "./audio/x.ogg"),
            Some(StoreTarget::Proxy("assets/en/audio/x.ogg".into()))
        );
        for rel in [
            "gamedata/excel/a.json",
            "derived/story/x.webp",
            "textures/../gamedata/a.json",
            "textures",
        ] {
            assert_eq!(
                store_target(Some(&cfg), DeliveryMode::Proxy, "out/en", rel),
                None,
                "{rel}"
            );
        }
    }

    struct Tree {
        _dir: tempfile::TempDir,
        base: String,
    }

    fn tree() -> Tree {
        let dir = tempfile::tempdir().expect("tempdir");
        let en = dir.path().join("en");
        std::fs::create_dir_all(en.join("textures")).expect("mkdir");
        std::fs::write(en.join("textures/a.png"), b"0123456789").expect("write");
        Tree {
            base: en.display().to_string(),
            _dir: dir,
        }
    }

    async fn parts(resp: Response) -> (StatusCode, HeaderMap, Vec<u8>) {
        let (head, body) = resp.into_parts();
        let bytes = axum::body::to_bytes(body, usize::MAX).await.expect("body");
        (head.status, head.headers, bytes.to_vec())
    }

    fn with_header(name: header::HeaderName, value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(name, value.parse().expect("header value"));
        h
    }

    #[tokio::test]
    async fn disabled_delivery_is_serve_file_byte_for_byte() {
        let t = tree();
        for headers in [HeaderMap::new(), with_header(header::RANGE, "bytes=2-4")] {
            let direct = parts(
                serve_file(&t.base, "textures/a.png", &headers)
                    .await
                    .expect("serve"),
            )
            .await;
            let delivered = parts(
                deliver_with(
                    None,
                    DeliveryMode::Local,
                    None,
                    &t.base,
                    "textures/a.png",
                    &headers,
                )
                .await
                .expect("deliver"),
            )
            .await;
            assert_eq!(direct, delivered);
        }
        assert!(matches!(
            deliver_with(
                None,
                DeliveryMode::Local,
                None,
                &t.base,
                "textures/missing.png",
                &HeaderMap::new()
            )
            .await,
            Err(ApiError::NotFound)
        ));
    }

    #[tokio::test]
    async fn redirect_answers_302_only_for_files_on_disk() {
        let t = tree();
        let cfg = storage(Some("https://cdn.example"));
        let store = myrtle_storage::providers::local::LocalStore::new("/nonexistent-bucket".into());
        let (status, headers, body) = parts(
            deliver_with(
                Some(&cfg),
                DeliveryMode::Redirect,
                Some(&store),
                &t.base,
                "textures/a.png",
                &HeaderMap::new(),
            )
            .await
            .expect("redirect"),
        )
        .await;
        assert_eq!(status, StatusCode::FOUND);
        assert_eq!(
            headers[header::LOCATION],
            "https://cdn.example/assets/en/textures/a.png"
        );
        assert_eq!(headers[header::CACHE_CONTROL], "public, max-age=3600");
        assert!(body.is_empty());
        assert!(matches!(
            deliver_with(
                Some(&cfg),
                DeliveryMode::Redirect,
                Some(&store),
                &t.base,
                "textures/b.png",
                &HeaderMap::new()
            )
            .await,
            Err(ApiError::NotFound)
        ));
        assert!(matches!(
            deliver_with(
                Some(&cfg),
                DeliveryMode::Redirect,
                Some(&store),
                &t.base,
                "textures/a.exe",
                &HeaderMap::new()
            )
            .await,
            Err(ApiError::Forbidden)
        ));
    }

    #[tokio::test]
    async fn proxy_streams_the_object_and_falls_back_to_disk() {
        use myrtle_storage::{PutBody, PutObject};
        let t = tree();
        let bucket = tempfile::tempdir().expect("tempdir");
        let store = myrtle_storage::providers::local::LocalStore::new(bucket.path().to_path_buf());
        store
            .put(
                "assets/en/textures/a.png",
                PutObject {
                    body: PutBody::Bytes(bytes_of(b"0123456789")),
                    content_type: "image/png".into(),
                    cache_control: None,
                    content_md5: None,
                },
            )
            .await
            .expect("put");
        let cfg = storage(None);
        let deliver = |headers: HeaderMap, rel: &'static str| {
            let (cfg, store, base) = (&cfg, &store, t.base.clone());
            async move {
                parts(
                    deliver_with(
                        Some(cfg),
                        DeliveryMode::Proxy,
                        Some(store),
                        &base,
                        rel,
                        &headers,
                    )
                    .await
                    .expect("proxy"),
                )
                .await
            }
        };

        let (status, headers, body) = deliver(HeaderMap::new(), "textures/a.png").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"0123456789");
        assert_eq!(
            headers[header::ETAG],
            "\"781e5e245d69b566979b86e28d23f2c7\""
        );
        assert_eq!(headers[header::CONTENT_TYPE], "image/png");
        assert_eq!(headers[header::CONTENT_LENGTH], "10");
        assert_eq!(headers[header::ACCEPT_RANGES], "bytes");
        assert_eq!(headers[header::CACHE_CONTROL], "public, max-age=604800");

        let (status, headers, body) =
            deliver(with_header(header::RANGE, "bytes=2-4"), "textures/a.png").await;
        assert_eq!(status, StatusCode::PARTIAL_CONTENT);
        assert_eq!(headers[header::CONTENT_RANGE], "bytes 2-4/10");
        assert_eq!(body, b"234");

        let (status, _, body) = deliver(
            with_header(
                header::IF_NONE_MATCH,
                "\"781e5e245d69b566979b86e28d23f2c7\"",
            ),
            "textures/a.png",
        )
        .await;
        assert_eq!(status, StatusCode::NOT_MODIFIED);
        assert!(body.is_empty());

        // On disk but not yet synced: disk answers, with its own validator.
        std::fs::write(Path::new(&t.base).join("textures/new.png"), b"fresh").expect("write");
        let (status, headers, body) = deliver(HeaderMap::new(), "textures/new.png").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"fresh");
        assert!(
            headers[header::ETAG]
                .to_str()
                .expect("etag")
                .starts_with("\"5-")
        );
    }

    #[tokio::test]
    async fn proxy_never_serves_an_empty_stub_when_the_store_fails() {
        use myrtle_storage::{PutBody, PutObject};
        let t = tree();
        // An evicted stub: on disk with 0 bytes, its bytes in the store.
        std::fs::write(Path::new(&t.base).join("textures/a.png"), b"").expect("truncate");
        let bucket = tempfile::tempdir().expect("tempdir");
        let store = myrtle_storage::providers::local::LocalStore::new(bucket.path().to_path_buf());
        let cfg = storage(None);
        let no_headers = HeaderMap::new();
        let deliver = || {
            deliver_with(
                Some(&cfg),
                DeliveryMode::Proxy,
                Some(&store),
                &t.base,
                "textures/a.png",
                &no_headers,
            )
        };

        // The object is gone: 404, not a 200 with an empty body.
        assert!(matches!(deliver().await, Err(ApiError::NotFound)));

        // With the object present the stub is invisible.
        store
            .put(
                "assets/en/textures/a.png",
                PutObject {
                    body: PutBody::Bytes(bytes_of(b"0123456789")),
                    content_type: "image/png".into(),
                    cache_control: None,
                    content_md5: None,
                },
            )
            .await
            .expect("put");
        let (status, _, body) = parts(deliver().await.expect("proxy")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body, b"0123456789");
    }

    fn bytes_of(b: &'static [u8]) -> axum::body::Bytes {
        axum::body::Bytes::from_static(b)
    }

    #[test]
    fn chunk_size_defaults_to_64_kib() {
        assert_eq!(parse_stream_chunk_bytes(None), 65_536);
        assert_eq!(parse_stream_chunk_bytes(Some("")), 65_536);
        assert_eq!(parse_stream_chunk_bytes(Some("0")), 65_536);
        assert_eq!(parse_stream_chunk_bytes(Some("4096")), 4096);
    }
}
