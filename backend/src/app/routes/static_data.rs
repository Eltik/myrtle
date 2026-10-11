use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::app::cache::{CachedJson, SharedJson};
use crate::app::services::static_data::get_resource;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `Cache-Control` of every game-data body (and the level and tier-list
/// catalogue routes): the data changes only on a reload, and five minutes
/// bounds how long a browser keeps the old copy before revalidating by `ETag`.
pub(crate) const GAMEDATA_CACHE_CONTROL: &str = "public, max-age=300";

/// Cached game-data body with `ETag` + `Cache-Control` [`GAMEDATA_CACHE_CONTROL`].
/// A matching `If-None-Match` gets a 304 without touching the body.
pub(crate) fn json_response(cached: CachedJson, headers: &HeaderMap) -> Response {
    let CachedJson { body, etag } = cached;
    etag_json_response(body, etag, headers)
}

/// [`json_response`] for any body: `ETag` + [`GAMEDATA_CACHE_CONTROL`],
/// and a 304 without the body when `If-None-Match` matches.
pub(crate) fn etag_json_response(
    body: impl Into<Body>,
    etag: String,
    headers: &HeaderMap,
) -> Response {
    if let Some(inm) = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        && inm == etag
    {
        return (
            StatusCode::NOT_MODIFIED,
            [
                (header::ETAG, etag),
                (header::CACHE_CONTROL, GAMEDATA_CACHE_CONTROL.to_owned()),
            ],
        )
            .into_response();
    }

    (
        [
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (header::ETAG, etag),
            (header::CACHE_CONTROL, GAMEDATA_CACHE_CONTROL.to_owned()),
        ],
        body.into(),
    )
        .into_response()
}

/// A [`SharedJson`] with its `ETag` and [`GAMEDATA_CACHE_CONTROL`]: the
/// stored brotli form to a client that accepts `br`, the plain body otherwise,
/// and a 304 when `If-None-Match` names the form it would get.
pub(crate) fn shared_json_response(json: &SharedJson, headers: &HeaderMap) -> Response {
    if !accepts_br(headers) {
        let mut res = etag_json_response(json.body.clone(), json.etag.clone(), headers);
        res.headers_mut()
            .insert(header::VARY, HeaderValue::from_static("accept-encoding"));
        return res;
    }
    let etag = json.br_etag.clone();
    let matched = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|inm| inm == etag);
    let cache = (header::CACHE_CONTROL, GAMEDATA_CACHE_CONTROL.to_owned());
    let vary = (header::VARY, "accept-encoding".to_owned());
    if matched {
        return (
            StatusCode::NOT_MODIFIED,
            [(header::ETAG, etag), cache, vary],
        )
            .into_response();
    }
    (
        [
            (header::CONTENT_TYPE, "application/json".to_owned()),
            (header::CONTENT_ENCODING, "br".to_owned()),
            (header::ETAG, etag),
            cache,
            vary,
        ],
        json.br.clone(),
    )
        .into_response()
}

/// Whether `Accept-Encoding` lists `br` without `q=0`.
fn accepts_br(headers: &HeaderMap) -> bool {
    headers
        .get_all(header::ACCEPT_ENCODING)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .flat_map(|v| v.split(','))
        .any(|item| {
            let mut parts = item.split(';').map(str::trim);
            parts
                .next()
                .is_some_and(|coding| coding.eq_ignore_ascii_case("br"))
                && !parts.any(|p| {
                    p.strip_prefix("q=")
                        .and_then(|q| q.parse::<f32>().ok())
                        .is_some_and(|q| q <= 0.0)
                })
        })
}

/// One whole game-data table from the default (EN) server, verbatim.
///
/// The resource names are fixed; `tests/api_shape_test.rs` pins the JSON shape
/// of every one of them.
#[utoipa::path(
    get,
    path = "/static/{resource}",
    tag = "gamedata",
    params(
        ("resource" = String, Path, description = "Table name, e.g. `item_table` or `stage-index`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The table. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_static(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(resource): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, state.default_server, &resource).await?;
    Ok(shared_json_response(&body, &headers))
}

/// `GET /{server}/static/{resource}` - per-server game-data table.
/// The `/{server}` form reads that server's game data; the bare form reads the
/// default server.
#[utoipa::path(
    get,
    path = "/{server}/static/{resource}",
    tag = "gamedata",
    params(
        ("server" = String, Path, description = "Game server: `en`, `jp`, `kr`, `cn` or `tw`."),
        ("resource" = String, Path, description = "Table name, e.g. `item_table` or `stage-index`."),
        ("If-None-Match" = Option<String>, Header, description = "Echo a previous response's `ETag` to get a 304 instead of the body.")
    ),
    responses(
        (status = 200, description = "The table. Served from cache with an `ETag` and `Cache-Control: public, max-age=300`.", content_type = "application/json"),
        (status = 304, description = "The caller's `If-None-Match` matched; no body is sent."),
        (status = 404, response = crate::app::openapi::responses::NotFound),
        (status = 429, response = crate::app::openapi::responses::RateLimited),
        (status = 500, response = crate::app::openapi::responses::InternalError),
        (status = 503, response = crate::app::openapi::responses::ServiceUnavailable)
    )
)]
pub async fn get_static_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, resource)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_resource(&state, server, &resource).await?;
    Ok(shared_json_response(&body, &headers))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn headers(pairs: &[(header::HeaderName, &str)]) -> HeaderMap {
        let mut h = HeaderMap::new();
        for (k, v) in pairs {
            h.append(k, HeaderValue::from_str(v).unwrap());
        }
        h
    }

    fn sample() -> SharedJson {
        SharedJson::new(br#"{"tracks":[1,2,3],"totals":{"tracks":3}}"#.repeat(50))
    }

    #[test]
    fn br_client_gets_the_stored_brotli_form() {
        let json = sample();
        let res = shared_json_response(
            &json,
            &headers(&[(header::ACCEPT_ENCODING, "gzip, deflate, br, zstd")]),
        );
        assert_eq!(res.status(), StatusCode::OK);
        assert_eq!(res.headers()[header::CONTENT_ENCODING], "br");
        assert_eq!(res.headers()[header::ETAG], json.br_etag.as_str());
        assert_eq!(res.headers()[header::VARY], "accept-encoding");
        assert_eq!(res.headers()[header::CACHE_CONTROL], GAMEDATA_CACHE_CONTROL);
    }

    #[test]
    fn other_clients_get_the_plain_body() {
        let json = sample();
        for accept in ["", "gzip", "br;q=0, gzip", "identity"] {
            let h = if accept.is_empty() {
                HeaderMap::new()
            } else {
                headers(&[(header::ACCEPT_ENCODING, accept)])
            };
            let res = shared_json_response(&json, &h);
            assert_eq!(res.status(), StatusCode::OK, "{accept}");
            assert!(
                res.headers().get(header::CONTENT_ENCODING).is_none(),
                "{accept}"
            );
            assert_eq!(res.headers()[header::ETAG], json.etag.as_str(), "{accept}");
        }
    }

    #[test]
    fn a_matching_etag_is_a_304_per_form() {
        let json = sample();
        let br = shared_json_response(
            &json,
            &headers(&[
                (header::ACCEPT_ENCODING, "br"),
                (header::IF_NONE_MATCH, &json.br_etag),
            ]),
        );
        assert_eq!(br.status(), StatusCode::NOT_MODIFIED);
        let plain = shared_json_response(&json, &headers(&[(header::IF_NONE_MATCH, &json.etag)]));
        assert_eq!(plain.status(), StatusCode::NOT_MODIFIED);
        // The plain form's tag does not validate the brotli form.
        let cross = shared_json_response(
            &json,
            &headers(&[
                (header::ACCEPT_ENCODING, "br"),
                (header::IF_NONE_MATCH, &json.etag),
            ]),
        );
        assert_eq!(cross.status(), StatusCode::OK);
    }
}
