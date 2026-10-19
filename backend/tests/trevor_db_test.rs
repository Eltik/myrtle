//! Trevor's queue against a real Postgres: a throwaway database
//! `trevor_test` on the local server (created when absent; its `public`
//! schema is dropped and rebuilt by the migrations on every run). It never
//! touches the `postgres` database's data: it only asks the server there
//! whether `trevor_test` exists.
//!
//! Covered: a chunked gzip publish and its activation (manifest kept across
//! chunks, missing gatePos filled from `storyOrder`, panel `Cache-Control`),
//! the horizon from the reader's profile and its era bucket, the twin join,
//! the per-user cap, rung selection against the heartbeat, the Neuron
//! ledger and the CPU queue, the ledger itself, the answer cache, SKIP LOCKED
//! with concurrent claimers, the Notify wake-up and the idle claim count
//! against the old 1 s poll, stale requeue then fail, the run ceiling on a
//! job a live worker keeps naming as busy, queue TTL expiry, the
//! SSE fan-out of worker deltas, `/trevor/status`, per-rung ETAs and the
//! heartbeat row, and the horizon of an account that holds only stage
//! progress (its read verdict re-derived on ask, and not under
//! `TREVOR_REVERDICT_ON_ASK=0`), and spoiler mode (its own cache bucket,
//! twin join and popular count, and `TREVOR_SPOILER_MODE=0`), and the bank
//! import (bank answers sent with the activating publish served as cache
//! hits, the era rule, the overwrite rule, a changed chunk sha, a failed
//! import that leaves the publish standing, and `TREVOR_BANK_IMPORT=0`).
//!
//! Ignored by default (CI has no Postgres):
//! `cargo test --test trevor_db_test -- --ignored --nocapture`

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::error::ApiError;
use backend::app::services::trevor::runtime::{StreamEvent, TrevorConfig};
use backend::app::services::trevor::worker::{self, Heartbeat, WorkerHello};
use backend::app::services::trevor::{
    AskRequest, AskResponse, JobStatus, NO_LIMIT_ERA, Profile, Rung, SpoilerMode, alerts,
    ask as svc_ask, stream,
};
use backend::app::state::{AppState, ServerData};
use backend::core::gamedata::types::GameData;
use backend::core::gamedata::types::story_review::StoryReviewGroup;
use backend::core::hypergryph::constants::Server;
use backend::database::queries::trevor as q;
use futures_util::StreamExt;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

#[path = "common/trevor.rs"]
mod trevor_common;
use trevor_common::{
    RecordingSink, app, app_config, app_state, fresh_db, gzip, insert_user, set_progress,
};

const WORKER_KEY: &str = "db-test-worker-key-0123456789";

/// Every contract v1.1 switch at its off value (the v1 behaviour).
fn v11_switches_off(c: &mut TrevorConfig) {
    c.normq = false;
    c.admission = false;
    c.fair_claim = false;
    c.claim_by_rung = false;
    c.cpu_profile = Profile::Slim;
    c.carry_over = false;
    c.ask_log = false;
    c.alerts = false;
}

fn state_on(pool: &PgPool, tweak: impl FnOnce(&mut TrevorConfig)) -> AppState {
    state_with(pool, HashMap::new(), tweak)
}

fn state_with(
    pool: &PgPool,
    server_data: HashMap<Server, Arc<ServerData>>,
    tweak: impl FnOnce(&mut TrevorConfig),
) -> AppState {
    let config = app_config("db-test-jwt-secret-0123456789abcdef", "db-test-service-key");
    let s = app_state(pool.clone(), server_data, config, WORKER_KEY);
    let mut cfg = TrevorConfig {
        worker_key: WORKER_KEY.into(),
        ..TrevorConfig::default()
    };
    // The v1 steps run with every contract v1.1 switch off.
    v11_switches_off(&mut cfg);
    tweak(&mut cfg);
    s.trevor.set_config(cfg);
    s
}

/// A state on the contract v1.1 defaults (every new switch on), alerts
/// recorded instead of posted.
fn state_with_recorder(
    pool: &PgPool,
    tweak: impl FnOnce(&mut TrevorConfig),
) -> (AppState, Arc<RecordingSink>) {
    let s = state_with(pool, HashMap::new(), |c| {
        *c = TrevorConfig {
            worker_key: WORKER_KEY.into(),
            ..TrevorConfig::default()
        };
        tweak(c);
    });
    let rec = Arc::new(RecordingSink::default());
    s.trevor.set_sink(rec.clone());
    (s, rec)
}

async fn new_user(pool: &PgPool) -> Uuid {
    insert_user(pool, "tdb").await.0
}

async fn game_read(pool: &PgPool, user: Uuid, story: &str) {
    sqlx::query(
        "INSERT INTO user_game_story_read (user_id, story_id, read_in_game) VALUES ($1, $2, true)",
    )
    .bind(user)
    .bind(story)
    .execute(pool)
    .await
    .expect("game read");
}

fn ask_req(question: &str, horizon_story: Option<&str>) -> AskRequest {
    serde_json::from_value(json!({ "question": question, "horizonStory": horizon_story }))
        .expect("ask")
}

async fn ask(
    s: &AppState,
    user: Uuid,
    question: &str,
    horizon_story: Option<&str>,
) -> Result<AskResponse, ApiError> {
    svc_ask::ask(s, user, None, ask_req(question, horizon_story)).await
}

async fn job(s: &AppState, id: Uuid) -> q::JobRow {
    q::job_by_id(&s.db, id).await.expect("query").expect("job")
}

async fn beat(s: &AppState, worker_id: &str, cf: bool, cpu: bool) {
    let hb: Heartbeat = serde_json::from_value(json!({
        "workerId": worker_id, "rungs": { "cf": cf, "cpu": cpu },
        "corpusVersion": "v1", "busyJobId": null
    }))
    .expect("heartbeat");
    worker::heartbeat(s, &hb).await.expect("heartbeat");
}

async fn set_neurons(pool: &PgPool, n: i32) {
    sqlx::query(
        "INSERT INTO trevor_neurons (day, neurons) VALUES ((now() AT TIME ZONE 'utc')::date, $1) \
         ON CONFLICT (day) DO UPDATE SET neurons = EXCLUDED.neurons",
    )
    .bind(n)
    .execute(pool)
    .await
    .expect("neurons");
}

async fn neurons_today(pool: &PgPool) -> i64 {
    q::neurons_today(pool).await.expect("neurons")
}

/// Empty the queue-side tables; the published version stays.
async fn reset_queue(pool: &PgPool) {
    sqlx::raw_sql(
        "TRUNCATE trevor_asks, trevor_feedback, trevor_jobs, trevor_answers, trevor_workers, trevor_neurons",
    )
    .execute(pool)
    .await
    .expect("truncate");
}

async fn next(s: &AppState, worker_id: &str) -> Option<worker::WorkerJob> {
    let hello: WorkerHello =
        serde_json::from_value(json!({ "workerId": worker_id })).expect("hello");
    worker::next(s, hello).await.expect("next")
}

fn result_json(job_id: Uuid, worker_id: &str, rung: &str, tokens: (u64, u64)) -> Vec<u8> {
    let long = format!("{}\n{}", "a".repeat(500), "b".repeat(200));
    json!({
        "jobId": job_id, "workerId": worker_id, "ok": true,
        "result": {
            "answer": "Amiya leads Rhodes Island [1].", "rung": rung,
            "profile": if rung == "cpu" { "slim" } else { "full" },
            "source": if rung == "passages" { "passages" } else { "live" },
            "citations": [{"n": 1, "storyId": "s1", "chunkId": "s1#0", "lineStart": 3,
                           "lineEnd": 9, "text": long}],
            "route": "who", "form": "short", "corpusVersion": "v1",
            "tokensIn": tokens.0, "tokensOut": tokens.1,
            "timings": {"routeMs": 5, "answerMs": 900, "totalMs": 910},
            "passages": [{"text": "the whole retrieved context, which is dropped"}]
        }
    })
    .to_string()
    .into_bytes()
}

async fn post_result(s: &AppState, body: &[u8]) {
    match worker::result(s, body).await {
        Ok(()) => {}
        Err(worker::ResultError::Api(e)) => panic!("result refused: {e}"),
        Err(worker::ResultError::TooLarge(n)) => panic!("result over {n}"),
    }
}

// --- publish ----------------------------------------------------------------

async fn send(app: &Router, req: Request<Body>) -> (StatusCode, axum::http::HeaderMap, Vec<u8>) {
    let resp = app.clone().oneshot(req).await.expect("infallible");
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = axum::body::to_bytes(resp.into_body(), 64 << 20)
        .await
        .expect("body")
        .to_vec();
    (status, headers, body)
}

async fn publish(app: &Router, body: &Value) -> StatusCode {
    let gz = gzip(body.to_string().into_bytes()).await;
    let req = Request::post("/api/trevor/versions")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_ENCODING, "gzip")
        .header("x-trevor-worker-key", WORKER_KEY)
        .body(Body::from(gz))
        .unwrap();
    send(app, req).await.0
}

fn panel(story: &str, position: Option<i32>) -> Value {
    let mut p = json!({ "schema": 1, "storyId": story, "recap": { "text": "x".repeat(2000) } });
    if let Some(pos) = position {
        p["position"] = json!(pos);
    }
    p
}

fn eras() -> Value {
    json!([
        { "id": "e1", "name": "Era One", "position": 1 },
        { "id": "e2", "name": "Era Two", "position": 4 }
    ])
}

fn story_order() -> Value {
    json!(["s1", "s2", "s3", "s4", "s5", "s6"])
}

async fn step_publish(s: &AppState) {
    let app = app(s);
    let m0 = json!({ "storyOrder": story_order(), "eras": eras() });
    let chunk1 = json!({
        "version": "v1", "activate": false, "manifest": m0.clone(),
        "panels": { "s1": panel("s1", Some(1)), "s2": panel("s2", Some(2)), "s3": panel("s3", Some(3)) }
    });
    assert_eq!(publish(&app, &chunk1).await, StatusCode::OK);
    let chunk2 = json!({
        "version": "v1", "activate": false,
        "panels": { "s4": panel("s4", Some(4)), "s5": panel("s5", None), "s6": panel("s6", Some(6)) }
    });
    assert_eq!(publish(&app, &chunk2).await, StatusCode::OK);
    let kept: Option<Value> =
        sqlx::query_scalar("SELECT manifest FROM trevor_versions WHERE version = 'v1'")
            .fetch_one(&s.db)
            .await
            .unwrap();
    assert_eq!(
        kept,
        Some(m0),
        "a chunk without a manifest keeps the stored one"
    );
    assert_eq!(q::active_version(&s.db, "en").await.unwrap(), None);

    // Activation must resend the manifest and the gate report.
    let bare = json!({ "version": "v1", "activate": true });
    assert_eq!(publish(&app, &bare).await, StatusCode::BAD_REQUEST);
    let activate = json!({
        "version": "v1", "activate": true, "gateReport": { "spoilerLeaks": 0 },
        "manifest": { "builtFrom": "26-10-01-res", "storyOrder": story_order(), "eras": eras() }
    });
    assert_eq!(publish(&app, &activate).await, StatusCode::OK);
    assert_eq!(
        q::active_version(&s.db, "en").await.unwrap().as_deref(),
        Some("v1")
    );
    let (built_from, report): (Option<String>, Option<Value>) = sqlx::query_as(
        "SELECT manifest ->> 'builtFrom', gate_report FROM trevor_versions WHERE version = 'v1'",
    )
    .fetch_one(&s.db)
    .await
    .unwrap();
    assert_eq!(built_from.as_deref(), Some("26-10-01-res"));
    assert_eq!(report, Some(json!({ "spoilerLeaks": 0 })));
    let positions: Vec<(String, Option<i32>)> = sqlx::query_as(
        "SELECT story_id, gate_pos FROM trevor_panels WHERE version = 'v1' ORDER BY story_id",
    )
    .fetch_all(&s.db)
    .await
    .unwrap();
    assert_eq!(
        positions,
        (1..=6)
            .map(|i| (format!("s{i}"), Some(i)))
            .collect::<Vec<_>>(),
        "s5's missing position comes from storyOrder"
    );

    // Panels: immutable for the active version, short otherwise, ETag 304.
    let get = |uri: &str, inm: Option<&str>| {
        let mut b = Request::get(uri);
        if let Some(etag) = inm {
            b = b.header(header::IF_NONE_MATCH, etag);
        }
        b.body(Body::empty()).unwrap()
    };
    let (st, h, body) = send(&app, get("/api/trevor/panels/s2?v=v1", None)).await;
    assert_eq!(st, StatusCode::OK);
    assert_eq!(
        h.get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=86400, immutable"
    );
    let v: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["version"], "v1");
    assert_eq!(v["panels"]["position"], 2);
    let etag = h.get(header::ETAG).unwrap().to_str().unwrap().to_owned();
    let (st, h, body) = send(&app, get("/api/trevor/panels/s2?v=v1", Some(&etag))).await;
    assert_eq!(st, StatusCode::NOT_MODIFIED);
    assert!(body.is_empty());
    assert_eq!(
        h.get(header::CACHE_CONTROL).unwrap(),
        "public, max-age=86400, immutable"
    );
    let (_, h, _) = send(&app, get("/api/trevor/panels/s2?v=v0", None)).await;
    assert_eq!(h.get(header::CACHE_CONTROL).unwrap(), "public, max-age=300");
    let (_, h, _) = send(&app, get("/api/trevor/panels/s2", None)).await;
    assert_eq!(h.get(header::CACHE_CONTROL).unwrap(), "public, max-age=300");
    let (st, _, _) = send(&app, get("/api/trevor/panels/nope", None)).await;
    assert_eq!(st, StatusCode::NOT_FOUND);
    eprintln!("publish: 2 gzip chunks + activation, 6 panels, gatePos 1..6, panels headers ok");
}

// --- horizon, twin join, per-user cap -----------------------------------------

async fn step_horizon_and_twins(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    let a = new_user(pool).await;
    set_progress(pool, a, &["s1", "s3"], &[]).await;
    let ra = ask(s, a, "Who is Amiya?", None).await.expect("ask a");
    assert_eq!(ra.status, JobStatus::Queued);
    let sa = ra.spoiler_safe_up_to.clone().expect("era");
    assert_eq!((sa.era_id.as_str(), sa.position), ("e1", 1));
    let ja = job(s, ra.job_id.unwrap()).await;
    assert_eq!(ja.horizon_pos, Some(1), "gatePos 3 buckets to era e1 at 1");
    assert_eq!(
        ra.stream_url.as_deref(),
        Some(format!("/api/trevor/jobs/{}/stream", ja.id).as_str())
    );

    // B read up to gatePos 5 -> era e2 (4); a different bucket, a new job.
    let b = new_user(pool).await;
    set_progress(pool, b, &["s5"], &[]).await;
    let rb = ask(s, b, "who is amiya", None).await.expect("ask b");
    assert_eq!(rb.spoiler_safe_up_to.as_ref().unwrap().era_id, "e2");
    assert_eq!(job(s, rb.job_id.unwrap()).await.horizon_pos, Some(4));
    assert_ne!(rb.job_id, ra.job_id);

    // B reading s2 now: the horizon drops to 2 -> e1, the same bucket as A:
    // B joins A's job (the twin join) and may stream it.
    let rb2 = ask(s, b, "Who is Amiya", Some("s2"))
        .await
        .expect("ask b at s2");
    assert_eq!(rb2.job_id, ra.job_id, "joined the open identical question");
    assert!(svc_ask::may_stream(s, &ja, b));
    let stranger = new_user(pool).await;
    assert!(!svc_ask::may_stream(s, &ja, stranger));

    // A request story never raises the horizon: B at s6 stays at 5 -> e2.
    let rb3 = ask(s, b, "Who is Kal'tsit?", Some("s6"))
        .await
        .expect("ask b at s6");
    assert_eq!(job(s, rb3.job_id.unwrap()).await.horizon_pos, Some(4));

    // No progress: nothing released.
    let c = new_user(pool).await;
    let rc = ask(s, c, "Who is Amiya?", Some("s6")).await.expect("ask c");
    assert_eq!(rc.spoiler_safe_up_to, None);
    assert_eq!(job(s, rc.job_id.unwrap()).await.horizon_pos, Some(0));
    assert!(matches!(
        ask(s, c, "Who is W?", Some("not-a-story")).await,
        Err(ApiError::BadRequest(_))
    ));

    // The game's read flags count; the site's "unread" marks remove them.
    let d = new_user(pool).await;
    game_read(pool, d, "s6").await;
    let rd = ask(s, d, "Who is Theresa?", None).await.expect("ask d");
    assert_eq!(job(s, rd.job_id.unwrap()).await.horizon_pos, Some(4));
    let e = new_user(pool).await;
    game_read(pool, e, "s6").await;
    set_progress(pool, e, &["s1"], &["s6"]).await;
    let re = ask(s, e, "Who is Theresa?", None).await.expect("ask e");
    assert_eq!(job(s, re.job_id.unwrap()).await.horizon_pos, Some(1));

    // TREVOR_ERA_BUCKETS=0: the exact gatePos keys the job.
    let exact = state_on(pool, |c| c.era_buckets = false);
    let rx = ask(&exact, b, "Who is Ch'en?", None).await.expect("exact");
    assert_eq!(job(s, rx.job_id.unwrap()).await.horizon_pos, Some(5));
    assert_eq!(rx.spoiler_safe_up_to.unwrap().era_id, "e2");

    // Three open questions per user; the fourth waits its turn.
    let f = new_user(pool).await;
    for i in 0..3 {
        ask(s, f, &format!("question {i}"), None)
            .await
            .expect("open");
    }
    assert!(matches!(
        ask(s, f, "question 3", None).await,
        Err(ApiError::Conflict(_))
    ));
    eprintln!("horizon: profile/era/twin/cap ok");
}

// --- rungs and the Neuron ledger ------------------------------------------------

async fn step_rungs(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    let u = new_user(pool).await;
    let v = new_user(pool).await;
    let w = new_user(pool).await;
    beat(s, "w1", true, true).await;

    let r = ask(s, u, "rung one", None).await.unwrap();
    assert_eq!(r.rung, Rung::Cf);
    assert_eq!(job(s, r.job_id.unwrap()).await.profile, "full");

    set_neurons(pool, 7999).await;
    assert_eq!(ask(s, u, "rung two", None).await.unwrap().rung, Rung::Cf);
    set_neurons(pool, 8000).await;
    let r3 = ask(s, u, "rung three", None).await.unwrap();
    assert_eq!(r3.rung, Rung::Cpu, "past 9000 - 1000 Neurons cf is off");
    assert_eq!(job(s, r3.job_id.unwrap()).await.profile, "slim");

    // Six live CPU jobs: the seventh question gets passages.
    for i in 0..3 {
        assert_eq!(
            ask(s, v, &format!("cpu {i}"), None).await.unwrap().rung,
            Rung::Cpu
        );
    }
    for i in 0..2 {
        assert_eq!(
            ask(s, w, &format!("cpu w {i}"), None).await.unwrap().rung,
            Rung::Cpu
        );
    }
    let x = new_user(pool).await;
    assert_eq!(
        ask(s, x, "the seventh", None).await.unwrap().rung,
        Rung::Passages
    );

    // A heartbeat with no model rung: passages.
    reset_queue(pool).await;
    beat(s, "w1", false, false).await;
    assert_eq!(
        ask(s, x, "no model", None).await.unwrap().rung,
        Rung::Passages
    );

    // Claim-time settle: queued as cf, the budget ran out while it waited.
    reset_queue(pool).await;
    beat(s, "w1", true, true).await;
    let r = ask(s, u, "settle me", None).await.unwrap();
    assert_eq!(r.rung, Rung::Cf);
    set_neurons(pool, 8000).await;
    let claimed = next(s, "w1").await.expect("claimed");
    assert_eq!(claimed.job_id, r.job_id.unwrap());
    assert_eq!((claimed.rung, claimed.profile), (Rung::Cpu, Profile::Slim));
    assert_eq!(claimed.horizon_pos, Some(0));
    let row = job(s, claimed.job_id).await;
    assert_eq!((row.rung.as_str(), row.profile.as_str()), ("cpu", "slim"));
    eprintln!("rungs: cf/cpu/passages selection and claim-time settle ok");
}

async fn step_ledger_and_cache(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    let u = new_user(pool).await;
    beat(s, "w1", true, true).await;
    let r = ask(s, u, "Who leads Rhodes Island?", None).await.unwrap();
    assert_eq!(r.rung, Rung::Cf);
    let claimed = next(s, "w1").await.expect("claim");
    assert_eq!(claimed.rung, Rung::Cf);
    post_result(s, &result_json(claimed.job_id, "w1", "cf", (2000, 250))).await;
    assert_eq!(neurons_today(pool).await, 26, "ceil(18.182 + 6.81825)");

    let done = job(s, claimed.job_id).await;
    assert_eq!(done.status, "done");
    let answer = q::answer_by_id(pool, done.answer_id.unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (answer.rung.as_str(), answer.source.as_str()),
        ("cf", "live")
    );
    assert_eq!(answer.era_pos, 0);
    assert!(answer.answer.get("passages").is_none(), "passages dropped");
    assert_eq!(answer.answer["tokensIn"], 2000);
    assert_eq!(answer.answer["tokensOut"], 250);
    let text = answer.answer["citations"][0]["text"].as_str().unwrap();
    assert_eq!(
        text,
        "a".repeat(500),
        "trimmed at the line break inside 600"
    );

    // A CPU answer books nothing.
    let r2 = ask(s, u, "Who is Kal'tsit?", None).await.unwrap();
    set_neurons(pool, 26).await;
    sqlx::query("UPDATE trevor_jobs SET rung = 'cpu', profile = 'slim' WHERE id = $1")
        .bind(r2.job_id.unwrap())
        .execute(pool)
        .await
        .unwrap();
    beat(s, "w1", false, true).await;
    let c2 = next(s, "w1").await.expect("claim 2");
    assert_eq!(c2.rung, Rung::Cpu);
    post_result(s, &result_json(c2.job_id, "w1", "cpu", (2000, 250))).await;
    assert_eq!(neurons_today(pool).await, 26);

    // The same question under the same era: from the cache.
    let hit = ask(s, u, "who leads rhodes island", None).await.unwrap();
    assert_eq!(hit.rung, Rung::Cache);
    assert_eq!(hit.status, JobStatus::Done);
    assert_eq!(hit.job_id, None);
    assert_eq!(hit.stream_url, None);
    let cached = hit.answer.expect("answer");
    assert_eq!(cached.id, answer.id);
    assert_eq!(cached.rung, Rung::Cf);
    let hits: i64 = sqlx::query_scalar("SELECT hits FROM trevor_answers WHERE id = $1")
        .bind(answer.id)
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(hits, 1);
    eprintln!("ledger: +26 Neurons for 2000/250 cf tokens, 0 for cpu; cache hit ok");
}

// --- the queue ------------------------------------------------------------------

/// Every v1.1 claim rule on, for a claimer with no heartbeat.
const RULES: q::ClaimRules = q::ClaimRules {
    admission: true,
    fair: true,
    by_rung: true,
    cf: false,
    cpu: false,
    online_secs: 75,
};

async fn step_skip_locked(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    let users: Vec<Uuid> = futures_util::future::join_all((0..7).map(|_| new_user(pool))).await;
    for (i, u) in users.iter().enumerate() {
        for k in 0..3 {
            ask(s, *u, &format!("skip locked {i} {k}"), None)
                .await
                .unwrap();
        }
    }
    let queued = q::queued_jobs(pool, "en").await.unwrap();
    assert_eq!(queued, 21);
    // Four claimers race on one queue until it is empty.
    let claimers = (0..4).map(|w| {
        let pool = pool.clone();
        async move {
            let mut mine = Vec::new();
            while let Some(job) = q::claim_next(&pool, "en", &format!("racer{w}"), RULES)
                .await
                .unwrap()
            {
                mine.push(job.id);
            }
            mine
        }
    });
    let claimed: Vec<Vec<Uuid>> = futures_util::future::join_all(claimers).await;
    let mut all: Vec<Uuid> = claimed.iter().flatten().copied().collect();
    let per: Vec<usize> = claimed.iter().map(Vec::len).collect();
    all.sort();
    let n = all.len();
    all.dedup();
    assert_eq!((n, all.len()), (21, 21), "every job claimed exactly once");
    eprintln!("skip locked: 21 jobs, 4 claimers, per-claimer {per:?}, 0 duplicates");

    // Two worker/next long-polls at once take two different jobs.
    reset_queue(pool).await;
    beat(s, "w1", true, true).await;
    let u = users[0];
    ask(s, u, "twin poll one", None).await.unwrap();
    ask(s, u, "twin poll two", None).await.unwrap();
    let (a, b) = tokio::join!(next(s, "w1"), next(s, "w2"));
    let (a, b) = (a.expect("a"), b.expect("b"));
    assert_ne!(a.job_id, b.job_id);
}

async fn step_notify(pool: &PgPool) {
    reset_queue(pool).await;
    let s = state_on(pool, |_| {});
    beat(&s, "w1", true, true).await;
    let u = new_user(pool).await;
    let poller = {
        let s = s.clone();
        tokio::spawn(async move {
            let t = Instant::now();
            let job = next(&s, "w1").await;
            (job, t.elapsed())
        })
    };
    tokio::time::sleep(Duration::from_millis(500)).await;
    let enqueued_at = Instant::now();
    let r = ask(&s, u, "wake the worker", None).await.unwrap();
    let (job, waited) = poller.await.unwrap();
    let woke_after = enqueued_at.elapsed();
    assert_eq!(job.expect("woken").job_id, r.job_id.unwrap());
    assert!(woke_after < Duration::from_secs(1), "{woke_after:?}");
    eprintln!("notify: poll held {waited:?}, answered {woke_after:?} after the enqueue");

    // Idle polls on an empty queue: the Notify poll issues one claim query,
    // the old poll (TREVOR_NOTIFY_POLL=0) one a second.
    let notify = state_on(pool, |_| {});
    let polling = state_on(pool, |c| c.notify_poll = false);
    let idle = |s: AppState| async move {
        let hello: WorkerHello =
            serde_json::from_value(json!({ "workerId": "idle", "server": "cn" })).unwrap();
        let t = Instant::now();
        let got = worker::next(&s, hello).await.unwrap();
        assert!(got.is_none());
        (
            s.trevor
                .claim_queries
                .load(std::sync::atomic::Ordering::Relaxed),
            t.elapsed(),
        )
    };
    let ((n_notify, t_notify), (n_poll, t_poll)) = tokio::join!(idle(notify), idle(polling));
    eprintln!(
        "idle 20 s poll: notify {n_notify} claim queries in {t_notify:?}; \
         TREVOR_NOTIFY_POLL=0 {n_poll} in {t_poll:?}"
    );
    assert_eq!(n_notify, 1);
    assert!(n_poll >= 15, "{n_poll}");
}

async fn age_running(pool: &PgPool, job: Uuid, secs: i32) {
    sqlx::query(
        "UPDATE trevor_jobs SET started_at = now() - make_interval(secs => $2) WHERE id = $1",
    )
    .bind(job)
    .bind(f64::from(secs))
    .execute(pool)
    .await
    .unwrap();
}

async fn step_stale_and_ttl(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    beat(s, "w1", true, true).await;
    let u = new_user(pool).await;
    let r = ask(s, u, "stale", None).await.unwrap();
    let id = r.job_id.unwrap();
    let mut rx = s.trevor.subscribe(id);

    assert_eq!(next(s, "w1").await.unwrap().job_id, id);
    age_running(pool, id, 299).await;
    worker::sweep(s).await.unwrap();
    assert_eq!(job(s, id).await.status, "running", "not stale at 299 s");
    age_running(pool, id, 301).await;
    worker::sweep(s).await.unwrap();
    let j = job(s, id).await;
    assert_eq!(
        (j.status.as_str(), j.attempts),
        ("queued", 1),
        "requeued once"
    );
    assert_eq!(j.worker_id, None);

    assert_eq!(next(s, "w2").await.unwrap().job_id, id);
    age_running(pool, id, 301).await;
    worker::sweep(s).await.unwrap();
    let j = job(s, id).await;
    assert_eq!((j.status.as_str(), j.attempts), ("failed", 2));
    assert_eq!(j.error.as_deref(), Some("the worker stopped answering"));
    match rx.recv().await.unwrap() {
        StreamEvent::Error(m) => assert_eq!(m, "the worker stopped answering"),
        other => panic!("expected an error event, got {other:?}"),
    }

    // Queue TTL (600 s): 599 s waits on, 601 s expires.
    let r1 = ask(s, u, "young", None).await.unwrap();
    let r2 = ask(s, u, "old", None).await.unwrap();
    for (job_id, age) in [(r1.job_id.unwrap(), 599.0), (r2.job_id.unwrap(), 601.0)] {
        sqlx::query(
            "UPDATE trevor_jobs SET created_at = now() - make_interval(secs => $2), queued_at = now() - make_interval(secs => $2) WHERE id = $1",
        )
        .bind(job_id)
        .bind(age)
        .execute(pool)
        .await
        .unwrap();
    }
    worker::sweep(s).await.unwrap();
    assert_eq!(job(s, r1.job_id.unwrap()).await.status, "queued");
    let old = job(s, r2.job_id.unwrap()).await;
    assert_eq!(old.status, "failed");
    assert_eq!(old.error.as_deref(), Some("expired in the queue"));
    eprintln!("sweep: requeue at 301 s, fail on the second, TTL 600 s ok");
}

// --- streaming ----------------------------------------------------------------

async fn read_sse(resp: axum::response::Response) -> String {
    let mut body = resp.into_body().into_data_stream();
    let mut text = String::new();
    while let Some(chunk) = tokio::time::timeout(Duration::from_secs(5), body.next())
        .await
        .expect("the stream ends")
    {
        text.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
    }
    text
}

async fn step_stream(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    beat(s, "w1", true, true).await;
    let owner = new_user(pool).await;
    let other = new_user(pool).await;
    let r = ask(s, owner, "stream me", None).await.unwrap();
    let id = r.job_id.unwrap();

    assert!(matches!(
        stream::open(s, other, id).await,
        Err(ApiError::Forbidden)
    ));
    assert!(matches!(
        stream::open(s, owner, Uuid::new_v4()).await,
        Err(ApiError::NotFound)
    ));
    let first = stream::open(s, owner, id).await.expect("open");
    let second = stream::open(s, owner, id).await.expect("open twice");
    assert_eq!(s.trevor.subscribers(id), 2);

    let delta = |worker_id: &str, seq: u64, text: &str| {
        serde_json::from_value::<worker::WorkerDelta>(
            json!({ "workerId": worker_id, "seq": seq, "text": text }),
        )
        .unwrap()
    };
    // Not running yet: 404.
    assert!(matches!(
        worker::delta(s, id, delta("w1", 0, "x")).await,
        Err(ApiError::NotFoundMessage(_))
    ));
    assert_eq!(next(s, "w1").await.unwrap().job_id, id);
    assert!(matches!(
        worker::delta(s, id, delta("w2", 0, "x")).await,
        Err(ApiError::NotFoundMessage(_))
    ));
    worker::delta(s, id, delta("w1", 0, "Amiya "))
        .await
        .unwrap();
    worker::delta(s, id, delta("w1", 1, "leads."))
        .await
        .unwrap();
    post_result(s, &result_json(id, "w1", "cf", (100, 10))).await;
    // Done: deltas are refused again, and the channel is gone.
    assert!(worker::delta(s, id, delta("w1", 2, "late")).await.is_err());

    for resp in [first, second] {
        let text = read_sse(resp).await;
        let d0 = text
            .find("event: delta\ndata: {\"seq\":0,\"text\":\"Amiya \"}")
            .expect("d0");
        let d1 = text
            .find("event: delta\ndata: {\"seq\":1,\"text\":\"leads.\"}")
            .expect("d1");
        let done = text.find("event: done\ndata: ").expect("done");
        assert!(d0 < d1 && d1 < done, "{text}");
        assert!(text.contains("\"status\":\"done\""));
        assert!(text.contains("Amiya leads Rhodes Island"));
    }
    // A finished job sends `done` at once and closes.
    let late = read_sse(stream::open(s, owner, id).await.unwrap()).await;
    assert!(late.starts_with("event: done\n"), "{late}");
    assert_eq!(late.matches("event:").count(), 1);
    eprintln!("stream: 2 readers got 2 deltas + done; late reader got done at once");
}

// --- status, ETA, heartbeat ---------------------------------------------------------

async fn step_status_eta_heartbeat(s: &AppState) {
    reset_queue(&s.db).await;
    let pool = &s.db;
    let app = app(s);
    let status = || async {
        let (st, h, body) = send(
            &app,
            Request::get("/api/trevor/status")
                .body(Body::empty())
                .unwrap(),
        )
        .await;
        assert_eq!(st, StatusCode::OK);
        assert_eq!(h.get(header::CACHE_CONTROL).unwrap(), "public, max-age=15");
        serde_json::from_slice::<Value>(&body).unwrap()
    };
    let v = status().await;
    assert_eq!(
        v,
        json!({ "online": false, "rungs": { "cf": false, "cpu": false }, "activeVersion": "v1",
                "queueDepth": { "cf": 0, "cpu": 0 }, "cfBudget": "ok",
                "workers": { "cf": 0, "cpu": 0 }, "guard": "ok", "mode": "on" })
    );
    beat(s, "w1", true, false).await;
    set_neurons(pool, 6401).await;
    let u = new_user(pool).await;
    ask(s, u, "depth", None).await.unwrap();
    let v = status().await;
    assert_eq!(v["online"], true);
    assert_eq!(v["rungs"], json!({ "cf": true, "cpu": false }));
    assert_eq!(v["queueDepth"], json!({ "cf": 1, "cpu": 0 }));
    assert_eq!(v["cfBudget"], "low");
    set_neurons(pool, 8000).await;
    assert_eq!(status().await["cfBudget"], "spent");

    // Heartbeat row: rungs, corpus version, busy job; online for 75 s.
    let busy = Uuid::new_v4();
    let hb: Heartbeat = serde_json::from_value(json!({
        "workerId": "w9", "rungs": { "cf": false, "cpu": true }, "corpusVersion": "v1",
        "busyJobId": busy.to_string()
    }))
    .unwrap();
    worker::heartbeat(s, &hb).await.unwrap();
    let row: (Value, Option<String>, Option<Uuid>) = sqlx::query_as(
        "SELECT rungs, corpus_version, busy_job_id FROM trevor_workers WHERE worker_id = 'w9'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        row,
        (
            json!({ "cf": false, "cpu": true }),
            Some("v1".into()),
            Some(busy)
        )
    );
    let bad: Heartbeat =
        serde_json::from_value(json!({ "workerId": "w9", "busyJobId": "nope" })).unwrap();
    assert!(matches!(
        worker::heartbeat(s, &bad).await,
        Err(ApiError::BadRequest(_))
    ));
    sqlx::query("UPDATE trevor_workers SET last_seen = now() - interval '76 seconds'")
        .execute(pool)
        .await
        .unwrap();
    let v = status().await;
    assert_eq!(v["online"], false);
    assert_eq!(v["rungs"], json!({ "cf": false, "cpu": false }));

    // ETA per rung: fewer than 5 finished CPU jobs -> the 120 s fallback;
    // five (10..50 s) -> their median, 30 s. Offline -> null.
    reset_queue(pool).await;
    set_neurons(pool, 0).await;
    let insert_done = |secs: i32| async move {
        sqlx::query(
            "INSERT INTO trevor_jobs (question, qkey, server, rung, profile, status, started_at, finished_at) \
             VALUES ('q', 'k', 'en', 'cpu', 'slim', 'done', now() - make_interval(secs => $1), now())",
        )
        .bind(f64::from(secs))
        .execute(pool)
        .await
        .unwrap();
    };
    for secs in [10, 20, 30, 40] {
        insert_done(secs).await;
    }
    let offline = ask(s, u, "eta offline", None).await.unwrap();
    assert_eq!(offline.eta_secs, None, "no worker online");
    beat(s, "w1", false, true).await;
    let four = ask(s, u, "eta four", None).await.unwrap();
    assert_eq!(four.rung, Rung::Cpu);
    // Two queued CPU jobs up to this one ("eta offline" went passages before
    // any heartbeat): 1 s + 120 s.
    assert_eq!(offline.rung, Rung::Passages);
    assert_eq!(four.eta_secs, Some(1 + 120));
    insert_done(50).await;
    let jv = svc_ask::job(s, u, four.job_id.unwrap()).await.unwrap();
    assert_eq!(jv.eta_secs, Some(1 + 30), "median of 10..50 s");
    eprintln!("status/eta/heartbeat ok: eta 121 s on the fallback, 31 s on the median");
}

#[tokio::test]
#[ignore = "needs the local Postgres; creates and resets the throwaway database trevor_test"]
async fn trevor_queue_against_a_throwaway_database() {
    let pool = fresh_db().await;
    // The v1 steps run with every contract v1.1 switch off: they pass
    // unchanged, which is the proof each off value restores v1.
    let s = state_on(&pool, v11_switches_off);
    step_publish(&s).await;
    step_horizon_and_twins(&s).await;
    step_rungs(&s).await;
    step_ledger_and_cache(&s).await;
    step_skip_locked(&s).await;
    step_stale_and_ttl(&s).await;
    step_stream(&s).await;
    step_status_eta_heartbeat(&s).await;
    step_review_fixes(&s).await;
    step_run_ceiling(&s).await;
    step_notify(&pool).await;
    step_v11_normq(&pool).await;
    step_v11_admission(&pool).await;
    step_v11_fair(&pool).await;
    step_v11_mode_status_alerts(&pool).await;
    step_spoiler_mode(&pool).await;
    // Activates its own versions.
    step_v11_lists_and_carry(&pool).await;
    // Activates its own versions too.
    step_bank_import(&pool).await;
    // Last: it activates its own version.
    step_reverdict_on_ask(&pool).await;
}

/// A result body with a chosen source and corpus version.
fn result_with(job_id: Uuid, worker_id: &str, rung: &str, source: &str, version: &str) -> Vec<u8> {
    let mut v: Value =
        serde_json::from_slice(&result_json(job_id, worker_id, rung, (2000, 250))).unwrap();
    v["result"]["source"] = json!(source);
    v["result"]["corpusVersion"] = json!(version);
    v.to_string().into_bytes()
}

async fn cached_flag(pool: &PgPool, job_id: Uuid) -> bool {
    sqlx::query_scalar(
        "SELECT a.cached FROM trevor_jobs j JOIN trevor_answers a ON a.id = j.answer_id WHERE j.id = $1",
    )
    .bind(job_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// The review's items 2 to 7 (item 1 is in `trevor_handler_test.rs`).
async fn step_review_fixes(s: &AppState) {
    let pool = &s.db;

    // 2. A passages answer is stored for its job but never served from the
    // cache, and a later live answer takes the key.
    reset_queue(pool).await;
    beat(s, "w1", false, false).await;
    let u = new_user(pool).await;
    let r = ask(s, u, "passages only", None).await.unwrap();
    assert_eq!(r.rung, Rung::Passages);
    let c = next(s, "w1").await.unwrap();
    post_result(
        s,
        &result_with(c.job_id, "w1", "passages", "passages", "v1"),
    )
    .await;
    assert!(!cached_flag(pool, c.job_id).await);
    let again = ask(s, u, "passages only", None).await.unwrap();
    assert_ne!(
        again.rung,
        Rung::Cache,
        "a passages answer is not a cache hit"
    );
    beat(s, "w1", false, true).await;
    let c2 = next(s, "w1").await.unwrap();
    assert_eq!(Some(c2.job_id), again.job_id);
    post_result(s, &result_with(c2.job_id, "w1", "passages", "live", "v1")).await;
    assert!(cached_flag(pool, c2.job_id).await);
    assert_eq!(
        ask(s, u, "passages only", None).await.unwrap().rung,
        Rung::Cache
    );
    // TREVOR_CACHE_LIVE_ONLY=0: passages answers are cached again.
    let old = state_on(pool, |c| c.cache_live_only = false);
    let r = ask(&old, u, "old passages", None).await.unwrap();
    let c3 = next(&old, "w1").await.unwrap();
    assert_eq!(Some(c3.job_id), r.job_id);
    post_result(
        &old,
        &result_with(c3.job_id, "w1", "passages", "passages", "v1"),
    )
    .await;
    assert!(cached_flag(pool, c3.job_id).await);

    // 6. An answer from another corpus version than the job's: stored, not cached.
    let r = ask(s, u, "version drift", None).await.unwrap();
    let c4 = next(s, "w1").await.unwrap();
    assert_eq!(Some(c4.job_id), r.job_id);
    assert_eq!(
        job(s, c4.job_id).await.corpus_version.as_deref(),
        Some("v1")
    );
    post_result(s, &result_with(c4.job_id, "w1", "cpu", "live", "v0")).await;
    assert!(!cached_flag(pool, c4.job_id).await);

    // 3. A requeue restarts the TTL clock (queued_at), not created_at.
    reset_queue(pool).await;
    beat(s, "w1", true, true).await;
    let r = ask(s, u, "requeue ttl", None).await.unwrap();
    let id = r.job_id.unwrap();
    next(s, "w1").await.unwrap();
    sqlx::query("UPDATE trevor_jobs SET created_at = now() - interval '700 seconds', queued_at = now() - interval '700 seconds' WHERE id = $1")
        .bind(id).execute(pool).await.unwrap();
    age_running(pool, id, 301).await;
    worker::sweep(s).await.unwrap();
    assert_eq!(
        job(s, id).await.status,
        "queued",
        "requeued, and its TTL restarted"
    );
    // TREVOR_TTL_FROM_QUEUED=0: the created_at clock expires it.
    let old = state_on(pool, |c| c.ttl_from_queued = false);
    worker::sweep(&old).await.unwrap();
    assert_eq!(
        job(s, id).await.error.as_deref(),
        Some("expired in the queue")
    );

    // 4. A job an online worker names as busy is never requeued; the switch
    // restores the age-only rule. A late cf result still books its Neurons.
    reset_queue(pool).await;
    beat(s, "w1", true, true).await;
    let r = ask(s, u, "long answer", None).await.unwrap();
    let id = r.job_id.unwrap();
    assert_eq!(next(s, "w1").await.unwrap().rung, Rung::Cf);
    let busy: Heartbeat = serde_json::from_value(json!({
        "workerId": "w1", "rungs": { "cf": true, "cpu": true }, "busyJobId": id.to_string()
    }))
    .unwrap();
    worker::heartbeat(s, &busy).await.unwrap();
    age_running(pool, id, 1799).await;
    worker::sweep(s).await.unwrap();
    assert_eq!(
        job(s, id).await.status,
        "running",
        "busy on an online worker, under the 1800 s ceiling"
    );
    // TREVOR_JOB_MAX_RUN_SECS=0: no ceiling, the busy heartbeat protects it
    // at any age (the rule before the ceiling).
    let no_ceiling = state_on(pool, |c| c.job_max_run_secs = 0);
    age_running(pool, id, 3600).await;
    worker::sweep(&no_ceiling).await.unwrap();
    assert_eq!(job(s, id).await.status, "running", "no ceiling at 3600 s");
    let old = state_on(pool, |c| c.busy_protects = false);
    worker::sweep(&old).await.unwrap();
    assert_eq!(job(s, id).await.status, "queued");
    let before = neurons_today(pool).await;
    let late = worker::result(s, &result_json(id, "w1", "cf", (2000, 250))).await;
    assert!(matches!(
        late,
        Err(worker::ResultError::Api(ApiError::Conflict(_)))
    ));
    assert_eq!(
        neurons_today(pool).await,
        before + 26,
        "late cf Neurons booked"
    );

    // 5. Two identical asks at once from two users: one job.
    reset_queue(pool).await;
    let (a, b) = (new_user(pool).await, new_user(pool).await);
    let (ra, rb) = tokio::join!(
        ask(s, a, "Concurrent twin?", None),
        ask(s, b, "concurrent twin", None)
    );
    let (ra, rb) = (ra.unwrap(), rb.unwrap());
    assert_eq!(ra.job_id, rb.job_id);
    let n: i64 = sqlx::query_scalar("SELECT count(*) FROM trevor_jobs")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_eq!(n, 1);

    // 7. GET /trevor/jobs/{id}: the asker and a joiner, nobody else.
    let id = ra.job_id.unwrap();
    let owner =
        sqlx::query_scalar::<_, Option<Uuid>>("SELECT user_id FROM trevor_jobs WHERE id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap()
            .unwrap();
    let joiner = if owner == a { b } else { a };
    assert!(svc_ask::job(s, owner, id).await.is_ok());
    assert!(svc_ask::job(s, joiner, id).await.is_ok());
    let stranger = new_user(pool).await;
    assert!(matches!(
        svc_ask::job(s, stranger, id).await,
        Err(ApiError::Forbidden)
    ));
    let open = state_on(pool, |c| c.job_owner_only = false);
    assert!(svc_ask::job(&open, stranger, id).await.is_ok());
    eprintln!("review fixes 2-7 ok");
}

/// The run ceiling (`TREVOR_JOB_MAX_RUN_SECS`, default 1800): a job a live
/// worker keeps naming as busy is requeued once past it, then failed.
async fn step_run_ceiling(s: &AppState) {
    let pool = &s.db;
    reset_queue(pool).await;
    beat(s, "w1", false, true).await;
    let u = new_user(pool).await;
    let id = ask(s, u, "a job that never ends", None)
        .await
        .unwrap()
        .job_id
        .unwrap();
    let mut rx = s.trevor.subscribe(id);
    let busy = |w: &str| -> Heartbeat {
        serde_json::from_value(json!({
            "workerId": w, "rungs": { "cf": false, "cpu": true }, "busyJobId": id.to_string()
        }))
        .unwrap()
    };

    assert_eq!(next(s, "w1").await.unwrap().job_id, id);
    worker::heartbeat(s, &busy("w1")).await.unwrap();
    age_running(pool, id, 1801).await;
    worker::sweep(s).await.unwrap();
    let j = job(s, id).await;
    assert_eq!(
        (j.status.as_str(), j.attempts, j.worker_id.as_deref()),
        ("queued", 1, None),
        "requeued at 1801 s although w1 names it"
    );
    assert_eq!(j.error, None);

    // The second claim runs past the ceiling too: failed, and the stream ends.
    assert_eq!(next(s, "w2").await.unwrap().job_id, id);
    worker::heartbeat(s, &busy("w2")).await.unwrap();
    age_running(pool, id, 1799).await;
    worker::sweep(s).await.unwrap();
    assert_eq!(
        job(s, id).await.status,
        "running",
        "1799 s on the second claim"
    );
    age_running(pool, id, 1801).await;
    worker::sweep(s).await.unwrap();
    let j = job(s, id).await;
    assert_eq!((j.status.as_str(), j.attempts), ("failed", 2));
    assert_eq!(j.error.as_deref(), Some("the job ran past its time limit"));
    match rx.recv().await.unwrap() {
        StreamEvent::Error(m) => assert_eq!(m, "the job ran past its time limit"),
        other => panic!("expected an error event, got {other:?}"),
    }
    // A late result from the first worker is refused.
    let late = worker::result(s, &result_json(id, "w1", "cpu", (2000, 250))).await;
    assert!(late.is_err(), "late result after the ceiling");

    // A silent job (no worker names it) past the ceiling keeps the old message.
    reset_queue(pool).await;
    beat(s, "w1", false, true).await;
    let id = ask(s, u, "silent twice", None)
        .await
        .unwrap()
        .job_id
        .unwrap();
    for w in ["w1", "w2"] {
        assert_eq!(next(s, w).await.unwrap().job_id, id);
        age_running(pool, id, 1801).await;
        worker::sweep(s).await.unwrap();
    }
    assert_eq!(
        job(s, id).await.error.as_deref(),
        Some("the worker stopped answering")
    );
    eprintln!("run ceiling: busy job requeued at 1801 s, failed on the second, 1799 s kept");
}

// --- the read verdict re-derived on ask ----------------------------------------

/// The EN story tree when `ASSETS_DIR` names one (the main checkout's
/// `assets/output/en`); the worktree carries none.
fn real_en_tree() -> Option<PathBuf> {
    let dir = PathBuf::from(std::env::var_os("ASSETS_DIR")?);
    let dir = if dir.is_absolute() {
        dir
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(dir)
    };
    dir.join("gamedata/excel/story_review_table.json")
        .is_file()
        .then_some(dir)
}

/// The two mainline chapters and the first story of the third, with the
/// review table's real ids and gates: a `_beg` is read once its stage was
/// PLAYED, an `_end` once it was PASSED.
fn fixture_game_data() -> GameData {
    let story = |chapter: u32, stage: &str, half: &str| {
        let min = if half == "beg" { "PLAYED" } else { "PASS" };
        json!({
            "storyId": format!("main_{chapter}_level_{stage}_{half}"),
            "storyGroup": format!("main_{chapter}"),
            "storyName": format!("{stage} {half}"),
            "storyTxt": format!("obt/main/level_{stage}_{half}"),
            "requiredStages": [{ "stageId": stage, "minState": min, "maxState": "COMPLETE" }]
        })
    };
    let group = |chapter: u32, stages: &[&str]| -> StoryReviewGroup {
        let stories: Vec<Value> = stages
            .iter()
            .flat_map(|st| [story(chapter, st, "beg"), story(chapter, st, "end")])
            .collect();
        serde_json::from_value(json!({
            "id": format!("main_{chapter}"), "name": format!("Chapter {chapter}"),
            "entryType": "MAINLINE", "actType": "MAIN_STORY", "infoUnlockDatas": stories
        }))
        .expect("review group")
    };
    let mut gd = GameData::default();
    for (chapter, stages) in [
        (0, &["main_00-01", "main_00-02"][..]),
        (1, &["main_01-01"][..]),
        (2, &["main_02-01"][..]),
    ] {
        gd.story_reviews
            .insert(format!("main_{chapter}"), group(chapter, stages));
    }
    gd
}

/// EN server data for the story index: the real tree when `ASSETS_DIR`
/// names one, else [`fixture_game_data`] over no assets at all.
fn en_story_data() -> (HashMap<Server, Arc<ServerData>>, &'static str) {
    use std::sync::atomic::AtomicBool;
    let (gd, assets, dir, label) = if let Some(dir) = real_en_tree() {
        let (gd, assets) =
            backend::core::gamedata::init_game_data(&dir.join("gamedata/excel"), &dir)
                .expect("the EN game data");
        (gd, assets, dir, "the real EN story index")
    } else {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/trevor-db-test-no-assets");
        let assets = backend::core::gamedata::assets::AssetIndex::build(&dir);
        (
            fixture_game_data(),
            assets,
            dir,
            "the 8-story fixture index",
        )
    };
    let sd = Arc::new(ServerData {
        game_data: arc_swap::ArcSwap::from_pointee(gd),
        asset_index: arc_swap::ArcSwap::from_pointee(assets),
        game_data_dir: dir.join("gamedata/excel").display().to_string(),
        assets_dir: dir.display().to_string(),
        art_dir: dir.display().to_string(),
        loaded: AtomicBool::new(true),
        residency: Default::default(),
    });
    (HashMap::from([(Server::EN, sd)]), label)
}

async fn set_stages(pool: &PgPool, user: Uuid, stages: &[(&str, i32)]) {
    let doc: serde_json::Map<String, Value> = stages
        .iter()
        .map(|(id, state)| {
            let played = i32::from(*state >= 2);
            (
                (*id).to_owned(),
                json!({ "stageId": id, "state": state, "startTimes": played, "completeTimes": played }),
            )
        })
        .collect();
    sqlx::query("INSERT INTO user_stage_progress (user_id, stages) VALUES ($1, $2)")
        .bind(user)
        .bind(Value::Object(doc))
        .execute(pool)
        .await
        .expect("stages");
}

async fn game_rows(pool: &PgPool, user: Uuid) -> (i64, i64) {
    sqlx::query_as(
        "SELECT count(*), count(*) FILTER (WHERE read_in_game) FROM user_game_story_read WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .expect("rows")
}

/// An account with ONLY `user_stage_progress` (no game read rows, no site
/// marks) asks at its stage-derived horizon; with the switch off, at 0.
async fn step_reverdict_on_ask(pool: &PgPool) {
    let (data, label) = en_story_data();
    let s = state_with(pool, data.clone(), |_| {});
    let off = state_with(pool, data, |c| c.reverdict_on_ask = false);
    reset_queue(pool).await;

    // gatePos 1..6; eras open at 1 (chapter 0), 3 (chapter 1), 5 (chapter 2).
    let order = [
        "main_0_level_main_00-01_beg",
        "main_0_level_main_00-01_end",
        "main_1_level_main_01-01_beg",
        "main_1_level_main_01-01_end",
        "main_2_level_main_02-01_beg",
        "main_2_level_main_02-01_end",
    ];
    let eras = json!([
        { "id": "main_0", "name": "Evil Time Part 1", "position": 1 },
        { "id": "main_1", "name": "Evil Time Part 2", "position": 3 },
        { "id": "main_2", "name": "Separated Hearts", "position": 5 }
    ]);
    let panels: serde_json::Map<String, Value> = order
        .iter()
        .zip(1..)
        .map(|(id, pos)| ((*id).to_owned(), panel(id, Some(pos))))
        .collect();
    let body = json!({
        "version": "v-stages", "activate": true, "gateReport": { "spoilerLeaks": 0 },
        "manifest": { "builtFrom": "26-10-01-res", "storyOrder": order, "eras": eras },
        "panels": panels
    });
    assert_eq!(publish(&app(&s), &body).await, StatusCode::OK);

    // Chapters 0 and 1 cleared, chapter 2's first stage still locked.
    let a = new_user(pool).await;
    set_stages(
        pool,
        a,
        &[
            ("main_00-01", 3),
            ("main_00-02", 3),
            ("main_01-01", 3),
            ("main_02-01", 0),
        ],
    )
    .await;

    // Switch off: today's behaviour, nothing released and nothing written.
    let r = ask(&off, a, "Who is Amiya?", None).await.expect("ask off");
    assert_eq!(r.spoiler_safe_up_to, None);
    assert_eq!(job(&off, r.job_id.unwrap()).await.horizon_pos, Some(0));
    assert_eq!(game_rows(pool, a).await, (0, 0));
    assert_eq!(off.trevor.reverdicts.load(Ordering::Relaxed), 0);

    // On: the verdict is re-derived from the stages, stored, and read.
    let started = Instant::now();
    let r = ask(&s, a, "Who is Kal'tsit?", None).await.expect("ask on");
    let ms = started.elapsed().as_millis();
    let era = r.spoiler_safe_up_to.clone().expect("an era");
    assert_eq!((era.era_id.as_str(), era.position), ("main_1", 3));
    assert_eq!(
        job(&s, r.job_id.unwrap()).await.horizon_pos,
        Some(3),
        "read up to gatePos 4 (01-01 end) -> era main_1 at 3"
    );
    let (rows, read) = game_rows(pool, a).await;
    assert!(read >= 4, "the 00-01 and 01-01 halves at least: {read}");
    assert_eq!(s.trevor.reverdicts.load(Ordering::Relaxed), 1);
    // Exact gatePos under TREVOR_ERA_BUCKETS=0, from the rows now stored.
    let exact = state_with(pool, HashMap::new(), |c| c.era_buckets = false);
    let rx = ask(&exact, a, "Who is Theresa?", None)
        .await
        .expect("exact");
    assert_eq!(job(&exact, rx.job_id.unwrap()).await.horizon_pos, Some(4));
    assert_eq!(
        exact.trevor.reverdicts.load(Ordering::Relaxed),
        0,
        "rows exist now"
    );

    // Stages that satisfy no gate: re-derived ONCE per process, not per ask.
    let b = new_user(pool).await;
    set_stages(pool, b, &[("main_00-01", 0)]).await;
    for q in ["Who is W?", "Who is Ch'en?"] {
        let r = ask(&s, b, q, None).await.expect("ask b");
        assert_eq!(r.spoiler_safe_up_to, None);
        assert_eq!(job(&s, r.job_id.unwrap()).await.horizon_pos, Some(0));
    }
    assert_eq!(game_rows(pool, b).await.1, 0);
    assert_eq!(
        s.trevor.reverdicts.load(Ordering::Relaxed),
        2,
        "one for a, one for b"
    );

    // No stage progress at all: no re-derivation.
    let c = new_user(pool).await;
    let r = ask(&s, c, "Who is Mephisto?", None).await.expect("ask c");
    assert_eq!(job(&s, r.job_id.unwrap()).await.horizon_pos, Some(0));
    assert_eq!(s.trevor.reverdicts.load(Ordering::Relaxed), 2);

    // A site read mark: the stored reads stand, no re-derivation.
    let d = new_user(pool).await;
    set_stages(pool, d, &[("main_01-01", 3)]).await;
    set_progress(pool, d, &[order[0]], &[]).await;
    let r = ask(&s, d, "Who is Talulah?", None).await.expect("ask d");
    assert_eq!(job(&s, r.job_id.unwrap()).await.horizon_pos, Some(1));
    assert_eq!(s.trevor.reverdicts.load(Ordering::Relaxed), 2);
    eprintln!(
        "reverdict on ask ({label}): stage-only account -> era main_1 (gatePos 4), {rows} rows stored ({read} read), first ask {ms} ms; off -> 0; empty verdict re-derived once; marks or no stages skip it"
    );
}

// --- contract v1.1 ----------------------------------------------------------

async fn ask_ok(s: &AppState, user: Uuid, question: &str) -> AskResponse {
    ask(s, user, question, None).await.expect("ask")
}

/// A reader at era 4 (has read s5).
async fn era4_user(pool: &PgPool) -> Uuid {
    let u = new_user(pool).await;
    set_progress(pool, u, &["s5"], &[]).await;
    u
}

async fn beat_guard(s: &AppState, worker_id: &str, cpu: bool, guard: Value) {
    let hb: Heartbeat = serde_json::from_value(json!({
        "workerId": worker_id, "rungs": { "cf": false, "cpu": cpu },
        "corpusVersion": "v1", "busyJobId": null, "guard": guard
    }))
    .expect("heartbeat");
    worker::heartbeat(s, &hb).await.expect("heartbeat");
}

/// Claim directly (no long-poll) under the state's rules for one worker.
async fn claim(s: &AppState, worker_id: &str, cf: bool, cpu: bool) -> Option<q::JobRow> {
    let cfg = s.trevor.config();
    q::claim_next(
        &s.db,
        "en",
        worker_id,
        q::ClaimRules {
            admission: cfg.admission,
            fair: cfg.fair_claim,
            by_rung: cfg.claim_by_rung,
            cf,
            cpu,
            online_secs: 75,
        },
    )
    .await
    .expect("claim")
}

/// A live result citing `cites` ((chunkId, sha or none)).
fn result_cites(
    job_id: Uuid,
    worker_id: &str,
    version: &str,
    cites: &[(&str, Option<&str>)],
) -> Vec<u8> {
    let citations: Vec<Value> = cites
        .iter()
        .enumerate()
        .map(|(i, (chunk, sha))| {
            let mut c = json!({"n": i + 1, "storyId": "s1", "chunkId": chunk, "lineStart": 1,
                               "lineEnd": 2, "text": "t"});
            if let Some(sha) = sha {
                c["sha"] = json!(sha);
            }
            c
        })
        .collect();
    json!({
        "jobId": job_id, "workerId": worker_id, "ok": true,
        "result": {
            "answer": "A.", "rung": "cpu", "profile": "mid", "source": "live",
            "citations": citations, "route": "who", "form": "short", "corpusVersion": version,
            "tokensIn": 100, "tokensOut": 10, "timings": {}
        }
    })
    .to_string()
    .into_bytes()
}

async fn send_json(
    app: &Router,
    method: &str,
    uri: &str,
    key: bool,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let mut req = Request::builder().method(method).uri(uri);
    if key {
        req = req.header("x-trevor-worker-key", WORKER_KEY);
    }
    let req = match body {
        Some(b) => req
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(b.to_string()))
            .unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    let (status, _, body) = send(app, req).await;
    (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
}

/// normQ: case, NFKC (full-width), punctuation, apostrophes and the
/// trailing "?" all map to one key, shared by the twin join and the cache;
/// under `TREVOR_NORMQ=0` the full-width variant is a separate job.
async fn step_v11_normq(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |_| {});
    beat(&s, "w1", false, true).await;
    let (a, b) = (era4_user(pool).await, era4_user(pool).await);
    let first = ask_ok(&s, a, "Who is Kal'tsit?").await;
    let twin = ask_ok(&s, b, "  WHO is \u{FF2B}altsit ").await;
    assert_eq!(first.job_id, twin.job_id, "normQ variants join one job");
    let (old, _) = state_with_recorder(pool, |c| c.normq = false);
    let other = ask_ok(&old, b, "who is \u{FF2B}altsit").await;
    assert_ne!(
        other.job_id, first.job_id,
        "the v1 key keeps the full-width text apart"
    );
    eprintln!("v1.1 normQ: 2 variants -> 1 job; TREVOR_NORMQ=0 -> 2 jobs");
}

/// Admission control: the cpu cap is per online cpu worker, an overflow gets
/// a quick passages job plus a deferred job (and a Quick look from an
/// earlier era), the global cap drops the deferred job, the cf cap moves
/// questions on; claims serve quick passages first and deferred last.
async fn step_v11_admission(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |c| c.cpu_queue_max = 1);
    beat(&s, "w1", false, true).await;
    let u = era4_user(pool).await;
    let u2 = era4_user(pool).await;
    let one = ask_ok(&s, u, "admission one").await;
    assert_eq!(one.rung, Rung::Cpu);
    assert!(one.deferred_job_id.is_none());
    // One cpu worker at TREVOR_CPU_QUEUE_MAX=1: the second overflows.
    let two = ask_ok(&s, u, "admission two").await;
    assert_eq!(two.rung, Rung::Passages);
    let deferred = two.deferred_job_id.expect("deferred job");
    let dj = job(&s, deferred).await;
    assert!(dj.deferred);
    assert_eq!(dj.rung, "cpu");
    assert_eq!(dj.profile, "mid");
    // A second cpu worker doubles the cap.
    beat(&s, "w2", false, true).await;
    let three = ask_ok(&s, u2, "admission three").await;
    assert_eq!(three.rung, Rung::Cpu);
    // A twin of the overflowed ask joins both jobs.
    let twin = ask_ok(&s, u2, "Admission two?").await;
    assert_eq!(twin.job_id, two.job_id);
    assert_eq!(twin.deferred_job_id, Some(deferred));

    // Claim order: the quick passages job first, the deferred one last.
    let c1 = claim(&s, "w1", false, true).await.unwrap();
    assert_eq!(Some(c1.id), two.job_id, "quick passages first");
    let c2 = claim(&s, "w1", false, true).await.unwrap();
    let c3 = claim(&s, "w1", false, true).await.unwrap();
    assert!(!c2.deferred && !c3.deferred);
    let c4 = claim(&s, "w1", false, true).await.unwrap();
    assert_eq!(c4.id, deferred, "deferred only when no normal job waits");

    // Global cap reached: passages, no deferred job.
    reset_queue(pool).await;
    let (g, _) = state_with_recorder(pool, |c| {
        c.cpu_queue_max = 1;
        c.global_queue_max = 1;
    });
    beat(&g, "w1", false, true).await;
    ask_ok(&g, u, "global one").await;
    let over = ask_ok(&g, u, "global two").await;
    assert_eq!(over.rung, Rung::Passages);
    assert!(over.deferred_job_id.is_none());

    // The cf cap: TREVOR_CF_QUEUE_MAX=1 sends the second to cpu.
    reset_queue(pool).await;
    let (c, _) = state_with_recorder(pool, |c| c.cf_queue_max = 1);
    beat(&c, "w1", true, true).await;
    assert_eq!(ask_ok(&c, u, "cf one").await.rung, Rung::Cf);
    assert_eq!(ask_ok(&c, u, "cf two").await.rung, Rung::Cpu);

    // Quick look: an earlier era's cached answer to the same question.
    reset_queue(pool).await;
    let (ql, _) = state_with_recorder(pool, |c| c.cpu_queue_max = 0);
    beat(&ql, "w1", false, true).await;
    let early = new_user(pool).await;
    set_progress(pool, early, &["s2"], &[]).await;
    let ok = state_on(pool, |_| {});
    beat(&ok, "w9", false, true).await;
    let e = ask_ok(&ok, early, "quick look question").await;
    let claimed = next(&ok, "w9").await.unwrap();
    assert_eq!(Some(claimed.job_id), e.job_id);
    post_result(
        &ok,
        &result_cites(claimed.job_id, "w9", "v1", &[("s1#0", Some("a"))]),
    )
    .await;
    let late = ask_ok(&ql, u, "Quick look question").await;
    assert_eq!(late.rung, Rung::Passages);
    let look = late.quick_look.expect("quick look");
    assert_eq!(look.era_pos, 1);

    // TREVOR_ADMISSION=0: the v1 path (no deferred, flat cpu cap).
    reset_queue(pool).await;
    let (v1s, _) = state_with_recorder(pool, |c| {
        c.admission = false;
        c.cpu_queue_max = 1;
    });
    beat(&v1s, "w1", false, true).await;
    beat(&v1s, "w2", false, true).await;
    ask_ok(&v1s, u, "flat one").await;
    let flat = ask_ok(&v1s, u, "flat two").await;
    assert_eq!(
        flat.rung,
        Rung::Passages,
        "flat cap ignores the second worker"
    );
    assert!(flat.deferred_job_id.is_none() && flat.quick_look.is_none());
    eprintln!(
        "v1.1 admission: per-worker cpu cap, overflow -> passages + deferred, global cap, cf cap, quick look, claim order, TREVOR_ADMISSION=0 ok"
    );
}

/// Fair claiming and claim by rung.
async fn step_v11_fair(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |_| {});
    beat(&s, "w1", false, true).await;
    let (a, b) = (era4_user(pool).await, era4_user(pool).await);
    for i in 0..3 {
        ask_ok(&s, a, &format!("fair a {i}")).await;
    }
    let first = claim(&s, "w1", false, true).await.unwrap();
    assert_eq!(first.user_id, Some(a));
    let bj = ask_ok(&s, b, "fair b").await;
    let second = claim(&s, "w1", false, true).await.unwrap();
    assert_eq!(
        Some(second.id),
        bj.job_id,
        "the user with none running goes first"
    );
    // TREVOR_FAIR_CLAIM=0: the oldest created first.
    reset_queue(pool).await;
    let (old, _) = state_with_recorder(pool, |c| c.fair_claim = false);
    beat(&old, "w1", false, true).await;
    for i in 0..3 {
        ask_ok(&old, a, &format!("unfair a {i}")).await;
    }
    claim(&old, "w1", false, true).await.unwrap();
    ask_ok(&old, b, "unfair b").await;
    assert_eq!(
        claim(&old, "w1", false, true).await.unwrap().user_id,
        Some(a)
    );

    // Claim by rung: a cpu-only worker leaves a cf job to the online cf worker.
    reset_queue(pool).await;
    beat(&s, "cfw", true, false).await;
    beat(&s, "cpuw", false, true).await;
    let cfj = ask_ok(&s, a, "rung cf job").await;
    assert_eq!(cfj.rung, Rung::Cf);
    assert!(claim(&s, "cpuw", false, true).await.is_none());
    assert_eq!(
        claim(&s, "cfw", true, false).await.map(|j| j.id),
        cfj.job_id
    );
    // With the cf worker gone (stale), the cpu worker takes it.
    reset_queue(pool).await;
    beat(&s, "cpuw", false, true).await;
    let gone = ask_ok(&s, a, "rung orphan").await;
    sqlx::query("INSERT INTO trevor_workers (worker_id, last_seen, rungs) VALUES ('cfw', now() - interval '5 minutes', '{\"cf\": true, \"cpu\": false}')")
        .execute(pool)
        .await
        .unwrap();
    // The latest heartbeat offers cpu only, so the job is cpu: claimable.
    assert_eq!(
        claim(&s, "cpuw", false, true).await.map(|j| j.id),
        gone.job_id
    );
    // TREVOR_CLAIM_BY_RUNG=0: any worker takes any job.
    reset_queue(pool).await;
    let (any, _) = state_with_recorder(pool, |c| c.claim_by_rung = false);
    beat(&any, "cfw", true, false).await;
    beat(&any, "cpuw", false, true).await;
    let cf2 = ask_ok(&any, a, "rung cf job two").await;
    assert_eq!(
        claim(&any, "cpuw", false, true).await.map(|j| j.id),
        cf2.job_id
    );
    eprintln!(
        "v1.1 fair claim: fewest running first, TREVOR_FAIR_CLAIM=0 oldest first; claim by rung ok"
    );
}

/// Mode, status, guard and the alerts (fake webhook, no network).
async fn step_v11_mode_status_alerts(pool: &PgPool) {
    reset_queue(pool).await;
    let u = era4_user(pool).await;
    // passages: a passages job, nothing deferred, no model job.
    let (p, _) = state_with_recorder(pool, |c| {
        c.mode = backend::app::services::trevor::Mode::Passages
    });
    beat(&p, "w1", true, true).await;
    let r = ask_ok(&p, u, "mode passages").await;
    assert_eq!(r.rung, Rung::Passages);
    assert!(r.deferred_job_id.is_none());
    // A model job queued before the switch runs as passages.
    let (on, _) = state_with_recorder(pool, |_| {});
    let m = ask_ok(&on, u, "mode model").await;
    assert_eq!(m.rung, Rung::Cf);
    claim(&p, "w1", true, true).await.unwrap(); // the passages one
    let w = next(&p, "w1").await.expect("job");
    assert_eq!(Some(w.job_id), m.job_id);
    assert_eq!(w.rung, Rung::Passages);
    // off: 503 "Trevor is resting" unless cached.
    let (off, _) =
        state_with_recorder(pool, |c| c.mode = backend::app::services::trevor::Mode::Off);
    match ask(&off, u, "mode off", None).await {
        Err(ApiError::ServiceUnavailableMessage(m)) => assert_eq!(m, "Trevor is resting"),
        other => panic!("expected 503, got {other:?}"),
    }
    post_result(
        &p,
        &result_cites(w.job_id, "w1", "v1", &[("s1#0", Some("a"))]),
    )
    .await;
    let hit = ask(&off, u, "Mode model?", None)
        .await
        .expect("cached in off mode");
    assert_eq!(hit.rung, Rung::Cache);

    // Status: workers per rung, guard, mode.
    reset_queue(pool).await;
    let (s, rec) = state_with_recorder(pool, |_| {});
    beat(&s, "a", true, false).await;
    beat(&s, "b", false, true).await;
    beat(&s, "c", false, true).await;
    let st = svc_ask::status(&s, "en").await.unwrap();
    assert_eq!((st.workers.cf, st.workers.cpu), (1, 2));
    assert_eq!(serde_json::to_value(st.guard).unwrap(), "ok");
    assert_eq!(serde_json::to_value(st.mode).unwrap(), "on");
    // guard_tripped: fires once, recovers once.
    let tripped = json!({"state": "tripped", "reason": "psi cpu 71 > 60", "since": 1});
    beat_guard(&s, "b", false, tripped.clone()).await;
    assert_eq!(rec.take(), vec![(alerts::AlertKind::GuardTripped, false)]);
    beat_guard(&s, "b", false, tripped).await;
    assert!(rec.take().is_empty(), "at most once");
    let st = svc_ask::status(&s, "en").await.unwrap();
    assert_eq!(serde_json::to_value(st.guard).unwrap(), "tripped");
    assert_eq!(st.workers.cpu, 1);
    beat_guard(
        &s,
        "b",
        true,
        json!({"state": "ok", "reason": null, "since": 2}),
    )
    .await;
    assert_eq!(rec.take(), vec![(alerts::AlertKind::GuardTripped, true)]);
    // backlog: a normal job queued 6 minutes ago; deferred ones never count.
    let j = ask_ok(&s, u, "backlog question").await;
    sqlx::query("UPDATE trevor_jobs SET queued_at = now() - interval '6 minutes' WHERE id = $1")
        .bind(j.job_id)
        .execute(pool)
        .await
        .unwrap();
    let sent = alerts::evaluate(&s).await.unwrap();
    assert_eq!(
        sent.iter()
            .map(|a| (a.kind, a.recovered))
            .collect::<Vec<_>>(),
        vec![(alerts::AlertKind::Backlog, false)]
    );
    assert_eq!(rec.take().len(), 1);
    assert!(alerts::evaluate(&s).await.unwrap().is_empty(), "cooldown");
    sqlx::query("UPDATE trevor_jobs SET deferred = true WHERE id = $1")
        .bind(j.job_id)
        .execute(pool)
        .await
        .unwrap();
    let sent = alerts::evaluate(&s).await.unwrap();
    assert_eq!(
        sent.iter()
            .map(|a| (a.kind, a.recovered))
            .collect::<Vec<_>>(),
        vec![(alerts::AlertKind::Backlog, true)]
    );
    // failure_rate: 11 of the last 40 finished failed.
    reset_queue(pool).await;
    sqlx::query(
        "INSERT INTO trevor_jobs (question, qkey, server, status, finished_at, horizon_pos) \
         SELECT 'q' || g, 'k' || g, 'en', CASE WHEN g <= 11 THEN 'failed' ELSE 'done' END, now(), g \
         FROM generate_series(1, 40) g",
    )
    .execute(pool)
    .await
    .unwrap();
    beat(&s, "a", true, false).await;
    let sent = alerts::evaluate(&s).await.unwrap();
    assert!(
        sent.iter()
            .any(|a| a.kind == alerts::AlertKind::FailureRate && !a.recovered),
        "{sent:?}"
    );
    // TREVOR_ALERTS=0: nothing.
    let (quiet, qrec) = state_with_recorder(pool, |c| c.alerts = false);
    assert!(alerts::evaluate(&quiet).await.unwrap().is_empty());
    assert!(qrec.take().is_empty());
    eprintln!(
        "v1.1 mode/status/alerts: passages + off modes, workers {{cf 1, cpu 2}}, guard fire+recover once, backlog fire+recover, failure_rate 11/40 fires, TREVOR_ALERTS=0 silent"
    );
}

async fn ask_spoilers(
    s: &AppState,
    user: Uuid,
    question: &str,
    spoilers: &str,
) -> Result<AskResponse, ApiError> {
    let req: AskRequest =
        serde_json::from_value(json!({ "question": question, "spoilers": spoilers })).expect("ask");
    svc_ask::ask(s, user, None, req).await
}

/// Claim the next job through `worker/next` (so the wire horizon is the
/// one a worker receives) and answer it live, citing one chunk.
async fn answer_next(s: &AppState, version: &str) -> worker::WorkerJob {
    let j = next(s, "w1").await.expect("a queued job");
    post_result(
        s,
        &result_cites(j.job_id, "w1", version, &[("s1#0", Some("aaa"))]),
    )
    .await;
    j
}

/// Spoiler mode (contract v1.1 section 8): a spoiler ask and a spoiler-safe
/// ask never share a cache entry or a job, the worker gets `horizonPos:
/// null`, "My questions" marks the spoiler asks, `popular` never counts or
/// lists them for a spoiler-safe reader, an unknown value is a 400, and
/// `TREVOR_SPOILER_MODE=0` reads every ask as spoiler-safe.
async fn step_spoiler_mode(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |_| {});
    beat(&s, "w1", false, true).await;
    let version: String = q::active_version(pool, "en").await.unwrap().unwrap();
    let (a, b, c, d) = (
        era4_user(pool).await,
        era4_user(pool).await,
        era4_user(pool).await,
        era4_user(pool).await,
    );

    // A spoiler-safe answer at era 4.
    let safe = ask_ok(&s, a, "who is the ghost").await;
    assert!(safe.spoilers.is_none() && safe.spoiler_safe_up_to.is_some());
    let wire = serde_json::to_value(&safe).unwrap();
    assert!(
        wire.get("spoilers").is_none(),
        "a safe response keeps its bytes"
    );
    let j = answer_next(&s, &version).await;
    assert_eq!(j.horizon_pos, Some(4));

    // A spoiler ask of the same question is not served the safe entry.
    let all = ask_spoilers(&s, b, "Who is the ghost?", "all")
        .await
        .unwrap();
    assert_ne!(
        all.rung,
        Rung::Cache,
        "a spoiler ask never reads a safe entry"
    );
    assert_eq!(all.spoilers, Some(SpoilerMode::All));
    assert!(all.spoiler_safe_up_to.is_none());
    assert_eq!(
        serde_json::to_value(&all).unwrap()["spoilers"],
        json!("all")
    );
    let row = job(&s, all.job_id.unwrap()).await;
    assert_eq!(row.horizon_pos, Some(NO_LIMIT_ERA));
    let j = answer_next(&s, &version).await;
    assert_eq!(j.job_id, all.job_id.unwrap());
    assert_eq!(j.horizon_pos, None, "the worker gets no limit");

    // Each bucket now serves only its own answer.
    let hit_all = ask_spoilers(&s, c, "who is the ghost", "all")
        .await
        .unwrap();
    assert_eq!(hit_all.rung, Rung::Cache);
    assert_eq!(hit_all.spoilers, Some(SpoilerMode::All));
    assert_eq!(hit_all.answer.as_ref().unwrap().era_pos, NO_LIMIT_ERA);
    let hit_safe = ask_ok(&s, d, "who is the ghost").await;
    assert_eq!(hit_safe.rung, Rung::Cache);
    assert_eq!(hit_safe.answer.as_ref().unwrap().era_pos, 4);
    assert!(hit_safe.spoilers.is_none());

    // The reverse: a question answered only in spoiler mode is a new job
    // for a spoiler-safe reader.
    ask_spoilers(&s, b, "only with spoilers", "all")
        .await
        .unwrap();
    answer_next(&s, &version).await;
    assert_eq!(
        ask_spoilers(&s, c, "only with spoilers", "all")
            .await
            .unwrap()
            .rung,
        Rung::Cache
    );
    let fresh = ask_ok(&s, a, "only with spoilers").await;
    assert_ne!(
        fresh.rung,
        Rung::Cache,
        "a safe ask never reads a spoiler entry"
    );
    answer_next(&s, &version).await;

    // Twins join only within their bucket.
    let open_safe = ask_ok(&s, a, "an open question").await;
    let open_all = ask_spoilers(&s, b, "an open question", "all")
        .await
        .unwrap();
    assert_ne!(open_safe.job_id, open_all.job_id);
    let twin_all = ask_spoilers(&s, c, "an open question", "all")
        .await
        .unwrap();
    assert_eq!(twin_all.job_id, open_all.job_id);
    let twin_safe = ask_ok(&s, d, "an open question").await;
    assert_eq!(twin_safe.job_id, open_safe.job_id);

    // My questions marks the spoiler asks only.
    let mine_b = svc_ask::my_questions(&s, b, None, None).await.unwrap();
    assert_eq!(mine_b.len(), 3);
    assert!(mine_b.iter().all(|m| m.spoilers == Some(SpoilerMode::All)));
    let mine_a = svc_ask::my_questions(&s, a, None, None).await.unwrap();
    assert!(mine_a.iter().all(|m| m.spoilers.is_none()));

    // Popular for a spoiler-safe reader: "who is the ghost" has 2 safe
    // askers (a, d) and is listed with its era-4 answer; "only with
    // spoilers" has 1 safe asker (a) and 2 spoiler ones (b, c), so it is
    // not listed although 3 accounts asked it.
    let pop = svc_ask::popular(&s, Some(a), "en", None).await.unwrap();
    let names: Vec<(&str, i64, i32)> = pop
        .items
        .iter()
        .map(|p| (p.question.as_str(), p.askers, p.answer.era_pos))
        .collect();
    assert_eq!(names, vec![("who is the ghost", 2, 4)]);

    // An unknown value is a 400.
    assert!(matches!(
        ask_spoilers(&s, a, "who is the ghost", "maybe").await,
        Err(ApiError::BadRequest(_))
    ));

    // TREVOR_SPOILER_MODE=0: the field is ignored, as before.
    let (off, _) = state_with_recorder(pool, |c| c.spoiler_mode = false);
    let as_safe = ask_spoilers(&off, b, "who is the ghost", "all")
        .await
        .unwrap();
    assert_eq!(as_safe.rung, Rung::Cache);
    assert_eq!(as_safe.answer.as_ref().unwrap().era_pos, 4);
    assert!(as_safe.spoilers.is_none() && as_safe.spoiler_safe_up_to.is_some());
    assert!(
        serde_json::to_value(&as_safe)
            .unwrap()
            .get("spoilers")
            .is_none()
    );
    let junk = ask_spoilers(&off, b, "who is the ghost", "maybe")
        .await
        .unwrap();
    assert_eq!(junk.rung, Rung::Cache);
    println!(
        "spoiler mode: safe/all buckets isolated both ways (eraPos 4 vs {NO_LIMIT_ERA}), wire horizonPos null, twins split by bucket, popular [who is the ghost 2], 400 on unknown, TREVOR_SPOILER_MODE=0 reads as safe"
    );
}

/// "My questions", popular, the warm-up routes, and the carry-over.
async fn step_v11_lists_and_carry(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |_| {});
    let app = app(&s);
    beat(&s, "w1", false, true).await;
    let (a, b, c) = (
        era4_user(pool).await,
        era4_user(pool).await,
        era4_user(pool).await,
    );
    let version: String = q::active_version(pool, "en").await.unwrap().unwrap();

    // Answer "carry me" (cites two chunks with shas), "carry changed" (one),
    // "no sha" (none).
    let mut answered = Vec::new();
    for (question, cites) in [
        (
            "carry me",
            vec![("s1#0", Some("aaa")), ("s2#1", Some("bbb"))],
        ),
        ("carry changed", vec![("s3#0", Some("ccc"))]),
        ("no sha", vec![("s1#0", None)]),
    ] {
        let r = ask_ok(&s, a, question).await;
        let j = claim(&s, "w1", false, true).await.unwrap();
        assert_eq!(Some(j.id), r.job_id);
        post_result(&s, &result_cites(j.id, "w1", &version, &cites)).await;
        answered.push(r.job_id.unwrap());
    }
    // b and c ask "carry me" (cache hits): 3 distinct askers.
    assert_eq!(ask_ok(&s, b, "Carry me?").await.rung, Rung::Cache);
    assert_eq!(ask_ok(&s, c, "carry  me").await.rung, Rung::Cache);
    assert_eq!(ask_ok(&s, b, "carry changed").await.rung, Rung::Cache);

    // My questions: newest first, the cache hits included.
    let mine = svc_ask::my_questions(&s, a, None, None).await.unwrap();
    assert_eq!(mine.len(), 3);
    assert_eq!(mine[0].question, "no sha");
    assert!(
        mine.iter()
            .all(|m| m.status == JobStatus::Done && m.answer.is_some())
    );
    let theirs = svc_ask::my_questions(&s, b, None, Some(1)).await.unwrap();
    assert_eq!(
        (theirs.len(), theirs[0].rung, theirs[0].job_id),
        (1, Rung::Cache, None)
    );
    let page2 = svc_ask::my_questions(&s, a, Some(mine[1].created_at), None)
        .await
        .unwrap();
    assert_eq!(page2.len(), 1);

    // Popular: at least 2 distinct askers, inside the era, which the body names.
    let pop = svc_ask::popular(&s, Some(a), "en", None).await.unwrap();
    let names: Vec<(&str, i64)> = pop
        .items
        .iter()
        .map(|p| (p.question.as_str(), p.askers))
        .collect();
    assert_eq!(names, vec![("carry me", 3), ("carry changed", 2)]);
    let era = pop
        .spoiler_safe_up_to
        .expect("a reader with progress has an era");
    let anon = svc_ask::popular(&s, None, "en", None).await.unwrap();
    assert!(anon.spoiler_safe_up_to.is_none() && anon.items.is_empty());
    let early = new_user(pool).await;
    set_progress(pool, early, &["s2"], &[]).await;
    let pop_early = svc_ask::popular(&s, Some(early), "en", None).await.unwrap();
    assert!(
        pop_early.items.is_empty(),
        "answers at era 4 are past era 1"
    );
    assert!(
        pop_early
            .spoiler_safe_up_to
            .is_some_and(|e| e.position < era.position),
        "the early reader's era comes before the first reader's"
    );
    let (st, body) = send_json(&app, "GET", "/api/trevor/popular?limit=1", false, None).await;
    assert_eq!(
        (st, body),
        (
            StatusCode::OK,
            json!({ "spoilerSafeUpTo": null, "items": [] })
        )
    );

    // Carry-over: v-carry ships chunk shas; s3#0 changed.
    let shas = json!({ "s1#0": "aaa", "s2#1": "bbb", "s3#0": "CHANGED" });
    let panels: serde_json::Map<String, Value> = (1..=6)
        .map(|i| (format!("s{i}"), panel(&format!("s{i}"), Some(i))))
        .collect();
    let chunk =
        json!({ "version": "v-carry", "activate": false, "panels": panels, "chunkShas": shas });
    assert_eq!(publish(&app, &chunk).await, StatusCode::OK);
    let (st, out) = send_json(
        &app,
        "POST",
        "/api/trevor/versions",
        true,
        Some(json!({
            "version": "v-carry", "activate": true, "gateReport": { "ok": true },
            "manifest": { "storyOrder": story_order(), "eras": eras() }
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{out}");
    assert_eq!(
        out["carried"], 1,
        "only the answer whose cited shas all match"
    );
    assert_eq!(ask_ok(&s, b, "carry me").await.rung, Rung::Cache);
    assert_ne!(ask_ok(&s, b, "carry changed").await.rung, Rung::Cache);
    assert_ne!(ask_ok(&s, b, "no sha").await.rung, Rung::Cache);

    // Warm-up list for v-carry: the asked pairs with no cached answer there.
    let (st, list) = send_json(
        &app,
        "GET",
        "/api/trevor/worker/popular?version=v-carry&limit=10",
        true,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{list}");
    assert_eq!(list["version"], "v-carry");
    let items = list["items"].as_array().unwrap();
    let norm: Vec<&str> = items.iter().map(|i| i["normQ"].as_str().unwrap()).collect();
    assert_eq!(
        norm,
        vec!["carry changed", "no sha"],
        "carry me was carried over"
    );
    assert_eq!(items[0]["hits"], 3);
    assert_eq!(items[0]["eraPos"], 4);
    assert_eq!(
        send_json(
            &app,
            "GET",
            "/api/trevor/worker/popular?version=nope",
            true,
            None
        )
        .await
        .0,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        send_json(
            &app,
            "GET",
            "/api/trevor/worker/popular?version=v-carry",
            false,
            None
        )
        .await
        .0,
        StatusCode::UNAUTHORIZED
    );

    // Warm: queue deferred internal jobs.
    let warm = json!({ "version": "v-carry", "items": [
        { "question": "carry me", "eraPos": 4 },
        { "question": "carry changed", "eraPos": 4 },
        { "question": "warm fresh", "eraPos": 4 },
        { "question": "warm fresh", "eraPos": 4 }
    ]});
    let (st, out) = send_json(
        &app,
        "POST",
        "/api/trevor/worker/warm",
        true,
        Some(warm.clone()),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{out}");
    // carry me: cached; carry changed: b's ask above left an open job (joined);
    // warm fresh: queued once, then joined.
    assert_eq!(out, json!({ "queued": 1, "cached": 1, "joined": 2 }));
    let (internal, deferred): (Option<Uuid>, bool) =
        sqlx::query_as("SELECT user_id, deferred FROM trevor_jobs WHERE kind = 'warm'")
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!((internal, deferred), (None, true));
    let stale = json!({ "version": version, "items": [] });
    assert_eq!(
        send_json(&app, "POST", "/api/trevor/worker/warm", true, Some(stale))
            .await
            .0,
        StatusCode::CONFLICT
    );
    let big =
        json!({ "version": "v-carry", "items": [{ "question": "x".repeat(70_000), "eraPos": 0 }] });
    assert_eq!(
        send_json(&app, "POST", "/api/trevor/worker/warm", true, Some(big))
            .await
            .0,
        StatusCode::PAYLOAD_TOO_LARGE
    );
    let (p, _) = state_with_recorder(pool, |c| {
        c.mode = backend::app::services::trevor::Mode::Passages
    });
    assert_eq!(
        send_json(
            &crate::app(&p),
            "POST",
            "/api/trevor/worker/warm",
            true,
            Some(warm)
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );

    // TREVOR_CARRY_OVER=0: nothing moves.
    let (nc, _) = state_with_recorder(pool, |c| c.carry_over = false);
    let nca = crate::app(&nc);
    let chunk = json!({ "version": "v-nocarry", "activate": false, "panels": panels, "chunkShas": { "s1#0": "aaa", "s2#1": "bbb" } });
    assert_eq!(publish(&nca, &chunk).await, StatusCode::OK);
    let (_, out) = send_json(
        &nca,
        "POST",
        "/api/trevor/versions",
        true,
        Some(json!({
            "version": "v-nocarry", "activate": true, "gateReport": { "ok": true },
            "manifest": { "storyOrder": story_order(), "eras": eras() }
        })),
    )
    .await;
    assert_eq!(out["carried"], 0);
    eprintln!(
        "v1.1 lists/carry: my questions 3 + paging, popular [carry me 3, carry changed 2], carried 1 of 3, warm-up list + warm {{queued 1, cached 1, joined 2}}, 404/401/409/413/503, TREVOR_CARRY_OVER=0 carried 0"
    );
}

/// A bank answer as `scripts/publish.py` sends it, citing `cites`
/// ((chunkId, sha)).
fn bank_answer(
    question: &str,
    gate_pos: Option<i32>,
    cites: &[(&str, &str)],
    source: &str,
    answered_at: Option<&str>,
) -> Value {
    let citations: Vec<Value> = cites
        .iter()
        .enumerate()
        .map(|(i, (chunk, sha))| {
            json!({"n": i + 1, "storyId": chunk.split('#').next().unwrap(), "chunkId": chunk,
                   "lineStart": 1, "lineEnd": 9, "text": "Quoted line.", "sha": sha})
        })
        .collect();
    json!({
        "question": question, "aliases": [], "answer": format!("Banked: {question}."),
        "citations": citations, "gatePos": gate_pos, "source": source, "answeredAt": answered_at
    })
}

/// Publish `version` (panels and `shas` in a chunk, then the activating call
/// with `bank`); returns the activating call's answer.
async fn publish_bank(app: &Router, version: &str, shas: Value, bank: Value) -> Value {
    let panels: serde_json::Map<String, Value> = (1..=6)
        .map(|i| (format!("s{i}"), panel(&format!("s{i}"), Some(i))))
        .collect();
    let chunk =
        json!({ "version": version, "activate": false, "panels": panels, "chunkShas": shas });
    assert_eq!(publish(app, &chunk).await, StatusCode::OK);
    let (st, out) = send_json(
        app,
        "POST",
        "/api/trevor/versions",
        true,
        Some(json!({
            "version": version, "activate": true, "gateReport": { "ok": true },
            "manifest": { "storyOrder": story_order(), "eras": eras() },
            "bankAnswers": bank
        })),
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{out}");
    assert_eq!(out["active"], true, "{out}");
    out
}

fn answer_text(r: &AskResponse) -> String {
    r.answer
        .as_ref()
        .and_then(|a| a.answer["answer"].as_str())
        .unwrap_or_default()
        .to_owned()
}

/// The bank import (contract v1.1 section 3a, addition 2026-10-19). Eras sit
/// at gatePos 1 and 4 over stories s1..s6.
async fn step_bank_import(pool: &PgPool) {
    reset_queue(pool).await;
    let (s, _) = state_with_recorder(pool, |_| {});
    let app = app(&s);
    beat(&s, "w1", false, true).await;
    let prev: String = q::active_version(pool, "en").await.unwrap().unwrap();
    let era4 = era4_user(pool).await;
    let era1 = new_user(pool).await;
    set_progress(pool, era1, &["s2"], &[]).await;
    let era0 = new_user(pool).await;

    // Two worker answers at era 4 in the active version, made now.
    for question in ["bank newer live", "bank replaces live"] {
        let r = ask_ok(&s, era4, question).await;
        let j = claim(&s, "w1", false, true).await.unwrap();
        assert_eq!(Some(j.id), r.job_id);
        post_result(
            &s,
            &result_cites(j.id, "w1", &prev, &[("s1#0", Some("aaa"))]),
        )
        .await;
    }

    let tomorrow = (chrono::Utc::now() + chrono::Duration::days(1)).to_rfc3339();
    let mut early = bank_answer("Bank early?", Some(1), &[("s1#0", "aaa")], "bank", None);
    early["aliases"] = json!(["What is bank early", "bank   EARLY"]);
    let bank = json!([
        early,
        bank_answer(
            "bank mid",
            Some(3),
            &[("s2#1", "bbb")],
            "reanswer",
            Some(&tomorrow)
        ),
        bank_answer("bank late", Some(5), &[("s5#0", "eee")], "bank", None),
        bank_answer("bank undatable", None, &[("s1#0", "aaa")], "bank", None),
        bank_answer("bank stale", Some(1), &[("s1#0", "zzz")], "bank", None),
        bank_answer("bank nocite", Some(1), &[], "bank", None),
        bank_answer(
            "bank newer live",
            Some(3),
            &[("s1#0", "aaa")],
            "bank",
            Some("2020-01-01T00:00:00Z")
        ),
        bank_answer(
            "bank replaces live",
            Some(3),
            &[("s1#0", "aaa")],
            "reanswer",
            Some(&tomorrow)
        ),
    ]);
    let shas = json!({ "s1#0": "aaa", "s2#1": "bbb", "s5#0": "eee" });
    let out = publish_bank(&app, "v-bank", shas, bank.clone()).await;
    // Rows: early 2 wordings x (era 1, spoiler) = 4 ("bank   EARLY" is the
    // same key as "Bank early?"), mid (4, spoiler) 2, late (spoiler) 1,
    // undatable (spoiler) 1, stale 2, newer live 2, replaces live 2.
    assert_eq!(
        out["bank"],
        json!({ "sent": 8, "invalid": 1, "rows": 14, "imported": 11, "stale": 2, "kept": 1, "failed": false })
    );
    let hit = |r: &AskResponse| r.rung == Rung::Cache;

    // A horizon that admits the answer: a cache hit, shaped like any other.
    let r = ask_ok(&s, era1, "bank early").await;
    assert!(hit(&r), "era 1 reads the era-1 row");
    let a = r.answer.as_ref().unwrap();
    assert_eq!(
        (a.rung, a.source, a.era_pos),
        (
            Rung::Cpu,
            backend::app::services::trevor::AnswerSource::Live,
            1
        )
    );
    assert_eq!(a.answer["citations"][0]["chunkId"], "s1#0");
    assert_eq!(a.answer["citations"][0]["sha"], "aaa");
    assert_eq!(answer_text(&r), "Banked: Bank early?.");
    assert!(r.spoilers.is_none() && r.spoiler_safe_up_to.is_some());
    assert!(
        hit(&ask_ok(&s, era1, "what is bank early?").await),
        "an alias is its own key"
    );
    assert!(
        hit(&ask_ok(&s, era4, "Bank early").await),
        "a later era reads the era-1 row"
    );
    assert!(hit(&ask_ok(&s, era4, "bank mid").await));
    // A horizon below the gate: a miss. Each miss queues a job, so the era-4
    // misses take fresh readers (a reader holds at most 3 open jobs).
    assert!(
        !hit(&ask_ok(&s, era1, "bank mid").await),
        "gatePos 3 is past era 1"
    );
    assert!(
        !hit(&ask_ok(&s, era0, "bank early").await),
        "nothing read, nothing served"
    );
    assert!(
        !hit(&ask_ok(&s, era4_user(pool).await, "bank late").await),
        "gatePos 5 is past the last era"
    );
    // Spoiler mode reads the no-limit rows.
    let r = ask_spoilers(&s, era0, "bank late", "all").await.unwrap();
    assert!(hit(&r) && r.answer.as_ref().unwrap().era_pos == NO_LIMIT_ERA);
    assert_eq!(r.spoilers, Some(SpoilerMode::All));
    assert!(hit(&ask_spoilers(&s, era1, "bank undatable", "all")
        .await
        .unwrap()));
    assert!(
        !hit(&ask_ok(&s, era4_user(pool).await, "bank undatable").await),
        "undatable: spoiler mode only"
    );
    assert!(
        !hit(&ask_ok(&s, era4_user(pool).await, "bank stale").await),
        "a sha the version does not hold"
    );
    // The overwrite rule: the worker's newer answer stays, an older one goes.
    let r = ask_ok(&s, era4, "bank newer live").await;
    assert!(hit(&r));
    assert_eq!(
        answer_text(&r),
        "A.",
        "the worker answer is newer than the bank's"
    );
    let r = ask_ok(&s, era4, "bank replaces live").await;
    assert!(hit(&r));
    assert_eq!(answer_text(&r), "Banked: bank replaces live.");
    let (carried, origins): (i64, i64) = sqlx::query_as(
        "SELECT count(*) FILTER (WHERE carried_from IS NOT NULL), count(*) FILTER (WHERE origin IS NOT NULL) \
         FROM trevor_answers WHERE version = 'v-bank'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(
        (carried, origins),
        (1, 11),
        "only the newer worker answer was carried"
    );

    // The warm-up skips what the bank covers: "bank early" at era 4 is served
    // by the era-1 row, while "bank mid" at era 1 is past its gate.
    let (st, list) = send_json(
        &app,
        "GET",
        "/api/trevor/worker/popular?version=v-bank&limit=50",
        true,
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK, "{list}");
    let pairs: Vec<(String, i64)> = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| {
            (
                i["normQ"].as_str().unwrap().to_owned(),
                i["eraPos"].as_i64().unwrap(),
            )
        })
        .collect();
    assert!(!pairs.contains(&("bank early".into(), 4)), "{pairs:?}");
    assert!(pairs.contains(&("bank mid".into(), 1)), "{pairs:?}");
    let (_, out) = send_json(
        &app,
        "POST",
        "/api/trevor/worker/warm",
        true,
        Some(json!({ "version": "v-bank", "items": [{ "question": "bank early", "eraPos": 4 }] })),
    )
    .await;
    assert_eq!(out, json!({ "queued": 0, "cached": 1, "joined": 0 }));

    // A failed import (a NUL byte Postgres refuses) rolls back alone: the
    // publish stands, imported rows are never carried over, and it is counted.
    let mut bad = bank_answer("bank nul\u{0}", Some(1), &[("s1#0", "aaa")], "bank", None);
    bad["answer"] = json!("nul");
    let failed_before = s.trevor.bank_import_failures.load(Ordering::Relaxed);
    let out = publish_bank(
        &app,
        "v-bank-fail",
        json!({ "s1#0": "aaa", "s2#1": "bbb", "s5#0": "eee" }),
        json!([bad, bank[1].clone()]),
    )
    .await;
    assert_eq!(
        (
            out["bank"]["failed"].clone(),
            out["bank"]["imported"].clone()
        ),
        (json!(true), json!(0)),
        "{out}"
    );
    assert_eq!(
        s.trevor.bank_import_failures.load(Ordering::Relaxed),
        failed_before + 1
    );
    assert_eq!(
        q::active_version(pool, "en").await.unwrap().as_deref(),
        Some("v-bank-fail")
    );
    assert!(
        !hit(&ask_ok(&s, era4_user(pool).await, "bank mid").await),
        "imported rows are not carried"
    );

    // A re-publish with a changed chunk sha: every row citing it is stale.
    let out = publish_bank(
        &app,
        "v-bank2",
        json!({ "s1#0": "CHANGED", "s2#1": "bbb", "s5#0": "eee" }),
        bank.clone(),
    )
    .await;
    assert_eq!(
        (
            out["bank"]["imported"].clone(),
            out["bank"]["stale"].clone()
        ),
        (json!(3), json!(11)),
        "{out}"
    );
    assert!(!hit(&ask_ok(&s, era1, "bank early").await), "s1#0 changed");
    assert!(hit(&ask_ok(&s, era4, "bank mid").await), "s2#1 did not");

    // TREVOR_BANK_IMPORT=0: no imported row is served, and a publish imports none.
    let (off, _) = state_with_recorder(pool, |c| c.bank_import = false);
    let offa = crate::app(&off);
    assert!(
        !hit(&ask_spoilers(&off, era0, "bank late", "all").await.unwrap()),
        "the exact key skips it"
    );
    assert!(
        !hit(&ask_ok(&off, era4_user(pool).await, "bank mid").await),
        "no fallback"
    );
    let out = publish_bank(
        &offa,
        "v-bank3",
        json!({ "s1#0": "aaa", "s2#1": "bbb", "s5#0": "eee" }),
        bank,
    )
    .await;
    assert_eq!(
        out["bank"],
        json!({ "sent": 8, "invalid": 0, "rows": 0, "imported": 0, "stale": 0, "kept": 0, "failed": false })
    );
    let n: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trevor_answers WHERE version = 'v-bank3' AND origin IS NOT NULL",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(n, 0);
    eprintln!(
        "bank import: 8 sent, 1 invalid, 14 rows, 11 imported, 2 stale, 1 kept by a newer worker answer; hits at admitting horizons, misses below the gate, spoiler mode on the no-limit rows; failed import rolled back, publish stood; changed sha 11 stale; TREVOR_BANK_IMPORT=0 imported 0, served none"
    );
}
