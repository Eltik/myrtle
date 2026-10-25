//! `POST /tier-lists/{slug}/view` through the real router against a database
//! built by the real migrations: a view with neither an account nor a session
//! id is not recorded, and the two kinds of viewer still dedupe.
//!
//! Each run creates its own empty database on the server `DATABASE_URL` points
//! at (default: the local Postgres), applies every migration, runs, and drops
//! it, so the counts are exact whatever the local database holds. Needs a role
//! that may create databases, so it is ignored by default:
//! `cargo test --test tier_list_view_test -- --ignored`

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::str::FromStr;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::cache::store::CacheStore;
use backend::app::server::api_parts;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::jwt::create_token;
use backend::core::hypergryph::constants::Server;
use backend::core::service_account::ServiceAccounts;
use backend::database::run_migrations;
use futures_util::FutureExt;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "tier-list-view-test-secret";
const SLUG: &str = "view-test-list";
/// A valid `X-Session-Id`: 64 hex characters.
const SESSION: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_tlview_{}",
        &Uuid::new_v4().simple().to_string()[..12]
    );
    let mut admin = PgConnection::connect_with(&base_options())
        .await
        .expect("connect to postgres");
    sqlx::raw_sql(&format!("CREATE DATABASE {name}"))
        .execute(&mut admin)
        .await
        .expect("create throwaway database");
    admin.close().await.ok();
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with(base_options().database(&name))
        .await
        .expect("connect to throwaway database");
    run_migrations(&pool)
        .await
        .expect("every migration applies");
    sqlx::query("INSERT INTO servers (id, code, name) VALUES (0, 'en', 'Global')")
        .execute(&pool)
        .await
        .expect("seed server");
    (pool, name)
}

async fn drop_database(pool: PgPool, name: &str) {
    pool.close().await;
    let mut admin = PgConnection::connect_with(&base_options())
        .await
        .expect("connect to postgres");
    sqlx::raw_sql(&format!("DROP DATABASE IF EXISTS {name} WITH (FORCE)"))
        .execute(&mut admin)
        .await
        .expect("drop throwaway database");
}

fn app(db: PgPool) -> Router {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "tier-list-view-test-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: false,
        storage: None,
        asset_delivery: myrtle_storage::DeliveryMode::Local,
    };
    let state = AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::new(),
        Server::EN,
        config,
        reqwest::Client::new(),
        ServiceAccounts::default(),
    );
    api_parts().0.with_state(state)
}

/// POST the view with the given credentials; returns the `unique` flag.
async fn view(app: &Router, bearer: Option<&str>, session: Option<&str>) -> bool {
    let mut req = Request::builder()
        .method("POST")
        .uri(format!("/api/tier-lists/{SLUG}/view"))
        .header(header::CONTENT_TYPE, "application/json");
    if let Some(t) = bearer {
        req = req.header(header::AUTHORIZATION, format!("Bearer {t}"));
    }
    if let Some(s) = session {
        req = req.header("x-session-id", s);
    }
    let res = app
        .clone()
        .oneshot(req.body(Body::from("{}")).expect("request"))
        .await
        .expect("router answers");
    assert_eq!(res.status(), StatusCode::OK);
    let body = axum::body::to_bytes(res.into_body(), 1 << 16)
        .await
        .expect("body");
    let json: serde_json::Value = serde_json::from_slice(&body).expect("json");
    json["unique"].as_bool().expect("unique is a bool")
}

/// (event rows, `view_count`, `unique_view_count`) for the list.
async fn counts(db: &PgPool, list: Uuid) -> (i64, i64, i64) {
    let events: i64 =
        sqlx::query_scalar("SELECT count(*) FROM tier_list_view_events WHERE tier_list_id = $1")
            .bind(list)
            .fetch_one(db)
            .await
            .expect("events");
    let (views, unique): (i64, i64) = sqlx::query_as(
        "SELECT view_count, unique_view_count FROM tier_list_stats WHERE tier_list_id = $1",
    )
    .bind(list)
    .fetch_one(db)
    .await
    .expect("stats");
    (events, views, unique)
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn a_view_with_no_account_and_no_session_is_not_recorded() {
    let (db, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(views(db.clone())).catch_unwind().await;
    drop_database(db, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn views(db: PgPool) {
    let user: Uuid = sqlx::query_scalar(
        "INSERT INTO users (uid, server_id) VALUES ('200000001', 0) RETURNING id",
    )
    .fetch_one(&db)
    .await
    .expect("user");
    let list: Uuid = sqlx::query_scalar(
        "INSERT INTO tier_lists (name, slug, list_type, created_by) \
         VALUES ('View test', $1, 'community', $2) RETURNING id",
    )
    .bind(SLUG)
    .bind(user)
    .fetch_one(&db)
    .await
    .expect("tier list");
    sqlx::query("INSERT INTO tier_list_stats (tier_list_id) VALUES ($1)")
        .bind(list)
        .execute(&db)
        .await
        .expect("stats row");
    let app = app(db.clone());

    // Before the fix each of these inserted an event and bumped both counters.
    assert!(!view(&app, None, None).await);
    assert!(!view(&app, None, None).await);
    assert_eq!(counts(&db, list).await, (0, 0, 0));

    // An anonymous session counts once per window.
    assert!(view(&app, None, Some(SESSION)).await);
    assert!(!view(&app, None, Some(SESSION)).await);
    assert_eq!(counts(&db, list).await, (1, 1, 1));

    // So does an account; its session header is ignored.
    let token =
        create_token(JWT_SECRET, &user.to_string(), "200000001", "en", "user", 1).expect("token");
    assert!(view(&app, Some(&token), Some(SESSION)).await);
    assert!(!view(&app, Some(&token), None).await);
    assert_eq!(counts(&db, list).await, (2, 2, 2));
}
