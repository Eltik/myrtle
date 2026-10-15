//! `POST /api/admin/i18n/permissions` answers 404 for a locale that is not in
//! the `locales` table, the way the catalog route does, instead of letting the
//! insert hit `translation_permissions_locale_fkey` (a 500). A user id with no
//! account is a 404 the same way, instead of a violation of the grant's user
//! foreign key.
//!
//! Reads the local Postgres and writes nothing (the request is refused before
//! the insert, and before the fix the insert failed), so it is ignored by
//! default:
//! `cargo test --test i18n_grant_locale_test -- --ignored --nocapture`

use std::collections::HashMap;

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
use sqlx::PgPool;
use tower::ServiceExt;

const JWT_SECRET: &str = "i18n-grant-locale-test-secret";

fn app(db: PgPool) -> Router {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "i18n-grant-locale-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        // The claim alone admits the caller, so the test needs no stored
        // super-admin; the locale check is what is under test.
        role_recheck: false,
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

async fn grant(app: &Router, subject: &str, locale: &str, user_id: uuid::Uuid) -> StatusCode {
    let token = create_token(
        JWT_SECRET,
        subject,
        "1",
        "en",
        &GlobalRole::SuperAdmin.to_string(),
        1,
    )
    .expect("token");
    let body = serde_json::json!({
        "locale": locale,
        "user_id": user_id,
        "permission": "edit",
    });
    let request = Request::builder()
        .method("POST")
        .uri("/api/admin/i18n/permissions")
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .expect("request builds");
    app.clone()
        .oneshot(request)
        .await
        .expect("the router is infallible")
        .status()
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn a_grant_on_an_unknown_locale_is_a_404_and_writes_nothing() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("local Postgres with real data");

    let unknown = "zz-not-a-locale";
    let known: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM locales WHERE code = $1)")
        .bind(unknown)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!known, "{unknown} must not be a real locale");

    let target: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("at least one account");
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM translation_permissions")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let before = count().await;

    let app = app(pool.clone());
    assert_eq!(
        grant(&app, &target.to_string(), unknown, target).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(count().await, before);
}

#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn a_grant_for_an_unknown_user_is_a_404_and_writes_nothing() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("local Postgres with real data");

    let locale: String = sqlx::query_scalar("SELECT code FROM locales ORDER BY code LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("at least one locale");
    let nobody = uuid::Uuid::new_v4();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
        .bind(nobody)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists, "{nobody} must not be an account");
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM translation_permissions")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let before = count().await;

    // The caller is a real account and a super-admin by claim (role_recheck
    // is off), so the target is the only unknown id.
    let caller: uuid::Uuid = sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("at least one account");
    let app = app(pool.clone());
    assert_eq!(
        grant(&app, &caller.to_string(), &locale, nobody).await,
        StatusCode::NOT_FOUND
    );
    assert_eq!(count().await, before);
}
