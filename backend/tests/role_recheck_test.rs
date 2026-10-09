//! Role gates act on the lower of the token's role claim and the account's
//! stored role (`RoleCheckedUser`), and `DISABLE_ROLE_RECHECK` restores gates
//! that read the claim alone.
//!
//! The first two tests need no database. The pool points at a closed port, so
//! a request that reads the stored role fails with a 5xx, while one the gate
//! answers from the claim alone gets its 403 untouched. That shows which
//! requests pay the read and that the switch removes it.
//!
//! The last test reads the local Postgres (no writes) and is ignored by default:
//! `cargo test --test role_recheck_test -- --ignored --nocapture`

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
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
use sqlx::PgPool;
use tower::ServiceExt;

const JWT_SECRET: &str = "role-recheck-test-secret";
/// Port 1 refuses connections, so any query fails fast.
const CLOSED_DB: &str = "postgres://recheck@127.0.0.1:1/none";
/// Super-admin only (`require_super_admin`), and a read.
const SUPER_ONLY: &str = "/api/release/overrides";

fn closed_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy(CLOSED_DB)
        .expect("lazy pool")
}

fn app(db: PgPool, role_recheck: bool) -> Router {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "role-recheck-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck,
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

async fn get(app: &Router, subject: &str, role: GlobalRole) -> StatusCode {
    let token = create_token(JWT_SECRET, subject, "1", "en", &role.to_string(), 1).expect("token");
    let request = Request::builder()
        .uri(SUPER_ONLY)
        .header(header::AUTHORIZATION, format!("Bearer {token}"))
        .body(Body::empty())
        .expect("request builds");
    app.clone()
        .oneshot(request)
        .await
        .expect("the router is infallible")
        .status()
}

const SUBJECT: &str = "00000000-0000-4000-8000-000000000001";

#[tokio::test]
async fn with_the_switch_off_the_gate_reads_only_the_claim() {
    let app = app(closed_pool(), false);
    for &role in GlobalRole::all() {
        let status = get(&app, SUBJECT, role).await;
        if role == GlobalRole::SuperAdmin {
            // Admitted, then the handler's own query fails.
            assert!(status.is_server_error(), "{role}: {status}");
        } else {
            assert_eq!(status, StatusCode::FORBIDDEN, "{role}");
        }
    }
}

#[tokio::test]
async fn with_the_switch_on_only_a_claim_above_user_pays_the_read() {
    let app = app(closed_pool(), true);
    for &role in GlobalRole::all() {
        let status = get(&app, SUBJECT, role).await;
        if role == GlobalRole::User {
            // The lower of `user` and anything is `user`: no read, same 403.
            assert_eq!(status, StatusCode::FORBIDDEN, "{role}");
        } else {
            // The stored role is read before the gate; the database is
            // unreachable, so the request fails closed.
            assert!(status.is_server_error(), "{role}: {status}");
        }
    }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read src") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `AuthUser::token_role` is the claim as minted, up to 7 days stale. Outside
/// the extractor only `/auth/verify` reads it, as its fallback when the row is
/// gone; a gate reading it would reopen the window `RoleCheckedUser` closes.
#[test]
fn only_the_extractor_and_verify_read_the_token_claim() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    rust_files(&src, &mut files);
    let mut readers = Vec::new();
    for path in files {
        let text = fs::read_to_string(&path).expect("read source");
        let reads = text.matches(".token_role").count();
        if reads > 0 {
            let rel = path.strip_prefix(&src).expect("under src");
            readers.push((rel.to_string_lossy().replace('\\', "/"), reads));
        }
    }
    readers.sort();
    let allowed = ["app/extractors/auth.rs", "app/routes/auth.rs"];
    let stray: Vec<_> = readers
        .iter()
        .filter(|(file, _)| !allowed.contains(&file.as_str()))
        .collect();
    assert!(
        stray.is_empty(),
        "the token claim is read outside the extractor: {stray:?}"
    );
    assert!(
        readers.contains(&("app/routes/auth.rs".to_owned(), 1)),
        "{readers:?}"
    );
}

/// A token claiming `super_admin` for an account stored as `user` stands in for
/// a super-admin demoted after login; an id with no row for a deleted account.
#[tokio::test]
#[ignore = "reads the local Postgres"]
async fn a_demoted_or_deleted_account_loses_the_gate_on_its_next_request() {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:password@127.0.0.1:5432/postgres".into());
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .expect("local Postgres with real data");
    let plain: uuid::Uuid =
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'user' ORDER BY id LIMIT 1")
            .fetch_one(&pool)
            .await
            .expect("at least one plain account");
    let gone = uuid::Uuid::new_v4();
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM users WHERE id = $1)")
        .bind(gone)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!exists);

    let on = app(pool.clone(), true);
    let off = app(pool.clone(), false);
    let super_admin = GlobalRole::SuperAdmin;

    assert_eq!(
        get(&on, &plain.to_string(), super_admin).await,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        get(&on, &gone.to_string(), super_admin).await,
        StatusCode::UNAUTHORIZED
    );
    // Today's behaviour, restored by the switch: the claim alone admits both.
    assert_eq!(
        get(&off, &plain.to_string(), super_admin).await,
        StatusCode::OK
    );
    assert_eq!(
        get(&off, &gone.to_string(), super_admin).await,
        StatusCode::OK
    );

    // A stored super-admin still gets in, when one exists locally.
    let admin: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'super_admin' ORDER BY id LIMIT 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
    match admin {
        Some(id) => assert_eq!(get(&on, &id.to_string(), super_admin).await, StatusCode::OK),
        None => println!("no local super_admin row; the admitted case is unchecked"),
    }
}
