//! The Trevor routes as the server wires them, on the paths that end before
//! the database: input validation, the hourly allowance, the service-key gate
//! on the worker routes, and the heartbeat. The pool is lazy and points at a
//! closed port, so a case that reached Postgres would fail rather than touch a
//! real database; the queue itself (SKIP LOCKED claim, result write, publish)
//! needs the v035 tables and is not exercised here.
//!
//! `cargo test --test trevor_handler_test`

use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use backend::app::cache::keys::CacheKey;
use backend::app::cache::store::CacheStore;
use backend::app::extractors::auth::AuthUser;
use backend::app::routes::trevor::{self, ServerQuery};
use backend::app::services::trevor::{AskRequest, FeedbackRequest, worker};
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::permissions::GlobalRole;
use backend::core::hypergryph::constants::Server;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

fn state() -> AppState {
    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
        .expect("lazy pool");
    let config = AppConfig {
        jwt_secret: "test-jwt-secret-not-used-by-this-test".into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "test-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
    };
    AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::new(),
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    )
}

fn user(id: Uuid) -> AuthUser {
    AuthUser {
        user_id: id.to_string(),
        uid: "12345678".into(),
        server: "en".into(),
        role: GlobalRole::default(),
    }
}

fn service() -> AuthUser {
    AuthUser {
        user_id: "service".into(),
        uid: "service".into(),
        server: "internal".into(),
        role: GlobalRole::SuperAdmin,
    }
}

fn question(q: &str, server: Option<&str>) -> AskRequest {
    serde_json::from_value(json!({ "question": q, "server": server })).expect("ask body")
}

fn status<T: IntoResponse>(r: Result<T, backend::app::error::ApiError>) -> StatusCode {
    match r {
        Ok(v) => v.into_response().status(),
        Err(e) => e.into_response().status(),
    }
}

#[tokio::test]
async fn ask_refuses_an_empty_question_and_an_unknown_server() {
    let s = state();
    let me = Uuid::new_v4();
    let empty = trevor::ask(State(s.clone()), user(me), Json(question("  ?? ", None))).await;
    assert_eq!(status(empty), StatusCode::BAD_REQUEST);
    let jp = trevor::ask(
        State(s),
        user(me),
        Json(question("Who is Amiya?", Some("jp"))),
    )
    .await;
    assert_eq!(status(jp), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn ask_answers_429_past_the_hourly_allowance() {
    let s = state();
    let me = Uuid::new_v4();
    let id = me.to_string();
    let hour = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_secs()
        / 3600;
    for _ in 0..backend::app::services::trevor::ASK_PER_HOUR {
        s.cache
            .incr_window(&CacheKey::TrevorAskQuota { user_id: &id, hour })
            .await;
    }
    let over = trevor::ask(State(s), user(me), Json(question("Who is Amiya?", None))).await;
    assert_eq!(status(over), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn feedback_refuses_a_vote_outside_minus_one_to_one() {
    let body: FeedbackRequest = serde_json::from_value(json!({ "vote": 5 })).expect("body");
    let r = trevor::feedback(
        State(state()),
        user(Uuid::new_v4()),
        Path(Uuid::new_v4()),
        Json(body),
    )
    .await;
    assert_eq!(status(r), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn panels_refuse_an_overlong_story_id() {
    let r = trevor::story_panels(
        State(state()),
        Path("x".repeat(300)),
        Query(ServerQuery { server: None }),
        HeaderMap::new(),
    )
    .await;
    assert_eq!(status(r), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn worker_routes_refuse_a_user_token() {
    let s = state();
    let hello = || -> worker::WorkerHello {
        serde_json::from_value(json!({ "workerId": "w1" })).expect("hello")
    };
    let me = || user(Uuid::new_v4());
    assert_eq!(
        status(trevor::worker_next(State(s.clone()), me(), Json(hello())).await),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        status(trevor::worker_heartbeat(State(s.clone()), me(), Json(hello())).await),
        StatusCode::FORBIDDEN
    );
    let result: worker::WorkerResult = serde_json::from_value(
        json!({ "jobId": Uuid::new_v4(), "workerId": "w1", "ok": false, "error": "x" }),
    )
    .expect("result");
    assert_eq!(
        status(trevor::worker_result(State(s.clone()), me(), Json(result)).await),
        StatusCode::FORBIDDEN
    );
    let publish: worker::PublishVersion =
        serde_json::from_value(json!({ "version": "v1" })).expect("publish");
    assert_eq!(
        status(trevor::publish_version(State(s), me(), Json(publish)).await),
        StatusCode::FORBIDDEN
    );
}

#[tokio::test]
async fn heartbeat_marks_the_server_online_with_its_slots() {
    let s = state();
    assert!(
        backend::app::services::trevor::ask::heartbeat(&s, "en")
            .await
            .is_none()
    );
    // The exact body `ask --worker` sends.
    let hello: worker::WorkerHello = serde_json::from_value(
        json!({ "workerId": "trevor-1", "server": "en", "corpusVersion": "abc", "slots": 2 }),
    )
    .expect("hello");
    let r = trevor::worker_heartbeat(State(s.clone()), service(), Json(hello)).await;
    assert_eq!(status(r), StatusCode::NO_CONTENT);
    let beat = backend::app::services::trevor::ask::heartbeat(&s, "en")
        .await
        .expect("online");
    assert_eq!(beat.slots, 2);
    assert_eq!(beat.corpus_version.as_deref(), Some("abc"));
    assert!(
        backend::app::services::trevor::ask::heartbeat(&s, "cn")
            .await
            .is_none()
    );
}

#[tokio::test]
async fn publish_refuses_an_empty_version_and_an_unknown_server() {
    let s = state();
    let empty: worker::PublishVersion =
        serde_json::from_value(json!({ "version": "" })).expect("publish");
    assert_eq!(
        status(trevor::publish_version(State(s.clone()), service(), Json(empty)).await),
        StatusCode::BAD_REQUEST
    );
    let kr: worker::PublishVersion =
        serde_json::from_value(json!({ "version": "v1", "server": "kr" })).expect("publish");
    assert_eq!(
        status(trevor::publish_version(State(s), service(), Json(kr)).await),
        StatusCode::BAD_REQUEST
    );
}
