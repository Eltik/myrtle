//! `user_settings.public_gacha` (v042) gates `GET /gacha/public-records`: a
//! visitor reads a player's pulls only while the profile is public AND the
//! owner opted in, and the owner always reads their own. `/get-user` reports
//! the switch and `/auth/update-settings` sets it.
//!
//! Each run creates its own empty database on the server `DATABASE_URL` points
//! at (default: the local Postgres), applies every migration, runs, and drops
//! it. Needs a role that may create databases, so it is ignored by default:
//! `cargo test --test gacha_public_history_test -- --ignored`

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
use backend::database::queries::gacha::{BatchOutcome, insert_batch};
use backend::database::run_migrations;
use futures_util::FutureExt;
use serde_json::{Value, json};
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "gacha-public-history-test-secret";

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_gacha_public_{}",
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

/// An account with a settings row; `public_gacha` None leaves the column to
/// its default.
async fn account(db: &PgPool, uid: &str, public_profile: bool, public_gacha: Option<bool>) -> Uuid {
    let id: Uuid =
        sqlx::query_scalar("INSERT INTO users (uid, server_id) VALUES ($1, 0) RETURNING id")
            .bind(uid)
            .fetch_one(db)
            .await
            .expect("account");
    sqlx::query("INSERT INTO user_settings (user_id, public_profile) VALUES ($1, $2)")
        .bind(id)
        .bind(public_profile)
        .execute(db)
        .await
        .expect("settings");
    if let Some(on) = public_gacha {
        sqlx::query("UPDATE user_settings SET public_gacha = $2 WHERE user_id = $1")
            .bind(id)
            .bind(on)
            .execute(db)
            .await
            .expect("public_gacha");
    }
    let outcome = insert_batch(
        db,
        id,
        &json!([
            { "char_id": "char_002_amiya", "pool_id": "NORM_0_0_1", "rarity": 5,
              "pull_timestamp": 1_700_000_000, "pool_name": "Standard", "gacha_type": "normal", "batch_index": 0 },
            { "char_id": "char_285_medic2", "pool_id": "LIMITED_EN_40_0_4", "rarity": 1,
              "pull_timestamp": 1_700_000_100, "pool_name": "Limited", "gacha_type": "limited", "batch_index": 0 }
        ]),
    )
    .await
    .expect("pulls");
    assert_eq!(outcome, BatchOutcome::Stored);
    id
}

fn state(db: PgPool) -> AppState {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 1000,
        service_key: "gacha-public-history-test-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: false,
        storage: None,
        asset_delivery: myrtle_storage::DeliveryMode::Local,
    };
    // Every outbound request goes to a closed port: nothing leaves the machine.
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

/// `method path` as `caller` (`(user, uid)`), or signed out.
async fn call(
    state: &AppState,
    method: &str,
    path: &str,
    caller: Option<(Uuid, &str)>,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let app: Router = api_parts().0.with_state(state.clone());
    let mut req = Request::builder().method(method).uri(format!("/api{path}"));
    if let Some((user, uid)) = caller {
        let token =
            create_token(JWT_SECRET, &user.to_string(), uid, "en", "user", 1).expect("token");
        req = req.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let req = req
        .header(header::CONTENT_TYPE, "application/json")
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .expect("request");
    let res = app.oneshot(req).await.expect("router answers");
    let status = res.status();
    let bytes = axum::body::to_bytes(res.into_body(), 1 << 20)
        .await
        .expect("body");
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

/// Every pool id in a `GachaRecordsDto`, sorted.
fn pools(body: &Value) -> Vec<String> {
    let mut out: Vec<String> = ["limited", "regular", "special"]
        .iter()
        .flat_map(|group| {
            body[group]["records"]
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
        .map(|r| r["poolId"].as_str().expect("poolId").to_owned())
        .collect();
    out.sort();
    out
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn public_records_honour_both_gates() {
    let (db, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(gates(db.clone())).catch_unwind().await;
    drop_database(db, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn gates(db: PgPool) {
    let state = state(db.clone());
    let visitor = account(&db, "500000009", true, None).await;
    let as_visitor = Some((visitor, "500000009"));

    // Default: a public profile whose owner never opted in shares no pulls.
    let default_off = account(&db, "500000001", true, None).await;
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000001",
        as_visitor,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000001",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    // The owner always reads their own.
    let (status, body) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000001",
        Some((default_off, "500000001")),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(pools(&body), ["LIMITED_EN_40_0_4", "NORM_0_0_1"]);
    let (_, profile) = call(&state, "GET", "/get-user?uid=500000001", as_visitor, None).await;
    assert_eq!(profile["public_gacha"], json!(false));

    // Opted in on a public profile: anyone reads it, signed in or not.
    account(&db, "500000002", true, Some(true)).await;
    let (status, body) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000002",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(pools(&body), ["LIMITED_EN_40_0_4", "NORM_0_0_1"]);
    assert!(
        !body.to_string().contains("user_id"),
        "no account id leaks: {body}"
    );
    let (_, profile) = call(&state, "GET", "/get-user?uid=500000002", None, None).await;
    assert_eq!(profile["public_gacha"], json!(true));

    // The banner filter keeps only the named banners.
    let (status, body) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000002&pools=LIMITED_EN_40_0_4",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(pools(&body), ["LIMITED_EN_40_0_4"]);
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000002&pools=LIMITED%27--",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Opted in on a private profile: the profile gate still refuses.
    account(&db, "500000003", false, Some(true)).await;
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000003",
        as_visitor,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Unknown player.
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=599999999",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // The owner turns it on from Settings; a save without the field keeps it.
    let owner = Some((default_off, "500000001"));
    let (status, _) = call(
        &state,
        "POST",
        "/auth/update-settings",
        owner,
        Some(json!({ "public_gacha": true })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &state,
        "POST",
        "/auth/update-settings",
        owner,
        Some(json!({ "share_stats": false })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = call(
        &state,
        "GET",
        "/gacha/public-records?uid=500000001",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let (_, profile) = call(&state, "GET", "/get-user?uid=500000001", None, None).await;
    assert_eq!(profile["public_gacha"], json!(true));
}
