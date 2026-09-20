use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::app::cache::CachedJson;
use crate::app::services::i18n::{get_catalog, get_manifest};
use crate::app::{error::ApiError, state::AppState};

/// `Cache-Control` for a catalog whose hash is still current. The hash is in
/// the URL, so the body at this URL can never change - which is what lets it
/// be cached forever and removes any need to purge Cloudflare, where this
/// project has no purge automation at all.
const IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// A superseded hash still answers with current content, because a client
/// holding an old manifest should get something sane rather than a 404 - but
/// it must NOT be cached as immutable, or that stale URL would be pinned in an
/// edge cache forever.
const SHORT: &str = "public, max-age=30";

fn etag_response(cached: CachedJson, cache_control: &str, headers: &HeaderMap) -> Response {
    let CachedJson { body, etag } = cached;
    if let Some(inm) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        && inm == etag
    {
        return (
            StatusCode::NOT_MODIFIED,
            [
                (header::ETAG, etag),
                (header::CACHE_CONTROL, cache_control.to_owned()),
            ],
        )
            .into_response();
    }

    (
        [
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, cache_control.to_owned()),
        ],
        body,
    )
        .into_response()
}

/// `GET /i18n/manifest` - enabled locales and the current content hash of each
/// namespace. Short TTL: this is the only read that has to go stale fast,
/// because publishing a new hash here is what makes a saved edit visible.
///
/// A client reads this first, then fetches the catalogs it needs at the hashes
/// named here. Public and unauthenticated.
#[utoipa::path(
    get,
    path = "/i18n/manifest",
    tag = "i18n",
    params(
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "Locales and per-namespace hashes.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let cached = get_manifest(&state).await?;
    Ok(etag_response(cached, SHORT, &headers))
}

/// `GET /i18n/{locale}/{namespace}/{hash}` - one rendered message catalog.
///
/// `hash` comes from the manifest and exists to make the URL change when the
/// content does. Pass `latest` to fetch the current catalog without consulting
/// the manifest first (useful in development; never cached hard).
#[utoipa::path(
    get,
    path = "/i18n/{locale}/{namespace}/{hash}",
    tag = "i18n",
    params(
        ("locale" = String, Path, description = "Locale code, e.g. `en` or `ja`."),
        ("namespace" = String, Path, description = "Namespace name."),
        ("hash" = String, Path, description = "Content hash from `/i18n/manifest`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The message catalog.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((locale, namespace, hash)): Path<(String, String, String)>,
) -> Result<Response, ApiError> {
    // `-` spells the every-namespace catalog, since an empty path segment
    // cannot be expressed in a URL.
    let namespace = if namespace == "-" {
        ""
    } else {
        namespace.as_str()
    };

    let response = get_catalog(&state, &locale, namespace, &hash).await?;
    let cache_control = if response.is_current && hash != "latest" {
        IMMUTABLE
    } else {
        SHORT
    };
    Ok(etag_response(response.json, cache_control, &headers))
}
