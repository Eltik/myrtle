use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use tokio_util::io::ReaderStream;

use crate::app::{error::ApiError, state::AppState};
use crate::core::gamedata::assets::AssetKind;

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "svg", "mp3", "ogg", "wav", "m4a", "mp4", "webm", "skel",
    "atlas", "json", "txt",
];

pub async fn portrait(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(char_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.portrait_path(&char_id).ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn avatar(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(avatar_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::Avatar, &avatar_id)
        .ok_or(ApiError::NotFound)?;

    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn skill_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(skill_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.skill_icon_path(&skill_id).ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn module_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(equip_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.module_icon_path(&equip_id).ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn module_big(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(equip_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.module_big_path(&equip_id).ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn enemy_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(enemy_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::EnemyIcon, &enemy_id)
        .ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn item_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(item_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::ItemIcon, &item_id)
        .ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn skin_portrait(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(skin_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::SkinPortrait, &skin_id)
        .ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, rel_path, &headers).await
}

pub async fn charart(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(char_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx.charart_path(&char_id).ok_or(ApiError::NotFound)?;
    serve_file(&state.config.assets_dir, &rel_path, &headers).await
}

pub async fn generic(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(asset_path): AxumPath<String>,
) -> Result<Response, ApiError> {
    serve_file(&state.config.assets_dir, &asset_path, &headers).await
}

fn validate_asset_path(base_dir: &Path, requested: &str) -> Result<PathBuf, ApiError> {
    if requested.contains('\0') {
        return Err(ApiError::BadRequest("invalid path".into()));
    }

    if requested.contains("..") {
        return Err(ApiError::BadRequest("invalid path".into()));
    }

    let full = base_dir.join(requested.trim_start_matches('/'));
    let canonical = full.canonicalize().map_err(|_| ApiError::NotFound)?;
    let canonical_base = base_dir
        .canonicalize()
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("assets dir missing")))?;

    if !canonical.starts_with(&canonical_base) {
        return Err(ApiError::Forbidden);
    }

    let ext = canonical.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(ApiError::Forbidden);
    }

    Ok(canonical)
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
        .map(|d| d.as_secs())
        .unwrap_or(0);

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
                (
                    header::CACHE_CONTROL,
                    "public, max-age=86400, stale-while-revalidate=3600".into(),
                ),
            ],
        )
            .into_response());
    }

    // MIME type from extension
    let mime = mime_guess::from_path(&full_path)
        .first_or_octet_stream()
        .to_string();

    // Stream the file
    let file = tokio::fs::File::open(&full_path)
        .await
        .map_err(|_| ApiError::NotFound)?;
    let stream = ReaderStream::new(file);
    let body = axum::body::Body::from_stream(stream);

    Ok((
        [
            (header::CONTENT_TYPE, mime),
            (header::CONTENT_LENGTH, size.to_string()),
            (header::ETAG, etag),
            (
                header::CACHE_CONTROL,
                "public, max-age=86400, stale-while-revalidate=3600".into(),
            ),
        ],
        body,
    )
        .into_response())
}
