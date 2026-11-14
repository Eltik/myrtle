//! "Sign in with myrtle.moe", end to end through the served router: a
//! developer registers an application, a player consents, the application
//! exchanges the code, reads the identity, refreshes and revokes.
//!
//! What the suite is there to hold:
//!
//! - an OAuth access token is not a site token, and a site token is not an
//!   access token: neither opens the other's routes;
//! - a code is single use, is bound to its application, its redirect URI and
//!   its PKCE challenge, and replaying one revokes what it issued;
//! - a refresh token is single use;
//! - a browser is only ever sent to a redirect URI the application registered;
//! - revoking, by the application or by the player, kills the tokens at once.
//!
//! The first three tests need no database (the pool points at a closed port):
//! they are refused before any query runs.
//!
//! The rest write to the local Postgres and delete what they wrote, the two
//! accounts and by cascade everything under them. Ignored by default:
//! `cargo test --test oauth_flow_test -- --ignored --nocapture`

use std::collections::HashMap;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode, header};
use backend::app::cache::store::CacheStore;
use backend::app::server::api_parts;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::jwt::create_token;
use backend::core::hypergryph::constants::Server;
use backend::core::service_account::ServiceAccounts;
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use reqwest::Url;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

const JWT_SECRET: &str = "oauth-flow-test-secret";
const DATABASE_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";
/// Port 1 refuses connections, so any query fails fast.
const CLOSED_DB: &str = "postgres://oauth@127.0.0.1:1/none";

const REDIRECT: &str = "https://client.example/callback";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

fn app(db: PgPool) -> Router {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "oauth-flow-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: true,
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

fn closed_pool() -> PgPool {
    sqlx::postgres::PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(300))
        .connect_lazy(CLOSED_DB)
        .expect("lazy pool")
}

/// A response's status, the headers a test reads, and its body as JSON
/// (`Null` for an empty or non-JSON body).
struct Reply {
    status: StatusCode,
    cache_control: Option<String>,
    www_authenticate: Option<String>,
    json: Value,
}

async fn send(app: &Router, request: Request<Body>) -> Reply {
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("the router is infallible");
    let header_text = |name: header::HeaderName| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let status = response.status();
    let cache_control = header_text(header::CACHE_CONTROL);
    let www_authenticate = header_text(header::WWW_AUTHENTICATE);
    let bytes = axum::body::to_bytes(response.into_body(), 1 << 20)
        .await
        .expect("body");
    Reply {
        status,
        cache_control,
        www_authenticate,
        json: serde_json::from_slice(&bytes).unwrap_or_default(),
    }
}

/// A JSON request to one of the site's own routes, as a signed-in account.
async fn site(
    app: &Router,
    method: Method,
    path: &str,
    token: Option<&str>,
    body: Option<Value>,
) -> Reply {
    let mut request = Request::builder().method(method).uri(path);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
    let body = match body {
        Some(json) => {
            request = request.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    send(app, request.body(body).expect("request builds")).await
}

fn form_encode(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

/// A form POST to a protocol endpoint, as a third party's server sends it.
async fn form(app: &Router, path: &str, fields: &[(&str, &str)]) -> Reply {
    let request = Request::builder()
        .method(Method::POST)
        .uri(path)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_encode(fields)))
        .expect("request builds");
    send(app, request).await
}

async fn userinfo(app: &Router, access_token: &str) -> Reply {
    site(
        app,
        Method::GET,
        "/api/oauth/userinfo",
        Some(access_token),
        None,
    )
    .await
}

fn site_token(user_id: Uuid) -> String {
    create_token(JWT_SECRET, &user_id.to_string(), "1", "en", "user", 1).expect("token")
}

fn challenge() -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(VERIFIER.as_bytes()))
}

/// One query parameter of a redirect URL.
fn param(url: &str, name: &str) -> Option<String> {
    Url::parse(url)
        .expect("redirect is a URL")
        .query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

// ---------------------------------------------------------------------------
// No database needed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn userinfo_without_a_token_is_the_rfc_error_not_the_site_envelope() {
    let app = app(closed_pool());
    let reply = send(
        &app,
        Request::builder()
            .uri("/api/oauth/userinfo")
            .body(Body::empty())
            .expect("request builds"),
    )
    .await;
    assert_eq!(reply.status, StatusCode::UNAUTHORIZED);
    assert_eq!(reply.json["error"], "invalid_token");
    assert!(reply.json["error_description"].is_string());
    assert_eq!(
        reply.www_authenticate.as_deref(),
        Some(r#"Bearer error="invalid_token""#)
    );
    assert_eq!(reply.cache_control.as_deref(), Some("no-store"));
}

#[tokio::test]
async fn the_token_endpoint_refuses_json_and_missing_credentials() {
    let app = app(closed_pool());

    // RFC 6749 bodies are form-encoded; JSON is a malformed request.
    let reply = site(
        &app,
        Method::POST,
        "/api/oauth/token",
        None,
        Some(json!({ "grant_type": "authorization_code" })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json["error"], "invalid_request");

    for path in ["/api/oauth/token", "/api/oauth/revoke"] {
        let reply = form(&app, path, &[("grant_type", "authorization_code")]).await;
        assert_eq!(reply.status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(reply.json["error"], "invalid_client", "{path}");
        assert_eq!(
            reply.www_authenticate.as_deref(),
            Some(r#"Basic realm="myrtle.moe""#),
            "{path}"
        );
    }
}

/// The other direction of the wall: something shaped like an access token
/// opens none of the routes a player's own session opens, and the management
/// routes want a site token.
#[tokio::test]
async fn an_access_token_is_not_a_site_token() {
    let app = app(closed_pool());
    let access_token =
        backend::core::auth::oauth::mint(backend::core::auth::oauth::SecretKind::AccessToken);
    for (method, path) in [
        (Method::GET, "/api/auth/verify"),
        (Method::POST, "/api/auth/delete-account"),
        (Method::POST, "/api/auth/update-settings"),
        (Method::GET, "/api/oauth/apps"),
        (Method::GET, "/api/oauth/authorizations"),
        (Method::GET, "/api/oauth/authorize?client_id=x"),
    ] {
        for token in [Some(access_token.as_str()), None] {
            let reply = site(&app, method.clone(), path, token, None).await;
            assert_eq!(
                reply.status,
                StatusCode::UNAUTHORIZED,
                "{method} {path} with {token:?}"
            );
            assert_eq!(reply.json["error"]["code"], "UNAUTHORIZED", "{path}");
        }
    }
}

// ---------------------------------------------------------------------------
// Against the local Postgres
// ---------------------------------------------------------------------------

/// Two accounts that exist only for one test run.
struct Accounts {
    pool: PgPool,
    developer: Uuid,
    player: Uuid,
    player_uid: String,
}

/// Migrate once per test binary. Every test has its own runtime and so its own
/// pool, but five of them migrating a fresh database at the same moment race
/// on `CREATE EXTENSION`.
static MIGRATED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

async fn pool() -> PgPool {
    MIGRATED
        .get_or_init(|| async {
            backend::database::init(DATABASE_URL)
                .await
                .expect("local Postgres at DATABASE_URL")
                .close()
                .await;
        })
        .await;
    backend::database::create_pool(DATABASE_URL)
        .await
        .expect("local Postgres at DATABASE_URL")
}

impl Accounts {
    async fn create() -> Self {
        let pool = pool().await;
        // Game uids are digits; a random 12-digit one cannot collide with a
        // real account, whose uids are 8 or 9.
        let uid = || format!("9{:011}", rand::random::<u32>());
        let developer_uid = uid();
        let player_uid = uid();
        let developer = sqlx::query_scalar(
            "INSERT INTO users (uid, server_id, nickname, nick_number)
             VALUES ($1, 0, 'Developer', '0001') RETURNING id",
        )
        .bind(&developer_uid)
        .fetch_one(&pool)
        .await
        .expect("developer");
        let player = sqlx::query_scalar(
            "INSERT INTO users (uid, server_id, nickname, nick_number, avatar_id)
             VALUES ($1, 0, 'Doctor', '1234', 'char_002_amiya@winter#1') RETURNING id",
        )
        .bind(&player_uid)
        .fetch_one(&pool)
        .await
        .expect("player");
        Self {
            pool,
            developer,
            player,
            player_uid,
        }
    }

    /// Both accounts go, and by cascade every application, grant, code and
    /// token the test made.
    async fn delete(self) {
        sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(vec![self.developer, self.player])
            .execute(&self.pool)
            .await
            .expect("cleanup");
        let left: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM oauth_apps WHERE owner_id = ANY($1))
                  + (SELECT COUNT(*) FROM oauth_grants WHERE user_id = ANY($1))",
        )
        .bind(vec![self.developer, self.player])
        .fetch_one(&self.pool)
        .await
        .expect("census");
        assert_eq!(left, 0, "deleting the accounts left OAuth rows behind");
    }
}

/// Register an application as `owner`; its id and secret.
async fn register(app: &Router, owner: Uuid, name: &str) -> (String, String) {
    let reply = site(
        app,
        Method::POST,
        "/api/oauth/apps",
        Some(&site_token(owner)),
        Some(json!({
            "name": name,
            "description": "integration test",
            "redirect_uris": [REDIRECT, "http://localhost:4000/callback"],
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.json);
    let id = reply.json["id"].as_str().expect("id").to_owned();
    let secret = reply.json["client_secret"]
        .as_str()
        .expect("secret")
        .to_owned();
    assert!(secret.starts_with("myrtle_cs_"), "{secret}");
    (id, secret)
}

/// The player approves `client_id` (with PKCE); the code on the redirect.
async fn approve(app: &Router, player: Uuid, client_id: &str) -> String {
    let reply = site(
        app,
        Method::POST,
        "/api/oauth/authorize",
        Some(&site_token(player)),
        Some(json!({
            "response_type": "code",
            "client_id": client_id,
            "redirect_uri": REDIRECT,
            "scope": "identify",
            "state": "st&te=1",
            "code_challenge": challenge(),
            "code_challenge_method": "S256",
            "approve": true,
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.json);
    let redirect_to = reply.json["redirect_to"].as_str().expect("redirect_to");
    assert!(redirect_to.starts_with(REDIRECT), "{redirect_to}");
    // `state` comes back byte for byte, reserved characters included.
    assert_eq!(param(redirect_to, "state").as_deref(), Some("st&te=1"));
    param(redirect_to, "code").expect("code on the redirect")
}

/// Exchange `code` as the application, with the right verifier.
async fn exchange(app: &Router, client_id: &str, secret: &str, code: &str) -> Reply {
    form(
        app,
        "/api/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", REDIRECT),
            ("code_verifier", VERIFIER),
            ("client_id", client_id),
            ("client_secret", secret),
        ],
    )
    .await
}

fn tokens(reply: &Reply) -> (String, String) {
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.json);
    assert_eq!(reply.json["token_type"], "Bearer");
    assert_eq!(reply.json["scope"], "identify");
    assert_eq!(reply.json["expires_in"], 604_800);
    assert_eq!(reply.cache_control.as_deref(), Some("no-store"));
    let access = reply.json["access_token"].as_str().expect("access token");
    let refresh = reply.json["refresh_token"].as_str().expect("refresh token");
    assert!(access.starts_with("myrtle_at_"), "{access}");
    assert!(refresh.starts_with("myrtle_rt_"), "{refresh}");
    (access.to_owned(), refresh.to_owned())
}

#[tokio::test]
#[ignore = "writes to the local Postgres (and cleans up after itself)"]
async fn a_player_signs_in_to_a_third_party_application() {
    let accounts = Accounts::create().await;
    let app = app(accounts.pool.clone());
    let (client_id, secret) = register(&app, accounts.developer, "Flow test").await;
    let player_token = site_token(accounts.player);

    // The consent page is told who is asking and where the player will land.
    let query = format!(
        "/api/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}&state=abc",
        urlencoding::encode(REDIRECT)
    );
    let check = site(&app, Method::GET, &query, Some(&player_token), None).await;
    assert_eq!(check.status, StatusCode::OK, "{}", check.json);
    assert_eq!(check.json["status"], "consent");
    assert_eq!(check.json["app"]["name"], "Flow test");
    assert_eq!(check.json["app"]["owner"]["nickname"], "Developer");
    assert_eq!(check.json["scopes"], json!(["identify"]));
    assert_eq!(check.json["redirect_host"], "client.example");
    assert_eq!(check.json["already_authorized"], false);
    // Nothing about the application's secret reaches a player.
    assert!(!check.json.to_string().contains("secret"));

    // Declining tells the application, and grants nothing.
    let declined = site(
        &app,
        Method::POST,
        "/api/oauth/authorize",
        Some(&player_token),
        Some(json!({
            "response_type": "code", "client_id": client_id, "redirect_uri": REDIRECT,
            "state": "abc", "approve": false,
        })),
    )
    .await;
    let redirect_to = declined.json["redirect_to"].as_str().expect("redirect_to");
    assert_eq!(
        param(redirect_to, "error").as_deref(),
        Some("access_denied")
    );
    assert_eq!(param(redirect_to, "state").as_deref(), Some("abc"));
    assert_eq!(param(redirect_to, "code"), None);

    // Approving yields a code, and the code yields tokens.
    let code = approve(&app, accounts.player, &client_id).await;
    assert!(code.starts_with("myrtle_ac_"), "{code}");
    let (access, refresh) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    // The access token reads the identity, and only the identity.
    let me = userinfo(&app, &access).await;
    assert_eq!(me.status, StatusCode::OK, "{}", me.json);
    assert_eq!(me.cache_control.as_deref(), Some("no-store"));
    assert_eq!(me.json["id"], accounts.player.to_string());
    assert_eq!(me.json["username"], "Doctor");
    assert_eq!(me.json["tag"], "1234");
    assert_eq!(me.json["uid"], accounts.player_uid);
    assert_eq!(me.json["server"], "en");
    assert_eq!(me.json["avatar_id"], "char_002_amiya@winter#1");
    assert!(
        me.json["avatar_url"]
            .as_str()
            .is_some_and(|u| u.ends_with("/api/avatar/char_002_amiya_winter%231")),
        "{}",
        me.json["avatar_url"]
    );
    let keys: Vec<&String> = me.json.as_object().expect("object").keys().collect();
    assert_eq!(
        keys,
        [
            "avatar_id",
            "avatar_url",
            "id",
            "server",
            "tag",
            "uid",
            "username"
        ]
    );

    // The wall, both ways, with real credentials this time.
    let verify = site(&app, Method::GET, "/api/auth/verify", Some(&access), None).await;
    assert_eq!(verify.status, StatusCode::UNAUTHORIZED);
    let as_site = userinfo(&app, &player_token).await;
    assert_eq!(as_site.status, StatusCode::UNAUTHORIZED);
    assert_eq!(as_site.json["error"], "invalid_token");
    // A refresh token is not an access token either.
    assert_eq!(
        userinfo(&app, &refresh).await.status,
        StatusCode::UNAUTHORIZED
    );

    // The player sees the application in their list, and the developer sees a
    // user on theirs.
    let list = site(
        &app,
        Method::GET,
        "/api/oauth/authorizations",
        Some(&player_token),
        None,
    )
    .await;
    assert_eq!(list.json[0]["app"]["id"], client_id);
    assert_eq!(list.json[0]["scopes"], json!(["identify"]));
    assert!(list.json[0]["last_used_at"].is_string());
    let apps = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&site_token(accounts.developer)),
        None,
    )
    .await;
    assert_eq!(apps.json[0]["authorized_users"], 1);
    assert!(apps.json[0].get("client_secret").is_none());
    let again = site(&app, Method::GET, &query, Some(&player_token), None).await;
    assert_eq!(again.json["already_authorized"], true);

    // Refreshing retires both old tokens.
    let refreshed = form(
        &app,
        "/api/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", &client_id),
            ("client_secret", &secret),
        ],
    )
    .await;
    let (access2, refresh2) = tokens(&refreshed);
    assert_ne!(access2, access);
    assert_ne!(refresh2, refresh);
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(userinfo(&app, &access2).await.status, StatusCode::OK);
    let reused = form(
        &app,
        "/api/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh),
            ("client_id", &client_id),
            ("client_secret", &secret),
        ],
    )
    .await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(reused.json["error"], "invalid_grant");

    // The application signs the player out (RFC 7009), with HTTP Basic this
    // time; a second revoke of the same token is still a 200.
    let basic = base64::engine::general_purpose::STANDARD.encode(format!("{client_id}:{secret}"));
    for _ in 0..2 {
        let request = Request::builder()
            .method(Method::POST)
            .uri("/api/oauth/revoke")
            .header(header::AUTHORIZATION, format!("Basic {basic}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_encode(&[("token", &refresh2)])))
            .expect("request builds");
        assert_eq!(send(&app, request).await.status, StatusCode::OK);
    }
    assert_eq!(
        userinfo(&app, &access2).await.status,
        StatusCode::UNAUTHORIZED
    );

    // The player withdraws consent: a fresh sign-in dies with it.
    let code = approve(&app, accounts.player, &client_id).await;
    let (access3, refresh3) = tokens(&exchange(&app, &client_id, &secret, &code).await);
    let withdrawn = site(
        &app,
        Method::DELETE,
        &format!("/api/oauth/authorizations/{client_id}"),
        Some(&player_token),
        None,
    )
    .await;
    assert_eq!(withdrawn.status, StatusCode::NO_CONTENT);
    assert_eq!(
        userinfo(&app, &access3).await.status,
        StatusCode::UNAUTHORIZED
    );
    let dead = form(
        &app,
        "/api/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh3),
            ("client_id", &client_id),
            ("client_secret", &secret),
        ],
    )
    .await;
    assert_eq!(dead.json["error"], "invalid_grant");

    accounts.delete().await;
}

#[tokio::test]
#[ignore = "writes to the local Postgres (and cleans up after itself)"]
async fn a_code_is_single_use_and_bound_to_its_request() {
    let accounts = Accounts::create().await;
    let app = app(accounts.pool.clone());
    let (client_id, secret) = register(&app, accounts.developer, "Code test").await;
    let (other_id, other_secret) = register(&app, accounts.developer, "Other app").await;

    let refused = |reply: &Reply, error: &str| {
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.json);
        assert_eq!(reply.json["error"], error, "{}", reply.json);
    };

    // A wrong secret never reaches the code, so the code survives it.
    let code = approve(&app, accounts.player, &client_id).await;
    let wrong_secret = exchange(&app, &client_id, "myrtle_cs_wrong", &code).await;
    assert_eq!(wrong_secret.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_secret.json["error"], "invalid_client");
    // An unknown application gets the same answer as a wrong secret.
    let unknown = exchange(&app, &Uuid::new_v4().to_string(), &secret, &code).await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.json["error"], "invalid_client");

    // Another application cannot redeem it, and cannot burn it by trying.
    refused(
        &exchange(&app, &other_id, &other_secret, &code).await,
        "invalid_grant",
    );
    let (access, _) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    // Replaying the spent code fails AND revokes what it issued.
    refused(
        &exchange(&app, &client_id, &secret, &code).await,
        "invalid_grant",
    );
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED
    );

    // PKCE: a wrong verifier, then a missing one. Each burns its code.
    for verifier in [
        Some("a-verifier-that-is-long-enough-but-not-the-right-one"),
        None,
    ] {
        let code = approve(&app, accounts.player, &client_id).await;
        let mut fields = vec![
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("client_id", client_id.as_str()),
            ("client_secret", secret.as_str()),
        ];
        if let Some(verifier) = verifier {
            fields.push(("code_verifier", verifier));
        }
        let reply = form(&app, "/api/oauth/token", &fields).await;
        refused(
            &reply,
            if verifier.is_some() {
                "invalid_grant"
            } else {
                "invalid_request"
            },
        );
        refused(
            &exchange(&app, &client_id, &secret, &code).await,
            "invalid_grant",
        );
    }

    // A redirect URI other than the one the code was issued for.
    let code = approve(&app, accounts.player, &client_id).await;
    let reply = form(
        &app,
        "/api/oauth/token",
        &[
            ("grant_type", "authorization_code"),
            ("code", &code),
            ("redirect_uri", "http://localhost:4000/callback"),
            ("code_verifier", VERIFIER),
            ("client_id", &client_id),
            ("client_secret", &secret),
        ],
    )
    .await;
    refused(&reply, "invalid_grant");

    // An expired code.
    let code = approve(&app, accounts.player, &client_id).await;
    sqlx::query(
        "UPDATE oauth_codes SET expires_at = now() - interval '1 second'
         WHERE code_hash = sha256(convert_to($1, 'UTF8'))",
    )
    .bind(&code)
    .execute(&accounts.pool)
    .await
    .expect("expire the code");
    refused(
        &exchange(&app, &client_id, &secret, &code).await,
        "invalid_grant",
    );

    // An unknown grant type, with good credentials.
    let reply = form(
        &app,
        "/api/oauth/token",
        &[
            ("grant_type", "password"),
            ("client_id", &client_id),
            ("client_secret", &secret),
        ],
    )
    .await;
    refused(&reply, "unsupported_grant_type");

    // A rotated secret: the old one stops authenticating, the new one works.
    let rotated = site(
        &app,
        Method::POST,
        &format!("/api/oauth/apps/{client_id}/secret"),
        Some(&site_token(accounts.developer)),
        None,
    )
    .await;
    let new_secret = rotated.json["client_secret"].as_str().expect("new secret");
    assert_ne!(new_secret, secret);
    let code = approve(&app, accounts.player, &client_id).await;
    assert_eq!(
        exchange(&app, &client_id, &secret, &code).await.json["error"],
        "invalid_client"
    );
    tokens(&exchange(&app, &client_id, new_secret, &code).await);

    accounts.delete().await;
}

#[tokio::test]
#[ignore = "writes to the local Postgres (and cleans up after itself)"]
async fn the_consent_page_only_ever_redirects_to_a_registered_uri() {
    let accounts = Accounts::create().await;
    let app = app(accounts.pool.clone());
    let (client_id, _) = register(&app, accounts.developer, "Redirect test").await;
    let player_token = site_token(accounts.player);
    let redirect = urlencoding::encode(REDIRECT);

    let check = |query: String| {
        let app = app.clone();
        let token = player_token.clone();
        async move {
            site(
                &app,
                Method::GET,
                &format!("/api/oauth/authorize?{query}"),
                Some(&token),
                None,
            )
            .await
        }
    };

    // Nowhere trustworthy to send the browser: the player is shown a 400.
    for query in [
        "response_type=code".to_owned(),
        "response_type=code&client_id=not-a-uuid".to_owned(),
        format!("response_type=code&client_id={}", Uuid::new_v4()),
        format!(
            "response_type=code&client_id={client_id}&redirect_uri=https%3A%2F%2Fevil.example%2Fcallback"
        ),
        // A prefix, a suffix and a different query are all different URIs.
        format!("response_type=code&client_id={client_id}&redirect_uri={redirect}%2Fextra"),
        format!("response_type=code&client_id={client_id}&redirect_uri={redirect}%3Fx%3D1"),
        // Two are registered, so leaving it out is ambiguous.
        format!("response_type=code&client_id={client_id}"),
    ] {
        let reply = check(query.clone()).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{query}: {}",
            reply.json
        );
        assert_eq!(reply.json["error"]["code"], "BAD_REQUEST", "{query}");
        assert!(reply.json.get("redirect_to").is_none(), "{query}");
    }

    // The application's own mistakes go back to it, at its registered URI.
    for (extra, error) in [
        ("response_type=token", "unsupported_response_type"),
        ("state=s", "invalid_request"),
        (
            "response_type=code&scope=identify%20roster",
            "invalid_scope",
        ),
        (
            "response_type=code&code_challenge=abc&code_challenge_method=S256",
            "invalid_request",
        ),
        (
            "response_type=code&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM&code_challenge_method=plain",
            "invalid_request",
        ),
        (
            "response_type=code&code_challenge=E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM",
            "invalid_request",
        ),
    ] {
        let query = format!("client_id={client_id}&redirect_uri={redirect}&{extra}");
        let reply = check(query.clone()).await;
        assert_eq!(reply.status, StatusCode::OK, "{query}: {}", reply.json);
        assert_eq!(reply.json["status"], "redirect", "{query}");
        let redirect_to = reply.json["redirect_to"].as_str().expect("redirect_to");
        assert!(
            redirect_to.starts_with(&format!("{REDIRECT}?")),
            "{redirect_to}"
        );
        assert_eq!(
            param(redirect_to, "error").as_deref(),
            Some(error),
            "{query}"
        );
    }

    // Approving an invalid request mints nothing, whatever `approve` says.
    let reply = site(
        &app,
        Method::POST,
        "/api/oauth/authorize",
        Some(&player_token),
        Some(json!({
            "response_type": "token", "client_id": client_id, "redirect_uri": REDIRECT,
            "approve": true,
        })),
    )
    .await;
    let redirect_to = reply.json["redirect_to"].as_str().expect("redirect_to");
    assert_eq!(param(redirect_to, "code"), None);
    assert_eq!(
        param(redirect_to, "error").as_deref(),
        Some("unsupported_response_type")
    );

    accounts.delete().await;
}

#[tokio::test]
#[ignore = "writes to the local Postgres (and cleans up after itself)"]
async fn an_application_is_its_owners_to_manage() {
    let accounts = Accounts::create().await;
    let app = app(accounts.pool.clone());
    let developer_token = site_token(accounts.developer);
    let player_token = site_token(accounts.player);
    let (client_id, secret) = register(&app, accounts.developer, "Owner test").await;
    let path = format!("/api/oauth/apps/{client_id}");
    let edit = json!({ "name": "Renamed", "redirect_uris": [REDIRECT] });

    // Registration validates every field and reports them together.
    let invalid = site(
        &app,
        Method::POST,
        "/api/oauth/apps",
        Some(&developer_token),
        Some(json!({ "name": " ", "redirect_uris": ["http://example.com/cb", "https://ok.example/cb#frag"] })),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    let details = invalid.json["error"]["details"]
        .as_array()
        .expect("details");
    let failed: Vec<&str> = details.iter().filter_map(|d| d["field"].as_str()).collect();
    assert_eq!(failed, ["name", "redirect_uris", "redirect_uris"]);

    // Someone else's application is a 404 to everyone but its owner.
    for (method, target, body) in [
        (Method::PUT, path.clone(), Some(edit.clone())),
        (Method::DELETE, path.clone(), None),
        (Method::POST, format!("{path}/secret"), None),
    ] {
        let reply = site(&app, method.clone(), &target, Some(&player_token), body).await;
        assert_eq!(reply.status, StatusCode::NOT_FOUND, "{method} {target}");
    }
    let theirs = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&player_token),
        None,
    )
    .await;
    assert_eq!(theirs.json, json!([]));

    // The owner edits it; a removed redirect URI stops working at once.
    let updated = site(&app, Method::PUT, &path, Some(&developer_token), Some(edit)).await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.json);
    assert_eq!(updated.json["name"], "Renamed");
    assert_eq!(updated.json["description"], "");
    assert_eq!(updated.json["redirect_uris"], json!([REDIRECT]));
    let removed = site(
        &app,
        Method::GET,
        &format!(
            "/api/oauth/authorize?response_type=code&client_id={client_id}&redirect_uri={}",
            urlencoding::encode("http://localhost:4000/callback")
        ),
        Some(&player_token),
        None,
    )
    .await;
    assert_eq!(removed.status, StatusCode::BAD_REQUEST);

    // The cap: ten applications an account.
    for n in 1..10 {
        register(&app, accounts.developer, &format!("Filler {n}")).await;
    }
    let over = site(
        &app,
        Method::POST,
        "/api/oauth/apps",
        Some(&developer_token),
        Some(json!({ "name": "One too many", "redirect_uris": [REDIRECT] })),
    )
    .await;
    assert_eq!(over.status, StatusCode::CONFLICT, "{}", over.json);

    // The takedown route is a super admin's, and no one else's: the owner of
    // the application is refused there too.
    let filler = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&developer_token),
        None,
    )
    .await;
    let filler_id = filler.json[1]["id"]
        .as_str()
        .expect("a second app")
        .to_owned();
    let takedown = format!("/api/admin/oauth/apps/{filler_id}");
    for token in [&player_token, &developer_token] {
        let reply = site(&app, Method::DELETE, &takedown, Some(token), None).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
    }
    let service = Request::builder()
        .method(Method::DELETE)
        .uri(&takedown)
        .header("x-service-key", "oauth-flow-service-key")
        .body(Body::empty())
        .expect("request builds");
    assert_eq!(send(&app, service).await.status, StatusCode::NO_CONTENT);
    let after = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&developer_token),
        None,
    )
    .await;
    assert_eq!(after.json.as_array().map(Vec::len), Some(9));

    // Deleting the application signs everyone out of it.
    let code = approve(&app, accounts.player, &client_id).await;
    let (access, _) = tokens(&exchange(&app, &client_id, &secret, &code).await);
    assert_eq!(userinfo(&app, &access).await.status, StatusCode::OK);
    let deleted = site(&app, Method::DELETE, &path, Some(&developer_token), None).await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED
    );
    let gone = site(
        &app,
        Method::GET,
        "/api/oauth/authorizations",
        Some(&player_token),
        None,
    )
    .await;
    assert_eq!(gone.json, json!([]));

    accounts.delete().await;
}

/// Deleting an account takes its OAuth rows with it, on both sides: what it
/// authorized, and what it registered (with other players' access to that).
#[tokio::test]
#[ignore = "writes to the local Postgres (and cleans up after itself)"]
async fn deleting_an_account_ends_its_oauth_access() {
    let accounts = Accounts::create().await;
    let app = app(accounts.pool.clone());
    let (client_id, secret) = register(&app, accounts.developer, "Deletion test").await;
    let code = approve(&app, accounts.player, &client_id).await;
    let (access, _) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    let deleted = site(
        &app,
        Method::POST,
        "/api/auth/delete-account",
        Some(&site_token(accounts.player)),
        None,
    )
    .await;
    assert_eq!(deleted.status, StatusCode::OK, "{}", deleted.json);
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED
    );
    let apps = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&site_token(accounts.developer)),
        None,
    )
    .await;
    assert_eq!(apps.json[0]["authorized_users"], 0);

    // A token for the deleted account can no longer consent to anything.
    let ghost = site(
        &app,
        Method::POST,
        "/api/oauth/authorize",
        Some(&site_token(accounts.player)),
        Some(json!({
            "response_type": "code", "client_id": client_id, "redirect_uri": REDIRECT,
            "approve": true,
        })),
    )
    .await;
    assert_eq!(ghost.status, StatusCode::UNAUTHORIZED);

    accounts.delete().await;
}
