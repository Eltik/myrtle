//! The Trevor routes as the server wires them, on the paths that end before
//! the database: input validation, the hourly allowance, Turnstile (with a
//! fake verifier, no network), the scoped worker key and its kill switch,
//! the 64 KB result cap, the publish route's gzip and its two body limits,
//! and the SSE body outliving the 30 s handler timeout. The pool is lazy and
//! points at a closed port, so a case that reached Postgres would fail rather
//! than touch a real database. The queue itself is exercised against a
//! throwaway database in `trevor_db_test.rs`.
//!
//! `cargo test --test trevor_handler_test`

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use axum::Json;
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, Request, StatusCode, header};
use axum::response::IntoResponse;
use axum::routing::get;
use backend::app::cache::keys::CacheKey;
use backend::app::cache::store::CacheStore;
use backend::app::extractors::auth::AuthUser;
use backend::app::routes::trevor::{self, ClientIp, PanelsQuery};
use backend::app::services::trevor::runtime::{StreamEvent, TrevorConfig};
use backend::app::services::trevor::turnstile::TurnstileVerifier;
use backend::app::services::trevor::{
    AskRequest, FeedbackRequest, JobStatus, JobView, Rung, stream,
};
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::auth::permissions::GlobalRole;
use backend::core::hypergryph::constants::Server;
use futures_util::StreamExt;
use futures_util::future::BoxFuture;
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;
use uuid::Uuid;

const SERVICE_KEY: &str = "test-service-key";
const WORKER_KEY: &str = "test-worker-key-0123456789";
const JWT_SECRET: &str = "test-jwt-secret-not-used-by-this-test";

fn state() -> AppState {
    let db = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(200))
        .connect_lazy("postgres://nobody:nothing@127.0.0.1:1/none")
        .expect("lazy pool");
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: SERVICE_KEY.into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: true,
    };
    let s = AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::new(),
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    );
    s.trevor.set_config(TrevorConfig {
        worker_key: WORKER_KEY.into(),
        ..TrevorConfig::default()
    });
    s
}

fn with_config(s: &AppState, f: impl FnOnce(&mut TrevorConfig)) {
    let mut cfg = (*s.trevor.config()).clone();
    f(&mut cfg);
    s.trevor.set_config(cfg);
}

fn user(id: Uuid) -> AuthUser {
    AuthUser {
        user_id: id.to_string(),
        uid: "12345678".into(),
        server: "en".into(),
        token_role: GlobalRole::default(),
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

async fn error_message(r: axum::response::Response) -> String {
    let body = axum::body::to_bytes(r.into_body(), 1 << 20)
        .await
        .expect("body");
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap_or_default();
    v["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

/// The `/api` router exactly as the server builds it (without the global
/// rate-limit and observe layers).
fn app(s: &AppState) -> Router {
    backend::app::server::api_parts().0.with_state(s.clone())
}

async fn call(app: &Router, req: Request<Body>) -> axum::response::Response {
    app.clone().oneshot(req).await.expect("infallible")
}

fn post_json(path: &str, headers: &[(&str, &str)], body: &serde_json::Value) -> Request<Body> {
    let mut b = Request::post(path).header(header::CONTENT_TYPE, "application/json");
    for (k, v) in headers {
        b = b.header(*k, *v);
    }
    b.body(Body::from(body.to_string())).expect("request")
}

#[tokio::test]
async fn ask_refuses_an_empty_question_and_an_unknown_server() {
    let s = state();
    let me = Uuid::new_v4();
    let empty = trevor::ask(
        State(s.clone()),
        user(me),
        ClientIp(None),
        Json(question("  ?? ", None)),
    )
    .await;
    assert_eq!(status(empty), StatusCode::BAD_REQUEST);
    let jp = trevor::ask(
        State(s),
        user(me),
        ClientIp(None),
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
    let over = trevor::ask(
        State(s),
        user(me),
        ClientIp(None),
        Json(question("Who is Amiya?", None)),
    )
    .await;
    assert_eq!(status(over), StatusCode::TOO_MANY_REQUESTS);
}

/// A Turnstile verifier that records its calls and answers as told.
struct FakeVerifier {
    answer: Result<bool, String>,
    calls: Mutex<Vec<(String, String, Option<IpAddr>)>>,
}

impl TurnstileVerifier for FakeVerifier {
    fn verify<'a>(
        &'a self,
        secret: &'a str,
        token: &'a str,
        remote_ip: Option<IpAddr>,
    ) -> BoxFuture<'a, Result<bool, String>> {
        self.calls
            .lock()
            .unwrap()
            .push((secret.to_owned(), token.to_owned(), remote_ip));
        let answer = self.answer.clone();
        Box::pin(async move { answer })
    }
}

fn turnstile_state(answer: Result<bool, String>) -> (AppState, Arc<FakeVerifier>) {
    let s = state();
    with_config(&s, |c| c.turnstile_secret = Some("ts-secret".into()));
    let fake = Arc::new(FakeVerifier {
        answer,
        calls: Mutex::new(Vec::new()),
    });
    s.trevor.set_verifier(fake.clone());
    (s, fake)
}

fn ask_with_token(token: Option<&str>) -> AskRequest {
    serde_json::from_value(json!({ "question": "Who is Amiya?", "turnstileToken": token }))
        .expect("ask body")
}

#[tokio::test]
async fn turnstile_refuses_a_missing_or_rejected_token() {
    let ip: IpAddr = "198.51.100.4".parse().unwrap();
    let (s, fake) = turnstile_state(Ok(false));
    let none = trevor::ask(
        State(s.clone()),
        user(Uuid::new_v4()),
        ClientIp(Some(ip)),
        Json(ask_with_token(None)),
    )
    .await;
    assert_eq!(status(none), StatusCode::FORBIDDEN);
    assert!(fake.calls.lock().unwrap().is_empty());
    let rejected = trevor::ask(
        State(s),
        user(Uuid::new_v4()),
        ClientIp(Some(ip)),
        Json(ask_with_token(Some("tok-1"))),
    )
    .await;
    assert_eq!(status(rejected), StatusCode::FORBIDDEN);
    assert_eq!(
        fake.calls.lock().unwrap().as_slice(),
        &[("ts-secret".to_owned(), "tok-1".to_owned(), Some(ip))]
    );
}

#[tokio::test]
async fn turnstile_fails_closed_when_siteverify_is_unreachable() {
    let (s, _) = turnstile_state(Err("connection refused".into()));
    let r = trevor::ask(
        State(s),
        user(Uuid::new_v4()),
        ClientIp(None),
        Json(ask_with_token(Some("tok"))),
    )
    .await
    .expect_err("fails closed");
    let resp = r.into_response();
    assert_eq!(resp.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(error_message(resp).await, "Trevor is busy");
}

#[tokio::test]
async fn turnstile_passes_a_good_token_on_to_the_ask() {
    let (s, fake) = turnstile_state(Ok(true));
    let r = trevor::ask(
        State(s),
        user(Uuid::new_v4()),
        ClientIp(None),
        Json(ask_with_token(Some("tok"))),
    )
    .await;
    // Past Turnstile the ask reaches the (closed) database: not a 403, and
    // not the Turnstile 503.
    let resp = match r {
        Ok(v) => v,
        Err(e) => e.into_response(),
    };
    assert_ne!(resp.status(), StatusCode::FORBIDDEN);
    assert_ne!(error_message(resp).await, "Trevor is busy");
    assert_eq!(fake.calls.lock().unwrap().len(), 1);
}

#[tokio::test]
async fn turnstile_is_skipped_without_a_secret() {
    let s = state();
    let fake = Arc::new(FakeVerifier {
        answer: Ok(false),
        calls: Mutex::new(Vec::new()),
    });
    s.trevor.set_verifier(fake.clone());
    let r = trevor::ask(
        State(s),
        user(Uuid::new_v4()),
        ClientIp(None),
        Json(ask_with_token(None)),
    )
    .await;
    assert_ne!(status(r), StatusCode::FORBIDDEN);
    assert!(fake.calls.lock().unwrap().is_empty());
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
        Query(PanelsQuery {
            server: None,
            v: None,
        }),
        HeaderMap::new(),
    )
    .await;
    assert_eq!(status(r), StatusCode::BAD_REQUEST);
}

/// Every worker route, with a body that fails validation before the
/// database: a 400 proves the request got past the key check.
fn worker_requests(headers: &[(&str, &str)]) -> Vec<Request<Body>> {
    let job = Uuid::new_v4();
    vec![
        post_json(
            "/api/trevor/worker/next",
            headers,
            &json!({ "workerId": "" }),
        ),
        post_json(
            "/api/trevor/worker/heartbeat",
            headers,
            &json!({ "workerId": "", "rungs": { "cf": true, "cpu": false } }),
        ),
        post_json(
            &format!("/api/trevor/worker/jobs/{job}/delta"),
            headers,
            &json!({ "workerId": "", "seq": 0, "text": "a" }),
        ),
        post_json(
            "/api/trevor/worker/result",
            headers,
            &json!({ "jobId": job, "workerId": "", "ok": false }),
        ),
        post_json("/api/trevor/versions", headers, &json!({ "version": "" })),
    ]
}

async fn statuses(app: &Router, headers: &[(&str, &str)]) -> Vec<StatusCode> {
    let mut out = Vec::new();
    for req in worker_requests(headers) {
        out.push(call(app, req).await.status());
    }
    out
}

fn bearer() -> String {
    let token = backend::core::auth::jwt::create_token(
        JWT_SECRET,
        &Uuid::new_v4().to_string(),
        "12345678",
        "en",
        "user",
        1,
    )
    .expect("token");
    format!("Bearer {token}")
}

#[tokio::test]
async fn worker_routes_take_only_the_worker_key() {
    let s = state();
    let app = app(&s);
    let five = |code| vec![code; 5];
    assert_eq!(
        statuses(&app, &[("x-trevor-worker-key", WORKER_KEY)]).await,
        five(StatusCode::BAD_REQUEST)
    );
    assert_eq!(statuses(&app, &[]).await, five(StatusCode::UNAUTHORIZED));
    assert_eq!(
        statuses(&app, &[("x-trevor-worker-key", "wrong")]).await,
        five(StatusCode::UNAUTHORIZED)
    );
    // The global service key (a SuperAdmin principal) is not a worker key.
    assert_eq!(
        statuses(&app, &[("x-service-key", SERVICE_KEY)]).await,
        five(StatusCode::UNAUTHORIZED)
    );
    let token = bearer();
    assert_eq!(
        statuses(&app, &[("authorization", token.as_str())]).await,
        five(StatusCode::UNAUTHORIZED)
    );
}

#[tokio::test]
async fn an_empty_worker_key_turns_the_worker_routes_off() {
    let s = state();
    with_config(&s, |c| c.worker_key = String::new());
    let app = app(&s);
    assert_eq!(
        statuses(&app, &[("x-trevor-worker-key", "")]).await,
        vec![StatusCode::SERVICE_UNAVAILABLE; 5]
    );
    assert_eq!(
        statuses(&app, &[("x-service-key", SERVICE_KEY)]).await,
        vec![StatusCode::SERVICE_UNAVAILABLE; 5]
    );
}

#[tokio::test]
async fn the_worker_key_opens_nothing_else() {
    let s = state();
    let app = app(&s);
    let ask = call(
        &app,
        post_json(
            "/api/trevor/ask",
            &[("x-trevor-worker-key", WORKER_KEY)],
            &json!({ "question": "" }),
        ),
    )
    .await;
    assert_eq!(ask.status(), StatusCode::UNAUTHORIZED);
    let job = call(
        &app,
        Request::get(format!("/api/trevor/jobs/{}", Uuid::new_v4()))
            .header("x-trevor-worker-key", WORKER_KEY)
            .body(Body::empty())
            .unwrap(),
    )
    .await;
    assert_eq!(job.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn scoped_key_switched_off_restores_the_service_key_gate() {
    let s = state();
    with_config(&s, |c| c.scoped_worker_key = false);
    let app = app(&s);
    assert_eq!(
        statuses(&app, &[("x-service-key", SERVICE_KEY)]).await,
        vec![StatusCode::BAD_REQUEST; 5]
    );
    assert_eq!(
        statuses(&app, &[("x-trevor-worker-key", WORKER_KEY)]).await,
        vec![StatusCode::UNAUTHORIZED; 5]
    );
    // A user token: authenticated, then refused as not the service (old 403).
    let token = bearer();
    assert_eq!(
        statuses(&app, &[("authorization", token.as_str())]).await,
        vec![StatusCode::FORBIDDEN; 5]
    );
}

/// A failed result padded to `len` bytes with a long `error`.
fn result_body(len: usize) -> String {
    let head = format!(
        r#"{{"jobId":"{}","workerId":"","ok":false,"error":""#,
        Uuid::nil()
    );
    let tail = r#""}"#;
    let pad = len - head.len() - tail.len();
    format!("{head}{}{tail}", "e".repeat(pad))
}

#[tokio::test]
async fn a_result_over_64_kb_is_a_413() {
    let s = state();
    let app = app(&s);
    let send = |len: usize| {
        Request::post("/api/trevor/worker/result")
            .header(header::CONTENT_TYPE, "application/json")
            .header("x-trevor-worker-key", WORKER_KEY)
            .body(Body::from(result_body(len)))
            .unwrap()
    };
    // At the cap the body is read (and refused for its empty workerId).
    assert_eq!(
        call(&app, send(65_536)).await.status(),
        StatusCode::BAD_REQUEST
    );
    let over = call(&app, send(65_537)).await;
    assert_eq!(over.status(), StatusCode::PAYLOAD_TOO_LARGE);
    // TREVOR_RESULT_MAX_BYTES=0 lifts the cap (the old behaviour).
    with_config(&s, |c| c.result_max_bytes = 0);
    assert_eq!(
        call(&app, send(1_000_000)).await.status(),
        StatusCode::BAD_REQUEST
    );
}

/// Gzip `body` with the server's own response compression, so the test
/// needs no gzip crate: a one-route router that echoes the bytes, asked
/// with `Accept-Encoding: gzip`.
async fn gzip(body: Vec<u8>) -> Vec<u8> {
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

/// A publish body that fails validation (empty version) once parsed, with
/// `content` as one panel's text.
fn publish_body_with(content: String) -> Vec<u8> {
    json!({ "version": "", "activate": false, "panels": { "s1": { "x": content } } })
        .to_string()
        .into_bytes()
}

/// About `len` bytes of highly compressible JSON.
fn publish_body(len: usize) -> Vec<u8> {
    publish_body_with("abcdefgh".repeat(len / 8))
}

/// `len` characters of base64-alphabet noise (6 bits each), which gzip can
/// only squeeze to about 3/4.
fn noise(len: usize) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut out = String::with_capacity(len);
    while out.len() < len {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        for i in 0..10 {
            out.push(ALPHABET[((x >> (6 * i)) & 63) as usize] as char);
        }
    }
    out.truncate(len);
    out
}

fn publish(body: Vec<u8>, gz: bool) -> Request<Body> {
    let mut b = Request::post("/api/trevor/versions")
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-trevor-worker-key", WORKER_KEY);
    if gz {
        b = b.header(header::CONTENT_ENCODING, "gzip");
    }
    b.body(Body::from(body)).unwrap()
}

const MIB: usize = 1024 * 1024;

#[tokio::test]
async fn publish_takes_gzip_up_to_4_mib_sent_and_8_mib_decompressed() {
    let s = state();
    let app = app(&s);

    // 7 MiB of JSON that gzips to a few KB: under both limits, so it is
    // decompressed and parsed (400 for the empty version, not 413).
    let seven = publish_body(7 * MIB);
    let seven_gz = gzip(seven).await;
    let seven_len = seven_gz.len();
    assert!(seven_len < MIB / 10, "{seven_len} bytes");
    assert_eq!(
        call(&app, publish(seven_gz, true)).await.status(),
        StatusCode::BAD_REQUEST
    );

    // 9 MiB decompressed from a small gzip: the 8 MiB limit applies to the
    // DECOMPRESSED body.
    let nine_gz = gzip(publish_body(9 * MIB)).await;
    let nine_len = nine_gz.len();
    assert!(nine_len < MIB);
    assert_eq!(
        call(&app, publish(nine_gz, true)).await.status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );

    // 6 MiB of noise gzips to over 4 MiB: refused on the bytes as SENT,
    // although it would decompress to under 8 MiB.
    let noisy = publish_body_with(noise(6 * MIB));
    let noisy_gz = gzip(noisy).await;
    assert!(noisy_gz.len() > 4 * MIB, "{} bytes", noisy_gz.len());
    eprintln!(
        "gzip: 7 MiB -> {seven_len} B, 9 MiB -> {nine_len} B, 6 MiB noise -> {} B",
        noisy_gz.len()
    );
    assert_eq!(
        call(&app, publish(noisy_gz, true)).await.status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );

    // Plain JSON: 3 MiB passes (axum's 2 MB default no longer applies), 5 MiB
    // is over the 4 MiB sent limit.
    assert_eq!(
        call(&app, publish(publish_body(3 * MIB), false))
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        call(&app, publish(publish_body(5 * MIB), false))
            .await
            .status(),
        StatusCode::PAYLOAD_TOO_LARGE
    );

    // An encoding other than gzip is refused.
    let br = Request::post("/api/trevor/versions")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_ENCODING, "br")
        .header("x-trevor-worker-key", WORKER_KEY)
        .body(Body::from(publish_body(1024)))
        .unwrap();
    assert_eq!(
        call(&app, br).await.status(),
        StatusCode::UNSUPPORTED_MEDIA_TYPE
    );
}

fn done_view(job_id: Uuid) -> JobView {
    JobView {
        job_id,
        status: JobStatus::Done,
        rung: Rung::Cf,
        position: 0,
        eta_secs: None,
        worker_online: true,
        answer: None,
        error: None,
    }
}

/// The SSE body under the server's real `observe` layer (and its 30 s
/// handler timeout), in paused time: keep-alives at 15 and 30 s, a delta at
/// 40 s and `done` at 41 s all arrive, and the stream then ends.
#[tokio::test(start_paused = true)]
async fn an_sse_stream_outlives_the_30_s_handler_timeout() {
    let s = state();
    let job = Uuid::new_v4();
    let rx = s.trevor.subscribe(job);
    let rx = Arc::new(Mutex::new(Some(rx)));
    let app: Router = Router::new()
        .route(
            "/s",
            get(move || {
                let rx = rx.lock().unwrap().take().expect("one request");
                async move { stream::sse(stream::events(None, rx), Duration::from_secs(15)) }
            }),
        )
        .layer(axum::middleware::from_fn(backend::app::middleware::observe));
    let resp = app
        .oneshot(Request::get("/s").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    assert_eq!(
        resp.headers().get(header::CONTENT_TYPE).unwrap(),
        "text/event-stream"
    );
    let rt = s.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(40)).await;
        rt.trevor.publish(
            job,
            StreamEvent::Delta {
                seq: 0,
                text: "Amiya is".into(),
            },
        );
        tokio::time::sleep(Duration::from_secs(1)).await;
        rt.trevor
            .finish(job, StreamEvent::Done(Box::new(done_view(job))));
    });
    let started = tokio::time::Instant::now();
    let mut body = resp.into_body().into_data_stream();
    let mut text = String::new();
    while let Some(chunk) = body.next().await {
        text.push_str(std::str::from_utf8(&chunk.expect("chunk")).expect("utf-8"));
    }
    assert!(started.elapsed() >= Duration::from_secs(41));
    assert_eq!(text.matches(": keepalive\n\n").count(), 2, "{text}");
    let delta = text
        .find("event: delta\ndata: {\"seq\":0,\"text\":\"Amiya is\"}")
        .expect("delta");
    let done = text.find("event: done\ndata: {").expect("done");
    assert!(delta < done);
    assert!(text.contains(&format!("\"jobId\":\"{job}\"")));
    assert_eq!(s.trevor.subscribers(job), 0);
}

#[tokio::test]
async fn a_finished_job_streams_its_terminal_event_at_once() {
    let s = state();
    let job = Uuid::new_v4();
    let rx = s.trevor.subscribe(job);
    let events: Vec<_> =
        stream::events(Some(StreamEvent::Error("expired in the queue".into())), rx)
            .collect()
            .await;
    assert_eq!(events.len(), 1);
    let resp = stream::sse(
        stream::events(
            Some(StreamEvent::Done(Box::new(done_view(job)))),
            s.trevor.subscribe(job),
        ),
        Duration::from_secs(15),
    );
    let body = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap();
    let text = String::from_utf8(body.to_vec()).unwrap();
    assert!(text.starts_with("event: done\n"), "{text}");
}

#[tokio::test]
async fn deltas_fan_out_to_every_subscriber_in_order() {
    let s = state();
    let job = Uuid::new_v4();
    let a = stream::events(None, s.trevor.subscribe(job));
    let b = stream::events(None, s.trevor.subscribe(job));
    assert_eq!(s.trevor.subscribers(job), 2);
    for seq in 0..3 {
        s.trevor.publish(
            job,
            StreamEvent::Delta {
                seq,
                text: format!("t{seq}"),
            },
        );
    }
    s.trevor
        .finish(job, StreamEvent::Error("the worker failed".into()));
    let (a, b): (Vec<_>, Vec<_>) = tokio::join!(a.collect(), b.collect());
    assert_eq!(a.len(), 4);
    assert_eq!(b.len(), 4);
    // A publish to a job nobody streams is dropped, not stored.
    s.trevor.publish(
        Uuid::new_v4(),
        StreamEvent::Delta {
            seq: 0,
            text: "x".into(),
        },
    );
}

/// Heartbeats through the server's real rate limiter from one non-loopback
/// address; the statuses seen.
async fn through_limiter(s: &AppState, ip: &str, key: &str, n: usize) -> Vec<StatusCode> {
    let app = backend::app::server::api_parts()
        .0
        .layer(axum::middleware::from_fn_with_state(
            s.clone(),
            backend::app::middleware::rate_limit,
        ))
        .with_state(s.clone());
    let peer: std::net::SocketAddr = format!("{ip}:40000").parse().unwrap();
    let mut out = Vec::new();
    for _ in 0..n {
        let mut req = post_json(
            "/api/trevor/worker/heartbeat",
            &[("x-trevor-worker-key", key)],
            &json!({ "workerId": "" }),
        );
        req.extensions_mut()
            .insert(axum::extract::ConnectInfo(peer));
        out.push(call(&app, req).await.status());
    }
    out
}

/// How many of 120 calls from `ip` the limiter refuses. The limiter counts fixed 60 s windows, so a burst that straddles
/// a window boundary is refused less; one retry on a fresh address (`retry_ip`) makes that a non-event.
async fn limited_of(s: &AppState, ip: &str, retry_ip: &str, key: &str) -> usize {
    let count = |v: Vec<StatusCode>| {
        v.iter()
            .filter(|c| **c == StatusCode::TOO_MANY_REQUESTS)
            .count()
    };
    let first = count(through_limiter(s, ip, key, 120).await);
    if first == 20 {
        return first;
    }
    count(through_limiter(s, retry_ip, key, 120).await)
}

#[tokio::test]
async fn a_valid_worker_key_skips_the_rate_limiter_and_an_invalid_one_does_not() {
    let s = state();
    // 120 calls in a minute from one remote address, past the default 100.
    let good = through_limiter(&s, "203.0.113.50", WORKER_KEY, 120).await;
    assert!(
        good.iter().all(|c| *c == StatusCode::BAD_REQUEST),
        "{good:?}"
    );
    assert_eq!(
        limited_of(&s, "203.0.113.51", "203.0.113.61", "not-the-key").await,
        20,
        "the 101st to 120th are refused"
    );
    // TREVOR_SCOPED_WORKER_KEY=0: the worker key is no bypass either.
    with_config(&s, |c| c.scoped_worker_key = false);
    assert_eq!(
        limited_of(&s, "203.0.113.52", "203.0.113.62", WORKER_KEY).await,
        20
    );
}
