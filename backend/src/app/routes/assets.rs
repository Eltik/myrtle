use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use axum::extract::{Path as AxumPath, State};
use axum::http::{HeaderMap, header};
use axum::response::{IntoResponse, Response};
use reqwest::StatusCode;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
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
    // A handful of operators (e.g. Medic Amiya `char_1037_amiya3`, Closure)
    // ship only the `_2` avatar variant — no bare-id file exists. Fall back
    // to the E2 then E1 suffix so plain char-id lookups still resolve.
    let rel_path = idx
        .path(AssetKind::Avatar, &avatar_id)
        .or_else(|| idx.path(AssetKind::Avatar, &format!("{avatar_id}_2")))
        .or_else(|| idx.path(AssetKind::Avatar, &format!("{avatar_id}_1")))
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

pub async fn medal_icon(
    State(state): State<AppState>,
    headers: HeaderMap,
    AxumPath(medal_id): AxumPath<String>,
) -> Result<Response, ApiError> {
    let idx = state.asset_index.load();
    let rel_path = idx
        .path(AssetKind::MedalIcon, &medal_id)
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
                (header::CACHE_CONTROL, "no-cache".into()),
            ],
        )
            .into_response());
    }

    // MIME type from extension
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
                (header::CACHE_CONTROL, "no-cache".into()),
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
            (header::CACHE_CONTROL, "no-cache".into()),
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
