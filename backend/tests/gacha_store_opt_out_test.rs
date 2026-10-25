//! `user_settings.store_gacha` ("Store gacha history", Settings > Privacy) is
//! honoured on the way into `gacha_records`: by the one write path
//! (`queries::gacha::insert_batch`) and by `POST /gacha/fetch`, which refuses
//! with 409 before calling Yostar.
//!
//! Each run creates its own empty database on the server `DATABASE_URL` points
//! at (default: the local Postgres), applies every migration, runs, and drops
//! it. The router's HTTP client is pinned to a proxy on a closed port, so no
//! request can leave the machine: a fetch that reached Yostar's step answers
//! 500, one stopped by the switch answers 409. Needs a role that may create
//! databases, so it is ignored by default:
//! `cargo test --test gacha_store_opt_out_test -- --ignored`

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::str::FromStr;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::cache::keys::CacheKey;
use backend::app::cache::store::CacheStore;
use backend::app::server::api_parts;
use backend::app::services::gacha::STORAGE_OFF_MESSAGE;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::jwt::create_token;
use backend::core::hypergryph::constants::Server;
use backend::core::service_account::ServiceAccounts;
use backend::database::queries::gacha::{BatchOutcome, insert_batch};
use backend::database::run_migrations;
use futures_util::FutureExt;
use serde_json::json;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "gacha-store-opt-out-test-secret";

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_gacha_{}",
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

/// An account; `store` None leaves it without a `user_settings` row.
async fn account(db: &PgPool, uid: &str, store: Option<bool>) -> Uuid {
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
            .bind(uid)
            .fetch_one(db)
            .await
            .expect("account");
    if let Some(on) = store {
        sqlx::query("INSERT INTO user_settings (user_id, store_gacha) VALUES ($1, $2)")
            .bind(id)
            .bind(on)
            .execute(db)
            .await
            .expect("settings");
    }
    id
}

async fn stored_pulls(db: &PgPool, user: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM gacha_records WHERE user_id = $1")
        .bind(user)
        .fetch_one(db)
        .await
        .expect("count")
}

fn pulls(at: i64) -> serde_json::Value {
    json!([
        { "char_id": "char_002_amiya", "pool_id": "NORM_0_0_1", "rarity": 5,
          "pull_timestamp": at, "pool_name": "Standard", "gacha_type": "normal", "batch_index": 0 },
        { "char_id": "char_285_medic2", "pool_id": "NORM_0_0_1", "rarity": 1,
          "pull_timestamp": at, "pool_name": "Standard", "gacha_type": "normal", "batch_index": 1 }
    ])
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn the_write_path_stores_nothing_for_an_opted_out_account() {
    let (db, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(write_path(db.clone()))
        .catch_unwind()
        .await;
    drop_database(db, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn write_path(db: PgPool) {
    let off = account(&db, "400000001", Some(false)).await;
    let on = account(&db, "400000002", Some(true)).await;
    let no_row = account(&db, "400000003", None).await;

    // Before the fix `insert_batch` never read the switch: 2 rows for `off`.
    let outcome = insert_batch(&db, off, &pulls(1_700_000_000)).await.unwrap();
    assert_eq!(stored_pulls(&db, off).await, 0);
    assert_eq!(outcome, BatchOutcome::StorageOff);

    let outcome = insert_batch(&db, on, &pulls(1_700_000_000)).await.unwrap();
    assert_eq!(stored_pulls(&db, on).await, 2);
    assert_eq!(outcome, BatchOutcome::Stored);

    // No settings row reads as the column default, ON: today's behaviour.
    let outcome = insert_batch(&db, no_row, &pulls(1_700_000_000))
        .await
        .unwrap();
    assert_eq!(stored_pulls(&db, no_row).await, 2);
    assert_eq!(outcome, BatchOutcome::Stored);

    // Turning the switch off keeps what was stored and stores nothing new.
    sqlx::query("UPDATE user_settings SET store_gacha = false WHERE user_id = $1")
        .bind(on)
        .execute(&db)
        .await
        .unwrap();
    let outcome = insert_batch(&db, on, &pulls(1_700_000_100)).await.unwrap();
    assert_eq!(stored_pulls(&db, on).await, 2);
    assert_eq!(outcome, BatchOutcome::StorageOff);
}

fn state(db: PgPool) -> AppState {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "gacha-store-opt-out-test-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: false,
        storage: None,
        asset_delivery: myrtle_storage::DeliveryMode::Local,
    };
    // Every outbound request goes to a closed port: nothing reaches Yostar.
    let http = reqwest::Client::builder()
        .proxy(reqwest::Proxy::all("http://127.0.0.1:1").expect("proxy"))
        .build()
        .expect("client");
    AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::new(),
        Server::EN,
        config,
        http,
        ServiceAccounts::default(),
    )
}

/// POST /gacha/fetch as `user`/`uid` with a cached portal session, so the
/// handler would go straight on to Yostar if nothing stopped it.
async fn fetch(state: &AppState, user: Uuid, uid: &str) -> (StatusCode, serde_json::Value) {
    state
        .cache
        .set(
            &CacheKey::PortalSession { uid },
            &r#"{"yssid":"x","yssid_sig":"y"}"#.to_owned(),
        )
        .await;
    let app: Router = api_parts().0.with_state(state.clone());
    let token = create_token(JWT_SECRET, &user.to_string(), uid, "en", "user", 1).expect("token");
    let req = Request::builder()
        .method("POST")
        .uri("/api/gacha/fetch")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from("{}"))
        .expect("request");
    let res = app.oneshot(req).await.expect("router answers");
    let status = res.status();
    let body = axum::body::to_bytes(res.into_body(), 1 << 16)
        .await
        .expect("body");
    (
        status,
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn fetch_refuses_an_opted_out_account_before_calling_yostar() {
    let (db, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(fetch_gate(db.clone()))
        .catch_unwind()
        .await;
    drop_database(db, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn fetch_gate(db: PgPool) {
    let state = state(db.clone());

    let off = account(&db, "400000011", Some(false)).await;
    let (status, body) = fetch(&state, off, "400000011").await;
    // Before the fix this went on to Yostar and failed on the closed proxy: 500.
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "CONFLICT");
    assert_eq!(body["error"]["message"], STORAGE_OFF_MESSAGE);

    // With the switch on (or never set) the handler does go on to Yostar,
    // which the closed proxy turns into a 500: the gate is not over-eager.
    let on = account(&db, "400000012", Some(true)).await;
    assert_eq!(
        fetch(&state, on, "400000012").await.0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
    let no_row = account(&db, "400000013", None).await;
    assert_eq!(
        fetch(&state, no_row, "400000013").await.0,
        StatusCode::INTERNAL_SERVER_ERROR
    );
}
