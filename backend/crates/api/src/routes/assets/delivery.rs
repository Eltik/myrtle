use std::collections::HashMap;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, LazyLock, Mutex, PoisonError};
use std::time::UNIX_EPOCH;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderName, StatusCode, header};
use axum::response::{IntoResponse, Response};
use myrtle_app::{AppContext, AppError};
use myrtle_storage::store::content_type_for;
use myrtle_storage::{
    Delivery, DeliveryOracle, DeliveryResolver, GetOptions, ObjectStore, StorageError,
};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::error::{ApiError, ApiResult};

const ALLOWED_EXTENSIONS: &[&str] = &[
    "png", "jpg", "jpeg", "webp", "svg", "mp3", "ogg", "wav", "m4a", "mp4", "webm", "skel",
    "atlas", "json", "txt",
];
const WEEK: &str = "public, max-age=604800";
const DEFAULT_STREAM_CHUNK_BYTES: usize = 64 * 1024;

static RESOLVERS: LazyLock<Mutex<HashMap<String, Arc<DeliveryResolver>>>> =
    LazyLock::new(Mutex::default);

fn validate_asset_path(requested: &str) -> ApiResult<()> {
    if requested.contains('\0') {
        return Err(ApiError::bad_request("invalid path"));
    }
    let requested = Path::new(requested.trim_start_matches('/'));
    for component in requested.components() {
        match component {
            Component::Normal(_) | Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(ApiError::bad_request("invalid path"));
            }
        }
    }
    let ext = requested.extension().and_then(|e| e.to_str()).unwrap_or("");
    if !ALLOWED_EXTENSIONS.contains(&ext) {
        return Err(ApiError(AppError::Forbidden));
    }
    Ok(())
}

pub(super) const fn not_found() -> ApiError {
    ApiError(AppError::NotFound(None))
}

pub(super) fn chunk_bytes(ctx: &AppContext) -> usize {
    match ctx.config.assets.stream_chunk_bytes {
        0 => DEFAULT_STREAM_CHUNK_BYTES,
        n => n,
    }
}

fn resolver(ctx: &AppContext) -> ApiResult<Arc<DeliveryResolver>> {
    let dir = &ctx.config.assets.dir;
    if let Some(found) = RESOLVERS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(dir)
    {
        return Ok(Arc::clone(found));
    }
    let storage = ctx.config.object_storage().map_err(AppError::internal)?;
    let mode = ctx
        .config
        .delivery_mode(storage.as_ref())
        .map_err(AppError::internal)?;
    let oracle = DeliveryOracle::from_env(storage.as_ref()).map_err(AppError::internal)?;
    tracing::info!(
        "{}, oracle {oracle}",
        myrtle_storage::delivery_log_line(mode, storage.as_ref())
    );
    let built = Arc::new(DeliveryResolver::new(
        PathBuf::from(dir),
        storage.as_ref(),
        mode,
        oracle,
    ));
    let mut resolvers = RESOLVERS.lock().unwrap_or_else(PoisonError::into_inner);
    Ok(Arc::clone(resolvers.entry(dir.clone()).or_insert(built)))
}

pub(super) async fn deliver(
    ctx: &AppContext,
    dirs: Vec<String>,
    rel: &str,
    headers: &HeaderMap,
) -> ApiResult<Response> {
    validate_asset_path(rel)?;
    let delivery = resolver(ctx)?
        .resolve_async(dirs, rel.to_owned())
        .await
        .map_err(AppError::internal)?;
    let chunk = chunk_bytes(ctx);
    match delivery {
        Delivery::NotFound => Err(not_found()),
        Delivery::Local { path } => serve_file(&path, headers, "", chunk).await,
        Delivery::Redirect { url } => Ok((
            StatusCode::FOUND,
            [
                (header::LOCATION, url),
                (header::CACHE_CONTROL, "public, max-age=3600".into()),
            ],
        )
            .into_response()),
        Delivery::Proxy { key, path, stub } => {
            let Some(store) = ctx.store.as_deref() else {
                return serve_file(&path, headers, "", chunk).await;
            };
            match proxy_object(store, &key, headers).await {
                Ok(response) => Ok(response),
                Err(e) if stub => {
                    tracing::warn!(key = %key, error = %e, "object store failed for an empty local file (an evicted stub?), not serving it");
                    Err(match e {
                        StorageError::NotFound(_) => not_found(),
                        _ => ApiError(AppError::Unavailable),
                    })
                }
                Err(e) => {
                    tracing::warn!(key = %key, error = %e, "object store failed, serving from disk");
                    serve_file(&path, headers, "", chunk).await
                }
            }
        }
    }
}

async fn proxy_object(
    store: &dyn ObjectStore,
    key: &str,
    request_headers: &HeaderMap,
) -> Result<Response, StorageError> {
    let header_str = |name: HeaderName| {
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
    let mut builder = Response::builder()
        .status(status)
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CACHE_CONTROL, WEEK);
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
        Body::empty()
    } else {
        Body::from_stream(got.body)
    };
    builder
        .body(body)
        .map_err(|e| StorageError::Request(e.to_string()))
}

pub(super) async fn serve_file(
    path: &Path,
    request_headers: &HeaderMap,
    etag_prefix: &str,
    chunk: usize,
) -> ApiResult<Response> {
    let metadata = tokio::fs::metadata(path).await.map_err(|_| not_found())?;
    let mtime = metadata
        .modified()
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map_or(0, |d| d.as_secs());
    let size = metadata.len();
    let etag = format!("\"{etag_prefix}{size}-{mtime}\"");

    if request_headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|inm| inm == etag)
    {
        return Ok((
            StatusCode::NOT_MODIFIED,
            [
                (header::ETAG, etag),
                (header::CACHE_CONTROL, WEEK.to_owned()),
            ],
        )
            .into_response());
    }

    let mime = content_type_for(&path.to_string_lossy());
    let range = request_headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| parse_range(v, size));
    let mut file = tokio::fs::File::open(path).await.map_err(|_| not_found())?;

    let Some((start, end)) = range else {
        let body = Body::from_stream(ReaderStream::with_capacity(file, chunk));
        return Ok((
            [
                (header::CONTENT_TYPE, mime),
                (header::CONTENT_LENGTH, size.to_string()),
                (header::ACCEPT_RANGES, "bytes".into()),
                (header::ETAG, etag),
                (header::CACHE_CONTROL, WEEK.to_owned()),
            ],
            body,
        )
            .into_response());
    };
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
        .map_err(|_| not_found())?;
    let body = Body::from_stream(ReaderStream::with_capacity(file.take(len), chunk));
    Ok((
        StatusCode::PARTIAL_CONTENT,
        [
            (header::CONTENT_TYPE, mime),
            (header::CONTENT_LENGTH, len.to_string()),
            (header::ACCEPT_RANGES, "bytes".into()),
            (header::CONTENT_RANGE, format!("bytes {start}-{end}/{size}")),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, WEEK.to_owned()),
        ],
        body,
    )
        .into_response())
}

fn parse_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let spec = value.strip_prefix("bytes=")?.trim();
    if spec.contains(',') {
        return None;
    }
    let (start_s, end_s) = spec.split_once('-')?;
    let (start_s, end_s) = (start_s.trim(), end_s.trim());
    if start_s.is_empty() {
        let n: u64 = end_s.parse().ok()?;
        if n == 0 || size == 0 {
            return Some((size, size));
        }
        let n = n.min(size);
        return Some((size - n, size - 1));
    }
    let start: u64 = start_s.parse().ok()?;
    if start >= size {
        return Some((size, size));
    }
    let end = if end_s.is_empty() {
        size.saturating_sub(1)
    } else {
        end_s.parse::<u64>().ok()?.min(size.saturating_sub(1))
    };
    (start <= end).then_some((start, end))
}
