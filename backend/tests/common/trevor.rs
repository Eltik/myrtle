//! Shared fixtures of the Trevor integration tests: the throwaway database
//! `trevor_test`, an `AppState` around a pool, the `/api` router, gzip, test
//! accounts with reading progress, and two alert sinks.
//!
//! A test binary includes this file with
//! `#[path = "common/trevor.rs"] mod trevor_common;` and uses a subset of it,
//! so unused items are expected per binary.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::Router;
use axum::body::{Body, Bytes};
use axum::http::{Request, header};
use axum::routing::get;
use backend::app::cache::store::CacheStore;
use backend::app::services::trevor::alerts;
use backend::app::services::trevor::runtime::TrevorConfig;
use backend::app::state::{AppConfig, AppState, ServerData};
use backend::core::auth::credentials::CredentialKey;
use backend::core::hypergryph::constants::Server;
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

/// The local server's maintenance database: only asked whether `trevor_test` exists.
pub const ADMIN_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";
pub const TEST_DB: &str = "trevor_test";
pub const TEST_URL: &str = "postgres://postgres:password@127.0.0.1:5432/trevor_test";

/// Create `trevor_test` if absent, rebuild its schema, run the migrations.
pub async fn fresh_db() -> PgPool {
    let mut admin = PgConnection::connect(ADMIN_URL)
        .await
        .expect("local Postgres");
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(TEST_DB)
            .fetch_one(&mut admin)
            .await
            .expect("catalog");
    if !exists {
        sqlx::raw_sql("CREATE DATABASE trevor_test")
            .execute(&mut admin)
            .await
            .expect("create trevor_test");
    }
    admin.close().await.ok();

    let mut c = PgConnection::connect(TEST_URL).await.expect("trevor_test");
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut c)
        .await
        .expect("name");
    assert_eq!(name, TEST_DB, "refusing to reset any other database");
    sqlx::raw_sql("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&mut c)
        .await
        .expect("reset schema");
    c.close().await.ok();
    backend::database::init(TEST_URL)
        .await
        .expect("migrations and seed")
}

/// A test's `AppConfig`: EN only, 100 requests a minute, role recheck on.
pub fn app_config(jwt_secret: &str, service_key: &str) -> AppConfig {
    AppConfig {
        jwt_secret: jwt_secret.into(),
        game_credential_key: CredentialKey::parse(&"22".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: service_key.into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: true,
    }
}

/// An `AppState` on `db` whose Trevor config is the default with `worker_key`.
pub fn app_state(
    db: PgPool,
    server_data: HashMap<Server, Arc<ServerData>>,
    config: AppConfig,
    worker_key: &str,
) -> AppState {
    let s = AppState::new(
        db,
        CacheStore::new_memory(),
        server_data,
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    );
    s.trevor.set_config(TrevorConfig {
        worker_key: worker_key.into(),
        ..TrevorConfig::default()
    });
    s
}

/// The `/api` router as the server builds it (without the global rate-limit
/// and observe layers).
pub fn app(s: &AppState) -> Router {
    backend::app::server::api_parts().0.with_state(s.clone())
}

/// Gzip `body` with the server's own response compression, so the tests need
/// no gzip crate: a one-route router that echoes the bytes, asked with
/// `Accept-Encoding: gzip`.
pub async fn gzip(body: Vec<u8>) -> Vec<u8> {
    let body = Bytes::from(body);
    let echo = Router::new()
        .route(
            "/",
            get(move || {
                let body = body.clone();
                async move { body }
            }),
        )
        .layer(tower_http::compression::CompressionLayer::new().gzip(true));
    let resp = echo
        .oneshot(
            Request::get("/")
                .header(header::ACCEPT_ENCODING, "gzip")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        resp.headers().get(header::CONTENT_ENCODING).unwrap(),
        "gzip"
    );
    axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap()
        .to_vec()
}

/// A `users` row with a fresh uid starting with `uid_prefix`: its id and uid.
pub async fn insert_user(pool: &PgPool, uid_prefix: &str) -> (Uuid, String) {
    let uid = format!("{uid_prefix}{}", &Uuid::new_v4().simple().to_string()[..12]);
    let id = sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
        .bind(&uid)
        .fetch_one(pool)
        .await
        .expect("user");
    (id, uid)
}

/// The site reader's progress document: `read` and `unread` marks.
pub async fn set_progress(pool: &PgPool, user: Uuid, read: &[&str], unread: &[&str]) {
    let marks = |ids: &[&str]| -> Value {
        ids.iter()
            .map(|id| ((*id).to_owned(), json!(1)))
            .collect::<serde_json::Map<_, _>>()
            .into()
    };
    sqlx::query(
        "INSERT INTO user_story_progress (user_id, progress) VALUES ($1, $2) \
         ON CONFLICT (user_id) DO UPDATE SET progress = EXCLUDED.progress",
    )
    .bind(user)
    .bind(json!({ "read": marks(read), "unread": marks(unread) }))
    .execute(pool)
    .await
    .expect("progress");
}

/// An alert sink that drops every alert.
pub struct QuietSink;

impl alerts::AlertSink for QuietSink {
    fn send(&self, _: alerts::Alert) {}
}

/// The fake webhook: records every alert, sends nothing.
#[derive(Default)]
pub struct RecordingSink(Mutex<Vec<alerts::Alert>>);

impl alerts::AlertSink for RecordingSink {
    fn send(&self, alert: alerts::Alert) {
        self.0.lock().unwrap().push(alert);
    }
}

impl RecordingSink {
    /// The alerts recorded since the last call, as (kind, recovered).
    pub fn take(&self) -> Vec<(alerts::AlertKind, bool)> {
        std::mem::take(&mut *self.0.lock().unwrap())
            .into_iter()
            .map(|a| (a.kind, a.recovered))
            .collect()
    }
}
