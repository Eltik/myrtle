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
///
/// A brotli-only value is decoded here for a plain client, on the calling
/// thread; [`serve_shared_json`] decodes in `cpu::run` instead.
pub(crate) fn shared_json_response(json: &SharedJson, headers: &HeaderMap) -> Response {
    if !accepts_br(headers) {
        let mut res = etag_json_response(json.identity(), json.etag.clone(), headers);
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

/// [`shared_json_response`] for a value that may hold only its brotli form
/// (the static tables, `STATIC_TABLES_BR_ONLY`): a client that will get the
/// plain body, and is not answered 304, has it decoded in `cpu::run`, off the
/// async workers. A brotli client and a revalidation never decode.
///
/// # Errors
/// `503` when the CPU pool sheds the decode.
pub(crate) async fn serve_shared_json(
    json: SharedJson,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    let revalidated = headers
        .get(header::IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|inm| inm == json.etag);
    let json = if json.br_only && !accepts_br(headers) && !revalidated {
        crate::app::cpu::run("static_identity", move || json.with_identity()).await?
    } else {
        json
    };
    Ok(shared_json_response(&json, headers))
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
    serve_shared_json(body, &headers).await
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
    serve_shared_json(body, &headers).await
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

    /// A body large enough to look like a table: about 2 MB of operator-like
    /// rows with varied numbers, so the brotli form is not trivially small.
    fn table_like() -> Vec<u8> {
        let rows: Vec<String> = (0..12_000)
            .map(|i| {
                format!(
                    r#""char_{i:04}":{{"name":"Op {i}","rarity":{},"atk":[{},{},{}],"tags":["a{}","b{}"]}}"#,
                    i % 6,
                    i * 7 % 997,
                    i * 13 % 1009,
                    i * 31 % 1013,
                    i % 17,
                    i % 23
                )
            })
            .collect();
        format!("{{{}}}", rows.join(",")).into_bytes()
    }

    /// One request through the handler and the server's compression layer:
    /// the status, the headers as sent, and the body decoded back to identity.
    async fn fetch(
        json: SharedJson,
        accept: &str,
        inm: Option<&str>,
    ) -> (StatusCode, HeaderMap, Vec<u8>) {
        use tower::ServiceExt;
        use tower_http::compression::{CompressionLayer, CompressionLevel};
        // `route_service`, not the plain form: `openapi_snapshot_test` counts
        // that spelling in this tree as an undocumented route.
        let app = axum::Router::new()
            .route_service(
                "/t",
                axum::routing::get(move |headers: HeaderMap| {
                    let json = json.clone();
                    async move { serve_shared_json(json, &headers).await }
                }),
            )
            .layer(CompressionLayer::new().quality(CompressionLevel::Precise(6)));
        let mut req = axum::http::Request::get("/t");
        if !accept.is_empty() {
            req = req.header(header::ACCEPT_ENCODING, accept);
        }
        if let Some(tag) = inm {
            req = req.header(header::IF_NONE_MATCH, tag);
        }
        let res = app
            .clone()
            .oneshot(req.body(Body::empty()).unwrap())
            .await
            .unwrap();
        let (parts, body) = res.into_parts();
        let raw = axum::body::to_bytes(body, usize::MAX).await.unwrap();
        let decoded = match parts
            .headers
            .get(header::CONTENT_ENCODING)
            .map(|v| v.to_str().unwrap().to_owned())
            .as_deref()
        {
            None => raw.to_vec(),
            Some("br") => {
                let mut out = Vec::new();
                brotli::BrotliDecompress(&mut raw.as_ref(), &mut out).unwrap();
                out
            }
            Some("gzip") => {
                // The decompression layer undoes the gzip the server layer applied.
                let svc = tower_http::decompression::Decompression::new(app);
                let mut again =
                    axum::http::Request::get("/t").header(header::ACCEPT_ENCODING, accept);
                if let Some(tag) = inm {
                    again = again.header(header::IF_NONE_MATCH, tag);
                }
                let res = svc
                    .oneshot(again.body(Body::empty()).unwrap())
                    .await
                    .unwrap();
                axum::body::to_bytes(Body::new(res.into_body()), usize::MAX)
                    .await
                    .unwrap()
                    .to_vec()
            }
            Some(other) => panic!("unexpected encoding {other}"),
        };
        (parts.status, parts.headers, decoded)
    }

    /// `STATIC_TABLES_BR_ONLY`: a brotli-only value serves the same bytes to
    /// brotli, gzip and identity clients as a value holding both forms, with
    /// the same `ETag`, `Content-Encoding` and `Vary`, and the same 304s.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn br_only_serves_identical_bodies_and_headers_for_every_encoding() {
        let _cpu = crate::app::cpu::tests::POOL.lock().await;
        let body = table_like();
        let want = crate::app::cache::strong_etag(&body);
        let both = SharedJson::encode(body.clone(), false);
        let br_only = SharedJson::encode(body.clone(), true);
        assert!(br_only.body.is_empty() && br_only.br_only);
        assert_eq!(both.etag, br_only.etag);
        assert_eq!(both.br_etag, br_only.br_etag);
        assert_eq!(both.br, br_only.br);
        assert_eq!(br_only.identity(), both.body);

        for accept in ["gzip, deflate, br, zstd", "gzip", "identity", ""] {
            let (s1, h1, b1) = fetch(both.clone(), accept, None).await;
            let (s2, h2, b2) = fetch(br_only.clone(), accept, None).await;
            assert_eq!((s1, s2), (StatusCode::OK, StatusCode::OK), "{accept}");
            assert_eq!(
                crate::app::cache::strong_etag(&b1),
                want,
                "{accept}: old arm"
            );
            assert_eq!(
                crate::app::cache::strong_etag(&b2),
                want,
                "{accept}: br-only arm"
            );
            for name in [
                header::ETAG,
                header::CONTENT_ENCODING,
                header::VARY,
                header::CACHE_CONTROL,
                header::CONTENT_TYPE,
            ] {
                assert_eq!(h1.get(&name), h2.get(&name), "{accept}: {name}");
            }
            // Revalidation by the tag each arm sent is a 304 in both.
            let tag = h1[header::ETAG].to_str().unwrap().to_owned();
            let (r1, _, _) = fetch(both.clone(), accept, Some(&tag)).await;
            let (r2, _, _) = fetch(br_only.clone(), accept, Some(&tag)).await;
            assert_eq!(
                (r1, r2),
                (StatusCode::NOT_MODIFIED, StatusCode::NOT_MODIFIED),
                "{accept}"
            );
        }
        let (_, h, _) = fetch(br_only.clone(), "gzip", None).await;
        assert_eq!(h[header::CONTENT_ENCODING], "gzip");
        let (_, h, _) = fetch(br_only, "br", None).await;
        assert_eq!(h[header::CONTENT_ENCODING], "br");
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
