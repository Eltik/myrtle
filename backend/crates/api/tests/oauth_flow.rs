//! "Sign in with myrtle.moe" end to end through the served router

#![allow(clippy::unwrap_used)]

#[path = "../../app/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use myrtle_app::AppContext;
use myrtle_app::context::Secrets;
use myrtle_app::services::accounts::jwt::create_token;
use myrtle_app::services::oauth::protocol::{self, SecretKind};
use myrtle_db::repos::oauth as repo;
use myrtle_testkit::db::TestDb;
use myrtle_types::{Role, Server, UserId};
use reqwest::Url;
use serde_json::{Value, json};
use tower::ServiceExt;
use uuid::Uuid;

const SERVICE_KEY: &str = "oauth-flow-service-key";
const REDIRECT: &str = "https://client.example/callback";
const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"; // the RFC 7636 appendix B pair
const CHALLENGE: &str = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

struct Reply {
    status: StatusCode,
    cache_control: Option<String>,
    www_authenticate: Option<String>,
    json: Value,
}

async fn send(app: &Router, request: Request) -> Reply {
    let response = app.clone().oneshot(request).await.unwrap();
    let text = |name: header::HeaderName| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let (status, cache_control, www_authenticate) = (
        response.status(),
        text(header::CACHE_CONTROL),
        text(header::WWW_AUTHENTICATE),
    );
    let bytes = to_bytes(response.into_body(), 1 << 20).await.unwrap();
    Reply {
        status,
        cache_control,
        www_authenticate,
        json: serde_json::from_slice(&bytes).unwrap_or_default(),
    }
}

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
    send(app, request.body(body).unwrap()).await
}

fn form_encode(fields: &[(&str, &str)]) -> String {
    fields
        .iter()
        .map(|(k, v)| format!("{k}={}", urlencoding::encode(v)))
        .collect::<Vec<_>>()
        .join("&")
}

async fn form(app: &Router, path: &str, fields: &[(&str, &str)]) -> Reply {
    let request = Request::post(path)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form_encode(fields)))
        .unwrap();
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

fn site_token(user: UserId) -> String {
    let now = chrono::Utc::now().timestamp() as u64;
    create_token(common::JWT_SECRET, user, "1", Server::EN, Role::User, now).unwrap()
}

fn param(url: &str, name: &str) -> Option<String> {
    Url::parse(url)
        .unwrap()
        .query_pairs()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.into_owned())
}

fn with_service_key(mut ctx: AppContext) -> AppContext {
    ctx.secrets = Arc::new(Secrets {
        jwt: ctx.secrets.jwt.clone(),
        credentials: Arc::clone(&ctx.secrets.credentials),
        service_key: Some(SERVICE_KEY.into()),
        discord_alert_webhook: None,
    });
    ctx
}

async fn bare() -> (Router, TestDb) {
    let (ctx, db) = common::context().await;
    (myrtle_api::router::build(with_service_key(ctx)), db)
}

#[tokio::test]
async fn userinfo_without_a_token_is_the_rfc_error_not_the_site_envelope() {
    let (app, db) = bare().await;
    let reply = send(
        &app,
        Request::get("/api/oauth/userinfo")
            .body(Body::empty())
            .unwrap(),
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
    db.close().await;
}

#[tokio::test]
async fn the_token_endpoint_refuses_json_and_missing_credentials() {
    let (app, db) = bare().await;

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
    assert_eq!(reply.cache_control.as_deref(), Some("no-store"));

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
    db.close().await;
}

#[tokio::test]
async fn an_access_token_is_not_a_site_token() {
    let (app, db) = bare().await;
    let access_token = protocol::mint(SecretKind::AccessToken);
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
    db.close().await;
}

/// A developer and a player, each with an in-game name
struct Accounts {
    ctx: AppContext,
    db: TestDb,
    developer: UserId,
    player: UserId,
}

impl Accounts {
    async fn create() -> Self {
        let (ctx, db) = common::context().await;
        let ctx = with_service_key(ctx);
        let developer = db.user(1, Server::EN).await;
        let player = db.user(2, Server::EN).await;
        let mut conn = ctx.db.conn().await.unwrap();
        for (user, nickname, tag, avatar) in [
            (developer, "Developer", "0001", None),
            (player, "Doctor", "1234", Some("char_002_amiya@winter#1")),
        ] {
            sqlx::query(
                "UPDATE users SET nickname = $2, nick_number = $3, avatar_id = $4 WHERE id = $1",
            )
            .bind(user.0)
            .bind(nickname)
            .bind(tag)
            .bind(avatar)
            .execute(&mut *conn)
            .await
            .unwrap();
        }
        Self {
            ctx,
            db,
            developer,
            player,
        }
    }

    fn app(&self) -> Router {
        myrtle_api::router::build(self.ctx.clone())
    }

    /// Both accounts go, and by cascade every application, grant, code and token the test made
    async fn delete(self) {
        let ids = vec![self.developer.0, self.player.0];
        let mut conn = self.ctx.db.conn().await.unwrap();
        sqlx::query("DELETE FROM users WHERE id = ANY($1)")
            .bind(&ids)
            .execute(&mut *conn)
            .await
            .unwrap();
        let left: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM oauth_apps WHERE owner_id = ANY($1))
                  + (SELECT COUNT(*) FROM oauth_grants WHERE user_id = ANY($1))",
        )
        .bind(&ids)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
        assert_eq!(left, 0, "deleting the accounts left OAuth rows behind");
        drop(conn);
        self.db.close().await;
    }
}

async fn register(app: &Router, owner: UserId, name: &str) -> (String, String) {
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
    let id = reply.json["id"].as_str().unwrap().to_owned();
    let secret = reply.json["client_secret"].as_str().unwrap().to_owned();
    assert!(secret.starts_with("myrtle_cs_"), "{secret}");
    (id, secret)
}

async fn approve(app: &Router, player: UserId, client_id: &str) -> String {
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
            "code_challenge": CHALLENGE,
            "code_challenge_method": "S256",
            "approve": true,
        })),
    )
    .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.json);
    let redirect_to = reply.json["redirect_to"].as_str().unwrap();
    assert!(redirect_to.starts_with(REDIRECT), "{redirect_to}");
    assert_eq!(param(redirect_to, "state").as_deref(), Some("st&te=1"));
    param(redirect_to, "code").unwrap()
}

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
    let access = reply.json["access_token"].as_str().unwrap();
    let refresh = reply.json["refresh_token"].as_str().unwrap();
    assert!(access.starts_with("myrtle_at_"), "{access}");
    assert!(refresh.starts_with("myrtle_rt_"), "{refresh}");
    (access.to_owned(), refresh.to_owned())
}

async fn refresh_with(app: &Router, refresh: &str, client_id: &str, secret: &str) -> Reply {
    form(
        app,
        "/api/oauth/token",
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh),
            ("client_id", client_id),
            ("client_secret", secret),
        ],
    )
    .await
}

#[tokio::test]
async fn a_player_signs_in_to_a_third_party_application() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
    let (client_id, secret) = register(&app, accounts.developer, "Flow test").await;
    let player_token = site_token(accounts.player);

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
    assert!(!check.json.to_string().contains("secret"));

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
    let redirect_to = declined.json["redirect_to"].as_str().unwrap();
    assert_eq!(
        param(redirect_to, "error").as_deref(),
        Some("access_denied")
    );
    assert_eq!(param(redirect_to, "state").as_deref(), Some("abc"));
    assert_eq!(param(redirect_to, "code"), None);

    let code = approve(&app, accounts.player, &client_id).await;
    assert!(code.starts_with("myrtle_ac_"), "{code}");
    let (access, refresh) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    let me = userinfo(&app, &access).await;
    assert_eq!(me.status, StatusCode::OK, "{}", me.json);
    assert_eq!(me.cache_control.as_deref(), Some("no-store"));
    assert_eq!(me.json["id"], accounts.player.0.to_string());
    assert_eq!(me.json["username"], "Doctor");
    assert_eq!(me.json["tag"], "1234");
    assert_eq!(me.json["uid"], "2");
    assert_eq!(me.json["server"], "en");
    assert_eq!(me.json["avatar_id"], "char_002_amiya@winter#1");
    assert!(
        me.json["avatar_url"]
            .as_str()
            .is_some_and(|u| u.ends_with("/api/avatar/char_002_amiya_winter%231")),
        "{}",
        me.json["avatar_url"]
    );
    let keys: Vec<&String> = me.json.as_object().unwrap().keys().collect();
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

    let verify = site(&app, Method::GET, "/api/auth/verify", Some(&access), None).await;
    assert_eq!(verify.status, StatusCode::UNAUTHORIZED);
    let as_site = userinfo(&app, &player_token).await;
    assert_eq!(as_site.status, StatusCode::UNAUTHORIZED);
    assert_eq!(as_site.json["error"], "invalid_token");
    assert_eq!(
        userinfo(&app, &refresh).await.status,
        StatusCode::UNAUTHORIZED
    );

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

    let (access2, refresh2) = tokens(&refresh_with(&app, &refresh, &client_id, &secret).await);
    assert_ne!(access2, access);
    assert_ne!(refresh2, refresh);
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(userinfo(&app, &access2).await.status, StatusCode::OK);
    let reused = refresh_with(&app, &refresh, &client_id, &secret).await;
    assert_eq!(reused.status, StatusCode::BAD_REQUEST);
    assert_eq!(reused.json["error"], "invalid_grant");

    let basic = STANDARD.encode(format!("{client_id}:{secret}"));
    for _ in 0..2 {
        let request = Request::post("/api/oauth/revoke")
            .header(header::AUTHORIZATION, format!("Basic {basic}"))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(Body::from(form_encode(&[("token", &refresh2)])))
            .unwrap();
        let revoked = send(&app, request).await;
        assert_eq!(revoked.status, StatusCode::OK);
        assert_eq!(revoked.cache_control.as_deref(), Some("no-store"));
        assert_eq!(revoked.json, Value::Null, "an empty body");
    }
    assert_eq!(
        userinfo(&app, &access2).await.status,
        StatusCode::UNAUTHORIZED
    );

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
    let dead = refresh_with(&app, &refresh3, &client_id, &secret).await;
    assert_eq!(dead.json["error"], "invalid_grant");

    accounts.delete().await;
}

#[tokio::test]
async fn a_code_is_single_use_and_bound_to_its_request() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
    let (client_id, secret) = register(&app, accounts.developer, "Code test").await;
    let (other_id, other_secret) = register(&app, accounts.developer, "Other app").await;

    let refused = |reply: &Reply, error: &str| {
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{}", reply.json);
        assert_eq!(reply.json["error"], error, "{}", reply.json);
    };

    let code = approve(&app, accounts.player, &client_id).await;
    let wrong_secret = exchange(&app, &client_id, "myrtle_cs_wrong", &code).await;
    assert_eq!(wrong_secret.status, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_secret.json["error"], "invalid_client");
    let unknown = exchange(&app, &Uuid::new_v4().to_string(), &secret, &code).await;
    assert_eq!(unknown.status, StatusCode::UNAUTHORIZED);
    assert_eq!(unknown.json["error"], "invalid_client");

    refused(
        &exchange(&app, &other_id, &other_secret, &code).await,
        "invalid_grant",
    );
    let (access, _) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    refused(
        &exchange(&app, &client_id, &secret, &code).await,
        "invalid_grant",
    );
    assert_eq!(
        userinfo(&app, &access).await.status,
        StatusCode::UNAUTHORIZED,
        "a replayed code revokes what it issued"
    );

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

    let code = approve(&app, accounts.player, &client_id).await;
    sqlx::query(
        "UPDATE oauth_codes SET expires_at = now() - interval '1 second'
         WHERE code_hash = sha256(convert_to($1, 'UTF8'))",
    )
    .bind(&code)
    .execute(&mut *accounts.ctx.db.conn().await.unwrap())
    .await
    .unwrap();
    refused(
        &exchange(&app, &client_id, &secret, &code).await,
        "invalid_grant",
    );

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

    let rotated = site(
        &app,
        Method::POST,
        &format!("/api/oauth/apps/{client_id}/secret"),
        Some(&site_token(accounts.developer)),
        None,
    )
    .await;
    let new_secret = rotated.json["client_secret"].as_str().unwrap();
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
async fn the_consent_page_only_ever_redirects_to_a_registered_uri() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
    let (client_id, _) = register(&app, accounts.developer, "Redirect test").await;
    let player_token = site_token(accounts.player);
    let redirect = urlencoding::encode(REDIRECT);
    let check = |query: String| {
        let (app, token) = (app.clone(), player_token.clone());
        async move {
            let path = format!("/api/oauth/authorize?{query}");
            site(&app, Method::GET, &path, Some(&token), None).await
        }
    };

    for query in [
        "response_type=code".to_owned(),
        "response_type=code&client_id=not-a-uuid".to_owned(),
        format!("response_type=code&client_id={}", Uuid::new_v4()),
        format!(
            "response_type=code&client_id={client_id}&redirect_uri=https%3A%2F%2Fevil.example%2Fcallback"
        ),
        format!("response_type=code&client_id={client_id}&redirect_uri={redirect}%2Fextra"),
        format!("response_type=code&client_id={client_id}&redirect_uri={redirect}%3Fx%3D1"),
        format!("response_type=code&client_id={client_id}"), // two are registered, so leaving it out is ambiguous
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
        let redirect_to = reply.json["redirect_to"].as_str().unwrap();
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
    let redirect_to = reply.json["redirect_to"].as_str().unwrap();
    assert_eq!(param(redirect_to, "code"), None);
    assert_eq!(
        param(redirect_to, "error").as_deref(),
        Some("unsupported_response_type")
    );

    accounts.delete().await;
}

#[tokio::test]
async fn an_application_is_its_owners_to_manage() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
    let developer_token = site_token(accounts.developer);
    let player_token = site_token(accounts.player);
    let (client_id, secret) = register(&app, accounts.developer, "Owner test").await;
    let path = format!("/api/oauth/apps/{client_id}");
    let edit = json!({ "name": "Renamed", "redirect_uris": [REDIRECT] });

    let invalid = site(
        &app,
        Method::POST,
        "/api/oauth/apps",
        Some(&developer_token),
        Some(json!({ "name": " ", "redirect_uris": ["http://example.com/cb", "https://ok.example/cb#frag"] })),
    )
    .await;
    assert_eq!(invalid.status, StatusCode::UNPROCESSABLE_ENTITY);
    let details = invalid.json["error"]["details"].as_array().unwrap();
    let failed: Vec<&str> = details.iter().filter_map(|d| d["field"].as_str()).collect();
    assert_eq!(failed, ["name", "redirect_uris", "redirect_uris"]);

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

    let filler = site(
        &app,
        Method::GET,
        "/api/oauth/apps",
        Some(&developer_token),
        None,
    )
    .await;
    let filler_id = filler.json[1]["id"].as_str().unwrap().to_owned();
    let takedown = format!("/api/admin/oauth/apps/{filler_id}");
    for token in [&player_token, &developer_token] {
        let reply = site(&app, Method::DELETE, &takedown, Some(token), None).await;
        assert_eq!(reply.status, StatusCode::FORBIDDEN);
    }
    let service = Request::delete(&takedown)
        .header("x-service-key", SERVICE_KEY)
        .body(Body::empty())
        .unwrap();
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

#[tokio::test]
async fn deleting_an_account_ends_its_oauth_access() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
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

#[tokio::test]
async fn one_application_cannot_touch_anothers_tokens_and_a_grant_is_its_players() {
    let accounts = Accounts::create().await;
    let app = accounts.app();
    let (client_id, secret) = register(&app, accounts.developer, "First").await;
    let (other_id, other_secret) = register(&app, accounts.developer, "Second").await;
    let code = approve(&app, accounts.player, &client_id).await;
    let (access, refresh) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    let stolen = refresh_with(&app, &refresh, &other_id, &other_secret).await;
    assert_eq!(stolen.status, StatusCode::BAD_REQUEST);
    assert_eq!(stolen.json["error"], "invalid_grant");

    for token in [&access, &refresh] {
        let revoked = form(
            &app,
            "/api/oauth/revoke",
            &[
                ("token", token),
                ("client_id", &other_id),
                ("client_secret", &other_secret),
            ],
        )
        .await;
        assert_eq!(revoked.status, StatusCode::OK);
    }
    assert_eq!(userinfo(&app, &access).await.status, StatusCode::OK);

    let withdrawn = site(
        &app,
        Method::DELETE,
        &format!("/api/oauth/authorizations/{client_id}"),
        Some(&site_token(accounts.developer)),
        None,
    )
    .await;
    assert_eq!(withdrawn.status, StatusCode::NOT_FOUND);
    assert_eq!(userinfo(&app, &access).await.status, StatusCode::OK);
    tokens(&refresh_with(&app, &refresh, &client_id, &secret).await);

    accounts.delete().await;
}

#[tokio::test]
async fn userinfo_needs_the_identify_scope_and_names_the_configured_api_base() {
    let mut accounts = Accounts::create().await;
    let app = accounts.app();
    let (client_id, secret) = register(&app, accounts.developer, "Scope test").await;
    let code = approve(&app, accounts.player, &client_id).await;
    let (access, _) = tokens(&exchange(&app, &client_id, &secret, &code).await);

    let behind_proxy = |token: &str| {
        Request::get("/api/oauth/userinfo")
            .header(header::AUTHORIZATION, format!("bearer {token}"))
            .header(header::HOST, "api.example.test:3060")
            .header("x-forwarded-proto", "https")
            .body(Body::empty())
            .unwrap()
    };
    let derived = send(&app, behind_proxy(&access)).await;
    assert_eq!(
        derived.status,
        StatusCode::OK,
        "the scheme is case-insensitive"
    );
    assert_eq!(
        derived.json["avatar_url"],
        "https://api.example.test:3060/api/avatar/char_002_amiya_winter%231"
    );

    let mut config = (*accounts.ctx.config).clone();
    config.server.public_api_url = "https://api.myrtle.example/".into();
    accounts.ctx.config = Arc::new(config);
    let configured = send(&accounts.app(), behind_proxy(&access)).await;
    assert_eq!(
        configured.json["avatar_url"],
        "https://api.myrtle.example/api/avatar/char_002_amiya_winter%231"
    );

    let grant = {
        let mut conn = accounts.ctx.db.conn().await.unwrap();
        repo::upsert_grant(
            &mut conn,
            client_id.parse().unwrap(),
            accounts.player,
            "identify",
        )
        .await
        .unwrap()
    };
    let scopeless = protocol::mint(SecretKind::AccessToken);
    repo::insert_token(
        &mut accounts.ctx.db.conn().await.unwrap(),
        grant,
        &protocol::digest(&scopeless),
        &protocol::digest(&protocol::mint(SecretKind::RefreshToken)),
        "",
        60,
        60,
        20,
    )
    .await
    .unwrap();
    let refused = userinfo(&app, &scopeless).await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    assert_eq!(refused.json["error"], "insufficient_scope");
    assert_eq!(refused.www_authenticate, None);
    assert_eq!(refused.cache_control.as_deref(), Some("no-store"));

    accounts.delete().await;
}
