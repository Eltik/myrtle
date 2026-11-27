#![allow(clippy::unwrap_used)]

use std::net::{IpAddr, SocketAddr};

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{ConnectInfo, Request};
use axum::http::{Extensions, HeaderMap, HeaderValue, StatusCode, header};
use axum::middleware::from_fn;
use axum::response::Response;
use axum::routing::{get, post};
use myrtle_api::extract::client_ip::peer_ip;
use myrtle_api::extract::{Json, Paging};
use myrtle_api::middleware::conditional;
use myrtle_api::respond::Cached;
use myrtle_app::cache::CachedJson;
use serde_json::{Value, json};
use tower::ServiceExt;

fn app() -> Router {
    Router::new()
        .route(
            "/cached",
            get(|| async { Cached::gamedata(CachedJson::new(r#"{"a":1}"#.into())) }),
        )
        .route(
            "/page",
            get(
                |Paging(page): Paging| async move { format!("{} {}", page.limit(), page.offset()) },
            ),
        )
        .route(
            "/echo",
            post(|Json(body): Json<Value>| async move { body.to_string() }),
        )
        .layer(from_fn(conditional))
}

async fn send(req: Request) -> Response {
    app().oneshot(req).await.unwrap()
}

async fn text(response: Response) -> String {
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

#[tokio::test]
async fn a_cached_body_carries_its_etag_and_answers_304() {
    let first = send(Request::get("/cached").body(Body::empty()).unwrap()).await;
    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(first.headers()[header::CONTENT_TYPE], "application/json");
    assert_eq!(
        first.headers()[header::CACHE_CONTROL],
        "public, max-age=300"
    );
    let etag = first.headers()[header::ETAG].clone();
    assert_eq!(text(first).await, r#"{"a":1}"#);

    let listed = format!("\"other\", {}", etag.to_str().unwrap());
    for asked in [etag.to_str().unwrap(), listed.as_str(), "*"] {
        let again = send(
            Request::get("/cached")
                .header(header::IF_NONE_MATCH, asked)
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(again.status(), StatusCode::NOT_MODIFIED, "{asked}");
        assert_eq!(again.headers()[header::ETAG], etag);
        assert_eq!(
            again.headers()[header::CACHE_CONTROL],
            "public, max-age=300"
        );
        assert!(text(again).await.is_empty());
    }

    let head = send(
        Request::head("/cached")
            .header(header::IF_NONE_MATCH, etag.to_str().unwrap())
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(head.status(), StatusCode::NOT_MODIFIED);

    let stale = send(
        Request::get("/cached")
            .header(header::IF_NONE_MATCH, "\"stale\"")
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(stale.status(), StatusCode::OK);
}

#[tokio::test]
async fn paging_reads_numbers_and_takes_empty_as_absent() {
    for (query, expected) in [
        ("", "20 0"),
        ("?limit=5&offset=10", "5 10"),
        ("?limit=&offset=", "20 0"),
        ("?limit=500", "100 0"),
    ] {
        let response = send(
            Request::get(format!("/page{query}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(text(response).await, expected, "{query}");
    }
    let refused = send(Request::get("/page?limit=x").body(Body::empty()).unwrap()).await;
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
    let body: Value = serde_json::from_str(&text(refused).await).unwrap();
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
}

#[tokio::test]
async fn a_malformed_body_is_the_bad_request_envelope() {
    let post = |body: &'static str| {
        Request::post("/echo")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap()
    };
    let ok = send(post(r#"{"a":1}"#)).await;
    assert_eq!(text(ok).await, json!({"a": 1}).to_string());

    let refused = send(post("{nope")).await;
    assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
    let body: Value = serde_json::from_str(&text(refused).await).unwrap();
    assert_eq!(body["error"]["code"], "BAD_REQUEST");
    assert!(!body["error"]["message"].as_str().unwrap().is_empty());
}

#[test]
fn the_client_address_is_the_trusted_hop_or_the_socket() {
    let socket: SocketAddr = "198.51.100.9:4000".parse().unwrap();
    let mut extensions = Extensions::new();
    extensions.insert(ConnectInfo(socket));
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-forwarded-for",
        HeaderValue::from_static("203.0.113.5, 10.0.0.2, 10.0.0.3"),
    );
    let ip = |s: &str| Some(s.parse::<IpAddr>().unwrap());

    assert_eq!(peer_ip(&headers, &extensions, 0), ip("198.51.100.9"));
    assert_eq!(peer_ip(&headers, &extensions, 1), ip("10.0.0.3"));
    assert_eq!(peer_ip(&headers, &extensions, 3), ip("203.0.113.5"));
    assert_eq!(peer_ip(&headers, &extensions, 4), ip("198.51.100.9"));
    assert_eq!(
        peer_ip(&HeaderMap::new(), &extensions, 2),
        ip("198.51.100.9")
    );
    assert_eq!(peer_ip(&headers, &Extensions::new(), 0), None);
}
