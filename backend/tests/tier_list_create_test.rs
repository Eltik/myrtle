//! `POST /tier-lists` through the real router: `list_type` is a closed set,
//! the 10-list cap applies to `community` lists of non-staff, and `official`
//! is staff-only.
//!
//! Before `TierListType`, the cap matched only the string `"community"` and
//! the staff gate only `"official"`, so any third value (`"x"`) skipped both.
//!
//! The rejections need no database and run by default: the pool points at a
//! closed port, so a request that got past the checks would fail with a 5xx,
//! not the 4xx asserted. The cap and the staff path need rows, so they run on
//! a throwaway database built by the real migrations on the server
//! `DATABASE_URL` points at (default: the local Postgres), dropped afterwards:
//! `cargo test --test tier_list_create_test -- --ignored`

use std::collections::HashMap;
use std::panic::AssertUnwindSafe;
use std::str::FromStr;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::cache::store::CacheStore;
use backend::app::server::api_parts;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::jwt::create_token;
use backend::core::auth::permissions::GlobalRole;
use backend::core::hypergryph::constants::Server;
use backend::core::service_account::ServiceAccounts;
use backend::database::run_migrations;
use futures_util::FutureExt;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "tier-list-create-test-secret";
/// Port 1 refuses connections, so a query fails fast instead of reaching data.
const CLOSED_DB: &str = "postgres://tlcreate@127.0.0.1:1/none";

const ROLES: [GlobalRole; 5] = [
    GlobalRole::User,
    GlobalRole::TierListEditor,
    GlobalRole::TierListAdmin,
    GlobalRole::Translator,
    GlobalRole::SuperAdmin,
];

fn app(db: PgPool) -> Router {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "tier-list-create-test-service-key".into(),
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

fn closed_app() -> Router {
    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy(CLOSED_DB)
        .expect("lazy pool");
    app(db)
}

fn token(subject: Uuid, role: GlobalRole) -> String {
    create_token(
        JWT_SECRET,
        &subject.to_string(),
        "1",
        "en",
        &role.to_string(),
        1,
    )
    .expect("token")
}

async fn create(app: &Router, bearer: &str, name: &str, list_type: &str) -> StatusCode {
    let body = serde_json::json!({ "name": name, "list_type": list_type }).to_string();
    let req = Request::builder()
        .method("POST")
        .uri("/api/tier-lists")
        .header(header::AUTHORIZATION, format!("Bearer {bearer}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body))
        .expect("request");
    app.clone()
        .oneshot(req)
        .await
        .expect("router answers")
        .status()
}

#[tokio::test]
async fn an_unknown_list_type_is_refused_for_every_role() {
    let app = closed_app();
    for role in ROLES {
        let bearer = token(Uuid::new_v4(), role);
        // The values that used to skip both the cap and the staff gate.
        for bad in ["x", "Community", "OFFICIAL", ""] {
            assert_eq!(
                create(&app, &bearer, "probe", bad).await,
                StatusCode::UNPROCESSABLE_ENTITY,
                "{role} with list_type {bad:?}"
            );
        }
    }
}

#[tokio::test]
async fn official_is_refused_below_tier_list_staff() {
    let app = closed_app();
    for role in ROLES.into_iter().filter(|r| !r.is_any_admin_role()) {
        let bearer = token(Uuid::new_v4(), role);
        assert_eq!(
            create(&app, &bearer, "probe", "official").await,
            StatusCode::FORBIDDEN,
            "{role}"
        );
    }
}

fn base_options() -> PgConnectOptions {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    PgConnectOptions::from_str(&url).expect("DATABASE_URL parses")
}

/// A fresh database with every migration applied, and its name for the drop.
async fn throwaway_database() -> (PgPool, String) {
    let name = format!(
        "myrtle_tlcreate_{}",
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

async fn account(db: &PgPool, uid: &str, role: GlobalRole) -> Uuid {
    sqlx::query_scalar("INSERT INTO users (uid, server_id, role) VALUES ($1, 0, $2) RETURNING id")
        .bind(uid)
        .bind(role.to_string())
        .fetch_one(db)
        .await
        .expect("account")
}

#[tokio::test]
#[ignore = "creates and drops a throwaway database on the local Postgres"]
async fn the_cap_holds_community_at_ten_and_staff_create_freely() {
    let (db, name) = throwaway_database().await;
    // Drop the database even when an assertion fails, then fail.
    let outcome = AssertUnwindSafe(cap_and_staff(db.clone()))
        .catch_unwind()
        .await;
    drop_database(db, &name).await;
    if let Err(panic) = outcome {
        std::panic::resume_unwind(panic);
    }
}

async fn cap_and_staff(db: PgPool) {
    let app = app(db.clone());

    // A regular account: ten community lists, then 409.
    let user = token(
        account(&db, "300000001", GlobalRole::User).await,
        GlobalRole::User,
    );
    for i in 0..10 {
        assert_eq!(
            create(&app, &user, &format!("mine {i}"), "community").await,
            StatusCode::OK
        );
    }
    assert_eq!(
        create(&app, &user, "eleventh", "community").await,
        StatusCode::CONFLICT
    );
    // The bypass: before `TierListType` this was 200 and an eleventh list.
    assert_eq!(
        create(&app, &user, "eleventh", "x").await,
        StatusCode::UNPROCESSABLE_ENTITY
    );
    assert_eq!(
        create(&app, &user, "official", "official").await,
        StatusCode::FORBIDDEN
    );

    // Every tier-list staff role: past ten, and official too.
    for (i, role) in ROLES
        .into_iter()
        .filter(|r| r.is_any_admin_role())
        .enumerate()
    {
        let staff = token(account(&db, &format!("30000010{i}"), role).await, role);
        for n in 0..11 {
            assert_eq!(
                create(&app, &staff, &format!("{role} {n}"), "community").await,
                StatusCode::OK,
                "{role} list {n}"
            );
        }
        assert_eq!(
            create(&app, &staff, &format!("{role} site"), "official").await,
            StatusCode::OK
        );
    }

    let stored: Vec<(String, i64)> =
        sqlx::query_as("SELECT list_type, count(*) FROM tier_lists GROUP BY 1 ORDER BY 1")
            .fetch_all(&db)
            .await
            .expect("count");
    // 10 + 3 x 11 community, 3 official; nothing else reached the column.
    assert_eq!(
        stored,
        vec![("community".into(), 43), ("official".into(), 3)]
    );
}
