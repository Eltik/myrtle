#![allow(clippy::unwrap_used)]

#[path = "../../app/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use myrtle_app::AppContext;
use myrtle_app::context::Secrets;
use myrtle_app::services::accounts::jwt::create_token;
use myrtle_types::{Role, Server};
use serde_json::{Value, json};
use tower::ServiceExt;

const SERVICE_KEY: &str = "router-test-service-key";

fn with_service_key(mut ctx: AppContext) -> AppContext {
    ctx.secrets = Arc::new(Secrets {
        jwt: ctx.secrets.jwt.clone(),
        credentials: Arc::clone(&ctx.secrets.credentials),
        service_key: Some(SERVICE_KEY.into()),
        discord_alert_webhook: None,
    });
    ctx
}

async fn call(app: &Router, req: Request) -> (StatusCode, Value) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn get(path: &str) -> Request {
    Request::get(path).body(Body::empty()).unwrap()
}

#[tokio::test]
async fn the_router_builds_and_answers_health() {
    let (ctx, db) = common::context().await;
    let app = myrtle_api::router::build(ctx);

    let (status, body) = call(&app, get("/api/health")).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert_eq!(body["cache"]["backend"], "memory");
    assert_eq!(body["database"]["status"], "connected");
    assert_eq!(body["gameData"]["degraded"], false);

    let (status, _) = call(&app, get("/api/en/health")).await;
    assert_eq!(status, StatusCode::OK, "the server prefix is stripped");
    let (status, _) = call(&app, get("/api/operator-notes?server=undefined")).await;
    assert_eq!(status, StatusCode::OK, "an unknown ?server= is the default");
    let (status, _) = call(&app, get("/api/xx/health")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = call(&app, get("/health")).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    db.close().await;
}

#[tokio::test]
async fn every_response_carries_a_request_id_and_open_cors() {
    let (ctx, db) = common::context().await;
    let app = myrtle_api::router::build(ctx);

    let minted = app.clone().oneshot(get("/api/health")).await.unwrap();
    assert_eq!(minted.headers()["x-request-id"].len(), 36);
    assert_eq!(minted.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN], "*");
    let varies_on_origin = minted
        .headers()
        .get_all(header::VARY)
        .iter()
        .any(|v| v.to_str().unwrap().to_lowercase().contains("origin"));
    assert!(!varies_on_origin, "a Vary on Origin splits the edge cache");

    let echoed = app
        .clone()
        .oneshot(
            Request::get("/api/health")
                .header("x-request-id", "abc-123")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(echoed.headers()["x-request-id"], "abc-123");

    let preflight = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::OPTIONS)
                .uri("/api/operator-notes/char_002_amiya")
                .header(header::ORIGIN, "https://myrtle.moe")
                .header(header::ACCESS_CONTROL_REQUEST_METHOD, "PUT")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(preflight.status(), StatusCode::OK);
    db.close().await;
}

#[tokio::test]
async fn callers_are_resolved_from_the_token_or_the_service_key() {
    let (ctx, db) = common::context().await;
    let ctx = with_service_key(ctx);
    let user = db.user(1, Server::EN).await;
    let now = chrono::Utc::now().timestamp() as u64;
    let token = |role| create_token(common::JWT_SECRET, user, "1", Server::EN, role, now).unwrap();
    let app = myrtle_api::router::build(ctx);
    let audit = |auth: Option<(&str, String)>| {
        let mut req = Request::get("/api/operator-notes/char_002_amiya/audit");
        if let Some((name, value)) = auth {
            req = req.header(name, value);
        }
        req.body(Body::empty()).unwrap()
    };

    let (status, body) = call(&app, audit(None)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        body,
        json!({ "error": { "code": "UNAUTHORIZED", "message": "unauthorized" } })
    );

    let bad = audit(Some(("authorization", "Bearer not-a-token".into())));
    assert_eq!(call(&app, bad).await.0, StatusCode::UNAUTHORIZED);

    let plain = audit(Some((
        "authorization",
        format!("Bearer {}", token(Role::User)),
    )));
    let (status, body) = call(&app, plain).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["error"]["code"], "FORBIDDEN");

    let service = audit(Some(("x-service-key", SERVICE_KEY.into())));
    assert_eq!(call(&app, service).await, (StatusCode::OK, json!([])));

    let wrong_key = audit(Some(("x-service-key", "guess".into())));
    assert_eq!(call(&app, wrong_key).await.0, StatusCode::UNAUTHORIZED);

    let (status, body) = call(&app, get("/api/jp/operator-notes")).await;
    assert_eq!((status, body), (StatusCode::OK, json!([])), "public route");
    db.close().await;
}
