//! Trevor under a burst (contract v1.1 sections 1a to 1d): the in-process
//! `/api` router against the throwaway database `trevor_test` (its `public`
//! schema is dropped and rebuilt, as in `trevor_db_test.rs`; the `postgres`
//! database is only asked whether `trevor_test` exists).
//!
//! 2,000 signed-in readers spread over 5 era buckets ask 10 distinct
//! questions at once (50 cache keys); 4 fake workers (no model: they sleep,
//! stream 3 deltas and post a result) serve the queue; 1,000 of the readers
//! hold the job's SSE stream open until `done`. Then the same 2,000 ask
//! again. Measured: asks/s accepted, p50/p99 ask latency, claim order
//! (deferred jobs never claimed while a normal job waited), job waits per
//! kind, the instant passages answer of every overflowed ask, the cache hit
//! share of the second burst, and the peak RSS of this test process.
//!
//! Ignored by default (needs the local Postgres):
//! `cargo test --release --test trevor_load_test -- --ignored --nocapture`

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use backend::app::cache::store::CacheStore;
use backend::app::services::trevor::alerts;
use backend::app::services::trevor::runtime::TrevorConfig;
use backend::app::state::{AppConfig, AppState};
use backend::core::auth::credentials::CredentialKey;
use backend::core::hypergryph::constants::Server;
use backend::database::queries::trevor as q;
use futures_util::StreamExt;
use serde_json::{Value, json};
use sqlx::{Connection, PgConnection, PgPool};
use tower::ServiceExt;
use uuid::Uuid;

const ADMIN_URL: &str = "postgres://postgres:password@127.0.0.1:5432/postgres";
const TEST_DB: &str = "trevor_test";
const TEST_URL: &str = "postgres://postgres:password@127.0.0.1:5432/trevor_test";
const WORKER_KEY: &str = "load-test-worker-key-0123456789";
const JWT_SECRET: &str = "load-test-jwt-secret-0123456789abcdef";

const USERS: usize = 2000;
const QUESTIONS: usize = 10;
const ERAS: usize = 5;
const WORKERS: usize = 4;
const SSE: usize = 1000;
/// A fake model answer takes this long; a passages answer `PASSAGES_MS`.
const MODEL_MS: u64 = 400;
const PASSAGES_MS: u64 = 20;

async fn fresh_db() -> PgPool {
    let mut admin = PgConnection::connect(ADMIN_URL)
        .await
        .expect("local Postgres");
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM pg_database WHERE datname = $1)")
            .bind(TEST_DB)
            .fetch_one(&mut admin)
            .await
            .expect("catalog");
    if !exists {
        sqlx::raw_sql("CREATE DATABASE trevor_test")
            .execute(&mut admin)
            .await
            .expect("create trevor_test");
    }
    admin.close().await.ok();
    let mut c = PgConnection::connect(TEST_URL).await.expect("trevor_test");
    let name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&mut c)
        .await
        .expect("name");
    assert_eq!(name, TEST_DB, "refusing to reset any other database");
    sqlx::raw_sql("DROP SCHEMA public CASCADE; CREATE SCHEMA public;")
        .execute(&mut c)
        .await
        .expect("reset schema");
    c.close().await.ok();
    backend::database::init(TEST_URL)
        .await
        .expect("migrations and seed")
}

struct Quiet;

impl alerts::AlertSink for Quiet {
    fn send(&self, _: alerts::Alert) {}
}

fn state(pool: &PgPool) -> AppState {
    let config = AppConfig {
        jwt_secret: JWT_SECRET.into(),
        game_credential_key: CredentialKey::parse(&"22".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "load-test-service-key".into(),
        assets_base_dir: String::new(),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
        role_recheck: false,
    };
    let s = AppState::new(
        pool.clone(),
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
    s.trevor.set_sink(Arc::new(Quiet));
    s
}

/// Version `vload`: 50 stories, eras at 1, 10, 20, 30, 40.
async fn publish(pool: &PgPool) {
    let order: Vec<String> = (1..=50).map(|i| format!("s{i}")).collect();
    let eras: Vec<Value> = [1, 10, 20, 30, 40]
        .iter()
        .map(|p| json!({ "id": format!("e{p}"), "name": format!("Era {p}"), "position": p }))
        .collect();
    let panels: Vec<q::PanelRow> = (1..=50)
        .map(|i| q::PanelRow {
            story_id: format!("s{i}"),
            gate_pos: Some(i),
            panels: json!({ "position": i }),
        })
        .collect();
    q::publish_version(
        pool,
        "vload",
        "en",
        None,
        Some(&json!({ "ok": true })),
        Some(&json!({ "storyOrder": order, "eras": eras })),
        &panels,
        true,
        &[],
        false,
    )
    .await
    .expect("publish");
}

/// 2,000 accounts; account i has read story `s{5 + 10 * ((i / 10) % 5)}`,
/// so the readers fall evenly into the 5 era buckets, independent of the
/// question they ask (`i % 10`): 50 cache keys.
async fn users(pool: &PgPool) -> Vec<Uuid> {
    let ids: Vec<Uuid> = sqlx::query_scalar(
        "INSERT INTO users (uid, server_id) \
         SELECT 'load' || g, 0 FROM generate_series(1, $1) g RETURNING id",
    )
    .bind(USERS as i32)
    .fetch_all(pool)
    .await
    .expect("users");
    let docs: Vec<Value> = (0..ids.len())
        .map(|i| {
            json!({ "read": { format!("s{}", 5 + 10 * ((i / QUESTIONS) % ERAS)): 1 }, "unread": {} })
        })
        .collect();
    sqlx::query(
        "INSERT INTO user_story_progress (user_id, progress) \
         SELECT * FROM UNNEST($1::uuid[], $2::jsonb[])",
    )
    .bind(&ids)
    .bind(&docs)
    .execute(pool)
    .await
    .expect("progress");
    ids
}

fn bearer(user: Uuid, i: usize) -> String {
    let token = backend::core::auth::jwt::create_token(
        JWT_SECRET,
        &user.to_string(),
        &format!("load{i}"),
        "en",
        "user",
        1,
    )
    .expect("token");
    format!("Bearer {token}")
}

async fn call(app: &Router, req: Request<Body>) -> (StatusCode, Vec<u8>) {
    let resp = app.clone().oneshot(req).await.expect("infallible");
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .map(|b| b.to_vec())
        .unwrap_or_default();
    (status, body)
}

fn worker_post(uri: &str, body: &Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-trevor-worker-key", WORKER_KEY)
        .body(Body::from(body.to_string()))
        .unwrap()
}

/// A fake worker: heartbeat, then claim, sleep, 3 deltas, result, until told
/// to stop.
async fn fake_worker(app: Router, id: String, stop: Arc<AtomicBool>, served: Arc<AtomicUsize>) {
    let beat = json!({ "workerId": id, "rungs": { "cf": false, "cpu": true },
                       "corpusVersion": "vload", "busyJobId": null,
                       "guard": { "state": "ok", "reason": null, "since": null } });
    let (st, _) = call(&app, worker_post("/api/trevor/worker/heartbeat", &beat)).await;
    assert_eq!(st, StatusCode::NO_CONTENT);
    while !stop.load(Ordering::Relaxed) {
        let (st, body) = call(
            &app,
            worker_post("/api/trevor/worker/next", &json!({ "workerId": id })),
        )
        .await;
        if st == StatusCode::NO_CONTENT {
            continue;
        }
        assert_eq!(st, StatusCode::OK, "{}", String::from_utf8_lossy(&body));
        let job: Value = serde_json::from_slice(&body).unwrap();
        let job_id = job["jobId"].as_str().unwrap().to_owned();
        let rung = job["rung"].as_str().unwrap().to_owned();
        let model = rung != "passages";
        tokio::time::sleep(Duration::from_millis(if model {
            MODEL_MS
        } else {
            PASSAGES_MS
        }))
        .await;
        for seq in 0..3 {
            let d = json!({ "workerId": id, "seq": seq, "text": format!("part {seq} ") });
            call(
                &app,
                worker_post(&format!("/api/trevor/worker/jobs/{job_id}/delta"), &d),
            )
            .await;
        }
        let result = json!({
            "jobId": job_id, "workerId": id, "ok": true,
            "result": {
                "answer": format!("An answer to {}.", job["question"]),
                "rung": rung, "profile": job["profile"],
                "source": if model { "live" } else { "passages" },
                "citations": [{ "n": 1, "storyId": "s1", "chunkId": "s1#0", "lineStart": 1,
                                "lineEnd": 2, "text": "t", "sha": "abc" }],
                "route": "who", "form": "short", "corpusVersion": "vload",
                "tokensIn": 1000, "tokensOut": 100, "timings": {}
            }
        });
        let (st, body) = call(&app, worker_post("/api/trevor/worker/result", &result)).await;
        assert_eq!(
            st,
            StatusCode::NO_CONTENT,
            "{}",
            String::from_utf8_lossy(&body)
        );
        served.fetch_add(1, Ordering::Relaxed);
    }
}

/// Peak RSS of this process in KiB, sampled every 100 ms with `ps`.
fn rss_sampler(stop: Arc<AtomicBool>) -> std::thread::JoinHandle<u64> {
    std::thread::spawn(move || {
        let pid = std::process::id().to_string();
        let mut peak = 0u64;
        while !stop.load(Ordering::Relaxed) {
            if let Ok(out) = std::process::Command::new("ps")
                .args(["-o", "rss=", "-p", &pid])
                .output()
                && let Ok(kib) = String::from_utf8_lossy(&out.stdout).trim().parse::<u64>()
            {
                peak = peak.max(kib);
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        peak
    })
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() as f64 - 1.0) * p).round() as usize;
    sorted[i]
}

#[derive(Default)]
struct Burst {
    latencies_ms: Vec<f64>,
    statuses: HashMap<u16, usize>,
    rungs: HashMap<String, usize>,
    overflowed: usize,
    quick_looks: usize,
    wall: Duration,
    /// (user index, job id) of every ask that got a job.
    jobs: Vec<(usize, String)>,
}

async fn burst(app: &Router, users: &[Uuid], round: usize) -> Burst {
    let start = Instant::now();
    let tasks = users.iter().enumerate().map(|(i, u)| {
        let app = app.clone();
        let auth = bearer(*u, i);
        async move {
            let question = format!("What happened to character {}?", i % QUESTIONS);
            // Round 2 types it differently: normQ must still hit the cache.
            let question = if round == 2 {
                format!("  WHAT happened to Character {} ", i % QUESTIONS)
            } else {
                question
            };
            let req = Request::post("/api/trevor/ask")
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::AUTHORIZATION, auth)
                .body(Body::from(json!({ "question": question }).to_string()))
                .unwrap();
            let t = Instant::now();
            let (st, body) = call(&app, req).await;
            (i, st, body, t.elapsed())
        }
    });
    let results = futures_util::future::join_all(tasks).await;
    let mut b = Burst {
        wall: start.elapsed(),
        ..Burst::default()
    };
    for (i, st, body, took) in results {
        b.latencies_ms.push(took.as_secs_f64() * 1000.0);
        *b.statuses.entry(st.as_u16()).or_default() += 1;
        let Ok(v) = serde_json::from_slice::<Value>(&body) else {
            continue;
        };
        if let Some(r) = v["rung"].as_str() {
            *b.rungs.entry(r.to_owned()).or_default() += 1;
        }
        if v.get("deferredJobId").is_some_and(|d| !d.is_null()) || v["rung"] == "passages" {
            b.overflowed += 1;
        }
        if v.get("quickLook").is_some() {
            b.quick_looks += 1;
        }
        if let Some(j) = v["jobId"].as_str() {
            b.jobs.push((i, j.to_owned()));
        }
    }
    b.latencies_ms.sort_by(f64::total_cmp);
    b
}

/// Open `n` job streams and read each to its end; returns (opened, got
/// `done`, deltas seen).
async fn sse_readers(
    app: &Router,
    users: &[Uuid],
    jobs: &[(usize, String)],
    n: usize,
) -> (usize, usize, u64) {
    let deltas = Arc::new(AtomicU64::new(0));
    let tasks = jobs.iter().take(n).map(|(i, job)| {
        let app = app.clone();
        let auth = bearer(users[*i], *i);
        let deltas = deltas.clone();
        let job = job.clone();
        async move {
            let req = Request::get(format!("/api/trevor/jobs/{job}/stream"))
                .header(header::AUTHORIZATION, auth)
                .body(Body::empty())
                .unwrap();
            let resp = app.oneshot(req).await.expect("infallible");
            if resp.status() != StatusCode::OK {
                return (false, false);
            }
            let mut body = resp.into_body().into_data_stream();
            let mut text = String::new();
            let read = tokio::time::timeout(Duration::from_secs(120), async {
                while let Some(Ok(chunk)) = body.next().await {
                    text.push_str(&String::from_utf8_lossy(&chunk));
                    if text.contains("event: done") || text.contains("event: error") {
                        break;
                    }
                }
            })
            .await;
            deltas.fetch_add(
                text.matches("event: delta").count() as u64,
                Ordering::Relaxed,
            );
            (true, read.is_ok() && text.contains("event: done"))
        }
    });
    let out = futures_util::future::join_all(tasks).await;
    (
        out.iter().filter(|(o, _)| *o).count(),
        out.iter().filter(|(_, d)| *d).count(),
        deltas.load(Ordering::Relaxed),
    )
}

async fn wait_drained(pool: &PgPool, limit: Duration) -> Duration {
    let start = Instant::now();
    loop {
        let open: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM trevor_jobs WHERE status IN ('queued', 'running')",
        )
        .fetch_one(pool)
        .await
        .unwrap();
        if open == 0 || start.elapsed() > limit {
            return start.elapsed();
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[ignore = "needs the local Postgres; creates and resets the throwaway database trevor_test"]
async fn trevor_under_a_burst() {
    let pool = fresh_db().await;
    publish(&pool).await;
    let users = users(&pool).await;
    let s = state(&pool);
    let app = backend::app::server::api_parts().0.with_state(s.clone());

    let stop_rss = Arc::new(AtomicBool::new(false));
    let rss = rss_sampler(stop_rss.clone());
    let stop = Arc::new(AtomicBool::new(false));
    let served = Arc::new(AtomicUsize::new(0));
    let workers: Vec<_> = (0..WORKERS)
        .map(|w| {
            tokio::spawn(fake_worker(
                app.clone(),
                format!("load-w{w}"),
                stop.clone(),
                served.clone(),
            ))
        })
        .collect();
    // Let the heartbeats land.
    tokio::time::sleep(Duration::from_millis(300)).await;

    // Round 1: the burst, then 1,000 stream readers.
    let b1 = burst(&app, &users, 1).await;
    let sse_started = Instant::now();
    let (opened, done, deltas) = sse_readers(&app, &users, &b1.jobs, SSE).await;
    let sse_wall = sse_started.elapsed();
    let drain1 = wait_drained(&pool, Duration::from_secs(300)).await;

    // Claim order and waits, from the rows.
    let (normal_wait, deferred_wait, passages_wait): (Vec<f64>, Vec<f64>, Vec<f64>) = {
        let rows: Vec<(bool, String, f64)> = sqlx::query_as(
            "SELECT deferred, rung, extract(epoch FROM started_at - queued_at)::float8 * 1000 \
             FROM trevor_jobs WHERE started_at IS NOT NULL",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        let mut n: Vec<f64> = rows
            .iter()
            .filter(|r| !r.0 && r.1 != "passages")
            .map(|r| r.2)
            .collect();
        let mut d: Vec<f64> = rows.iter().filter(|r| r.0).map(|r| r.2).collect();
        let mut p: Vec<f64> = rows
            .iter()
            .filter(|r| !r.0 && r.1 == "passages")
            .map(|r| r.2)
            .collect();
        n.sort_by(f64::total_cmp);
        d.sort_by(f64::total_cmp);
        p.sort_by(f64::total_cmp);
        (n, d, p)
    };
    // A deferred job claimed while a normal job waited is an order violation.
    let violations: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs d JOIN trevor_jobs n \
           ON NOT n.deferred AND n.queued_at < d.started_at AND n.started_at > d.started_at \
         WHERE d.deferred",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let (jobs_total, deferred_total, passages_total, failed): (i64, i64, i64, i64) =
        sqlx::query_as(
            "SELECT count(*), count(*) FILTER (WHERE deferred), \
                    count(*) FILTER (WHERE rung = 'passages' AND NOT deferred), \
                    count(*) FILTER (WHERE status = 'failed') FROM trevor_jobs",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
    // Every overflowed ask's quick passages job finished.
    let passages_done: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM trevor_jobs WHERE rung = 'passages' AND NOT deferred AND status = 'done'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let passages_total_ms: Vec<f64> = {
        let mut v: Vec<f64> = sqlx::query_scalar(
            "SELECT extract(epoch FROM finished_at - created_at)::float8 * 1000 FROM trevor_jobs \
             WHERE rung = 'passages' AND NOT deferred AND finished_at IS NOT NULL",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        v.sort_by(f64::total_cmp);
        v
    };

    // Round 2: the same readers, the same questions typed differently.
    let b2 = burst(&app, &users, 2).await;
    let drain2 = wait_drained(&pool, Duration::from_secs(120)).await;

    stop.store(true, Ordering::Relaxed);
    for w in workers {
        // Each returns after its current long-poll (at most 20 s).
        let _ = tokio::time::timeout(Duration::from_secs(25), w).await;
    }
    stop_rss.store(true, Ordering::Relaxed);
    let peak_kib = rss.join().unwrap();

    let hits2 = b2.rungs.get("cache").copied().unwrap_or(0);
    eprintln!(
        "--- Trevor load test ({USERS} users, {QUESTIONS} questions x {ERAS} eras, {WORKERS} fake workers at {MODEL_MS} ms) ---"
    );
    eprintln!(
        "round 1: {} asks in {:.3} s = {:.1} asks/s; latency p50 {:.1} ms p99 {:.1} ms max {:.1} ms; statuses {:?}; rungs {:?}",
        USERS,
        b1.wall.as_secs_f64(),
        USERS as f64 / b1.wall.as_secs_f64(),
        pct(&b1.latencies_ms, 0.5),
        pct(&b1.latencies_ms, 0.99),
        b1.latencies_ms.last().copied().unwrap_or(0.0),
        b1.statuses,
        b1.rungs
    );
    eprintln!(
        "round 1 overflow: {} asks answered by a quick passages job ({} with a Quick look); jobs {} (deferred {}, quick passages {}, failed {}); quick passages done {} of {}, created->finished p50 {:.1} ms p99 {:.1} ms",
        b1.overflowed,
        b1.quick_looks,
        jobs_total,
        deferred_total,
        passages_total,
        failed,
        passages_done,
        passages_total,
        pct(&passages_total_ms, 0.5),
        pct(&passages_total_ms, 0.99)
    );
    eprintln!(
        "claim: waits (queued->started) normal model p50 {:.0} ms p99 {:.0} ms; quick passages p50 {:.0} p99 {:.0}; deferred p50 {:.0} p99 {:.0}; deferred claimed while a normal job waited: {violations}; served {} jobs; drained in {:.2} s",
        pct(&normal_wait, 0.5),
        pct(&normal_wait, 0.99),
        pct(&passages_wait, 0.5),
        pct(&passages_wait, 0.99),
        pct(&deferred_wait, 0.5),
        pct(&deferred_wait, 0.99),
        served.load(Ordering::Relaxed),
        drain1.as_secs_f64()
    );
    eprintln!(
        "sse: {opened} of {} streams opened, {done} got done, {deltas} deltas, all read in {:.2} s",
        SSE.min(b1.jobs.len()),
        sse_wall.as_secs_f64()
    );
    eprintln!(
        "round 2: {} asks in {:.3} s = {:.1} asks/s; latency p50 {:.1} ms p99 {:.1} ms; cache hits {} of {} = {:.1}%; rungs {:?}; drained in {:.2} s",
        USERS,
        b2.wall.as_secs_f64(),
        USERS as f64 / b2.wall.as_secs_f64(),
        pct(&b2.latencies_ms, 0.5),
        pct(&b2.latencies_ms, 0.99),
        hits2,
        USERS,
        100.0 * hits2 as f64 / USERS as f64,
        b2.rungs,
        drain2.as_secs_f64()
    );
    eprintln!(
        "peak RSS of the test process: {:.1} MiB",
        peak_kib as f64 / 1024.0
    );

    assert_eq!(b1.statuses.values().sum::<usize>(), USERS);
    assert!(
        b1.statuses.keys().all(|s| *s == 200 || *s == 202),
        "every ask accepted: {:?}",
        b1.statuses
    );
    assert!(b1.overflowed > 0, "the burst overflows the cpu rung");
    assert_eq!(
        passages_done, passages_total,
        "every quick passages job answered"
    );
    assert_eq!(violations, 0, "deferred jobs wait for normal ones");
    assert_eq!(done, opened, "every stream reader got done");
    assert_eq!(hits2, USERS, "the second burst is served from the cache");
}
