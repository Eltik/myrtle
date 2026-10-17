//! Trevor end to end: this branch's `/api` router served in-process on a
//! random localhost port against the throwaway database `trevor_test`, and
//! the real worker (`trevor/target/release/ask --worker`) pulling from it
//! over HTTP. It never starts the backend server binary and never touches
//! the `postgres` database's data (it only asks that server whether
//! `trevor_test` exists).
//!
//! Driven, in order:
//! 1. A worker with no model (`--no-spawn` at an unused port): heartbeats,
//!    `/trevor/status` online with `cpu: false`, `last_seen` advancing.
//! 2. A publish of the worker's corpus version: three panels at their real
//!    gatePos (artifacts/chrono/gate_order.json) and the real `storyOrder`
//!    and `eras` in the manifest.
//! 3. Signed-in asks (a bearer token for a `users` row) on the `passages`
//!    rung: a reader of the first two episodes gets era `main_1` (gatePos
//!    26) and no cited passage past it; a reader of Episode 16 gets a later
//!    era and a separate job.
//! 4. Unless `TREVOR_E2E_CPU=0`: a worker that starts the local Gemma 12B
//!    llama-server under the model lock, the `cpu` rung (slim profile), SSE
//!    deltas, the stored answer, `busyJobId` in the heartbeat mid-job, and a
//!    cache hit on the identical second ask. The ask-to-first-delta latency
//!    is printed. The worker and its llama-server are stopped by PID.
//!
//! Needs the local Postgres, `trevor/target/release/ask` (`cargo build
//! --release --features embed --bin ask` in trevor/), its artifacts and, for
//! step 4, models/llm/gemma-4-12b-it-qat-q4_0.gguf and `llama-server` on the
//! PATH. A worker starts only with 25% of memory free and the 12B only with
//! 30% (waiting up to `TREVOR_E2E_WAIT_SECS`, default 900, then panicking);
//! step 4 refuses at once when a llama-server is already running or another
//! job holds the model lock (`ask`'s default: $TREVOR_LLM_LOCK, else
//! trevor-llm.lock in $TMPDIR or /tmp). It shares `trevor_test` with
//! `trevor_db_test`, so never run the two at once:
//!
//! `cargo test --test trevor_e2e_test -- --ignored --nocapture`
//!
//! `TREVOR_E2E_RETRIEVAL=bm25` runs the workers with `--retrieval bm25` (no
//! query embedder); worker logs go to `$CARGO_TARGET_TMPDIR/trevor-e2e/`.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use backend::app::state::{AppConfig, AppState};
use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

#[path = "common/trevor.rs"]
mod trevor_common;
use trevor_common::{TEST_DB, app, app_config, app_state, fresh_db, insert_user, set_progress};

const WORKER_KEY: &str = "e2e-test-worker-key-0123456789";
const JWT_SECRET: &str = "e2e-test-jwt-secret-0123456789abcdef";
/// The model lock, resolved as `ask --lock` resolves its default (trevor/src/bin/ask/cli.rs, `lock_path_from`).
fn model_lock() -> std::path::PathBuf {
    let set = |k: &str| std::env::var_os(k).filter(|v| !v.is_empty());
    set("TREVOR_LLM_LOCK").map(Into::into).unwrap_or_else(|| {
        std::path::PathBuf::from(set("TMPDIR").unwrap_or_else(|| "/tmp".into()))
            .join("trevor-llm.lock")
    })
}
/// The worker's heartbeat interval in this test (30 s in production).
const HEARTBEAT_SECS: u64 = 2;

// --- the database and the server ---------------------------------------------

fn state_on(pool: &PgPool) -> AppState {
    let config = AppConfig {
        rate_limit_rpm: 1000,
        ..app_config(JWT_SECRET, "e2e-service-key")
    };
    app_state(pool.clone(), HashMap::new(), config, WORKER_KEY)
}

/// Serve the `/api` router (as `server::run` builds it, without the global
/// rate-limit and observe layers) on 127.0.0.1 at a random port.
async fn serve(s: &AppState) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("addr");
    let app = app(s);
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .expect("serve");
    });
    format!("http://{addr}")
}

async fn new_user(pool: &PgPool) -> (Uuid, String) {
    let (id, uid) = insert_user(pool, "te2e").await;
    let token =
        backend::core::auth::jwt::create_token(JWT_SECRET, &id.to_string(), &uid, "en", "user", 1)
            .expect("token");
    (id, format!("Bearer {token}"))
}

// --- the worker process --------------------------------------------------------

fn trevor_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../trevor")
}

fn scratch() -> PathBuf {
    let d = Path::new(env!("CARGO_TARGET_TMPDIR")).join("trevor-e2e");
    std::fs::create_dir_all(&d).expect("scratch dir");
    d
}

fn sh(cmd: &str, args: &[&str]) -> (bool, String) {
    match Command::new(cmd).args(args).output() {
        Ok(o) => (
            o.status.success(),
            String::from_utf8_lossy(&o.stdout).into_owned(),
        ),
        Err(_) => (false, String::new()),
    }
}

fn alive(pid: u32) -> bool {
    sh("kill", &["-0", &pid.to_string()]).0
}

/// `memory_pressure`'s "System-wide memory free percentage".
fn free_memory_pct() -> u32 {
    let (_, out) = sh("memory_pressure", &[]);
    out.lines()
        .rev()
        .find_map(|l| {
            l.split("percentage:")
                .nth(1)?
                .trim()
                .trim_end_matches('%')
                .parse()
                .ok()
        })
        .unwrap_or(0)
}

/// Wait (up to `TREVOR_E2E_WAIT_SECS`, default 900) for at least `min`%
/// free memory, polling every 15 s; panics when it never comes.
async fn wait_for_memory(min: u32, what: &str) -> u32 {
    let within = Duration::from_secs(
        std::env::var("TREVOR_E2E_WAIT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(900),
    );
    let t = Instant::now();
    loop {
        let free = free_memory_pct();
        if free >= min {
            return free;
        }
        assert!(
            t.elapsed() < within,
            "free memory {free}% < {min}% for {within:?}: not {what}"
        );
        if t.elapsed().as_secs() % 300 < 15 {
            eprintln!("free memory {free}% < {min}%: waiting before {what}");
        }
        tokio::time::sleep(Duration::from_secs(15)).await;
    }
}

/// The model lock's holder while that process lives.
fn lock_holder() -> Option<String> {
    let text = std::fs::read_to_string(model_lock()).ok()?;
    let pid: u32 = text.split_whitespace().next()?.parse().ok()?;
    alive(pid).then(|| text.trim().to_owned())
}

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .expect("bind")
        .local_addr()
        .expect("addr")
        .port()
}

/// A worker process, stopped by PID when dropped: SIGTERM (the worker stops
/// the llama-server it started), then SIGKILL after 20 s, then SIGKILL for
/// any llama-server child still alive, and the model lock removed if it
/// still names the worker.
struct Worker {
    child: Child,
    name: &'static str,
    llama: Vec<u32>,
}

impl Worker {
    fn spawn(name: &'static str, base: &str, args: &[String]) -> Self {
        let bin = trevor_dir().join("target/release/ask");
        assert!(
            bin.exists(),
            "{} is missing: cargo build --release --features embed --bin ask in trevor/",
            bin.display()
        );
        let log = std::fs::File::create(scratch().join(format!("{name}.log"))).expect("log");
        let spool = scratch().join(format!("{name}-spool"));
        let mut cmd = Command::new(bin);
        cmd.current_dir(trevor_dir())
            .arg("--worker")
            .arg(base)
            .args(args)
            .env("TREVOR_WORKER_KEY", WORKER_KEY)
            .env("TREVOR_WORKER_ID", format!("e2e-{name}"))
            .env("TREVOR_HEARTBEAT_SECS", HEARTBEAT_SECS.to_string())
            .env("TREVOR_SPOOL_DIR", &spool)
            .env_remove("TREVOR_CF_MODELS")
            .env_remove("TREVOR_CF_ACCOUNT_ID")
            .env_remove("TREVOR_CF_BASE_URL")
            .stdin(Stdio::null())
            .stdout(log.try_clone().expect("log"))
            .stderr(log);
        if let Ok(r) = std::env::var("TREVOR_E2E_RETRIEVAL") {
            cmd.args(["--retrieval", &r]);
        }
        let child = cmd.spawn().expect("spawn the worker");
        eprintln!(
            "worker {name}: pid {}, log {}",
            child.id(),
            scratch().join(format!("{name}.log")).display()
        );
        Self {
            child,
            name,
            llama: Vec::new(),
        }
    }

    fn id(&self) -> String {
        format!("e2e-{}", self.name)
    }

    /// Record the worker's llama-server children, so they are stopped even
    /// if the worker has to be killed.
    fn note_children(&mut self) {
        let (_, out) = sh("pgrep", &["-P", &self.child.id().to_string()]);
        for pid in out.split_whitespace().filter_map(|p| p.parse().ok()) {
            if !self.llama.contains(&pid) {
                self.llama.push(pid);
            }
        }
    }

    fn stop(&mut self) {
        let pid = self.child.id();
        self.note_children();
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = sh("kill", &["-TERM", &pid.to_string()]);
            let t = Instant::now();
            while t.elapsed() < Duration::from_secs(20) {
                if self.child.try_wait().ok().flatten().is_some() {
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
            if self.child.try_wait().ok().flatten().is_none() {
                eprintln!("worker {}: no exit 20 s after SIGTERM; SIGKILL", self.name);
                let _ = self.child.kill();
                let _ = self.child.wait();
            }
        }
        for &l in &self.llama {
            if alive(l) {
                eprintln!(
                    "worker {}: llama-server {l} outlived it; SIGKILL",
                    self.name
                );
                let _ = sh("kill", &["-KILL", &l.to_string()]);
            }
        }
        if std::fs::read_to_string(model_lock())
            .is_ok_and(|t| t.split_whitespace().next() == Some(pid.to_string().as_str()))
        {
            let _ = std::fs::remove_file(model_lock());
        }
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        self.stop();
    }
}

/// The worker's heartbeat row: (rungs, corpus version, busy job, last seen).
async fn heartbeat_row(
    pool: &PgPool,
    worker_id: &str,
) -> Option<(
    Value,
    Option<String>,
    Option<Uuid>,
    chrono::DateTime<chrono::Utc>,
)> {
    sqlx::query_as(
        "SELECT rungs, corpus_version, busy_job_id, last_seen FROM trevor_workers WHERE worker_id = $1",
    )
    .bind(worker_id)
    .fetch_optional(pool)
    .await
    .expect("worker row")
}

/// Wait for a heartbeat (with `cpu` as asked); the corpus version it names.
async fn await_worker(pool: &PgPool, w: &mut Worker, want_cpu: bool, within: Duration) -> String {
    let t = Instant::now();
    loop {
        if let Some((rungs, version, _, _)) = heartbeat_row(pool, &w.id()).await
            && rungs["cpu"] == json!(want_cpu)
        {
            w.note_children();
            eprintln!(
                "worker {}: online after {:.1} s, rungs {rungs}",
                w.name,
                t.elapsed().as_secs_f64()
            );
            return version.expect("corpus version");
        }
        if let Ok(Some(status)) = w.child.try_wait() {
            panic!("worker {} exited ({status}) before its heartbeat", w.name);
        }
        assert!(
            t.elapsed() < within,
            "worker {} sent no heartbeat with cpu {want_cpu} in {within:?}",
            w.name
        );
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

// --- the HTTP client -----------------------------------------------------------

struct Client {
    http: reqwest::Client,
    base: String,
}

impl Client {
    async fn send(&self, req: reqwest::RequestBuilder) -> (u16, Value) {
        let resp = req.send().await.expect("request");
        let status = resp.status().as_u16();
        let body = resp.bytes().await.expect("body");
        (status, serde_json::from_slice(&body).unwrap_or(Value::Null))
    }

    async fn ask(&self, bearer: &str, question: &str) -> (u16, Value) {
        self.send(
            self.http
                .post(format!("{}/api/trevor/ask", self.base))
                .header("authorization", bearer)
                .json(&json!({ "question": question })),
        )
        .await
    }

    async fn job(&self, bearer: &str, id: &str) -> Value {
        let (st, v) = self
            .send(
                self.http
                    .get(format!("{}/api/trevor/jobs/{id}", self.base))
                    .header("authorization", bearer),
            )
            .await;
        assert_eq!(st, 200, "job {id}: {v}");
        v
    }

    async fn status(&self) -> Value {
        let (st, v) = self
            .send(self.http.get(format!("{}/api/trevor/status", self.base)))
            .await;
        assert_eq!(st, 200);
        v
    }

    async fn publish(&self, body: &Value) -> u16 {
        self.send(
            self.http
                .post(format!("{}/api/trevor/versions", self.base))
                .header("x-trevor-worker-key", WORKER_KEY)
                .json(body),
        )
        .await
        .0
    }

    /// Read a job's SSE stream to its end: the deltas with their arrival
    /// times, and the terminal event's name and data.
    async fn stream(&self, bearer: &str, id: &str, within: Duration) -> Sse {
        let mut resp = self
            .http
            .get(format!("{}/api/trevor/jobs/{id}/stream", self.base))
            .header("authorization", bearer)
            .send()
            .await
            .expect("stream");
        assert_eq!(resp.status().as_u16(), 200, "stream status");
        let mut buf = String::new();
        let mut out = Sse::default();
        let t = Instant::now();
        loop {
            let left = within.saturating_sub(t.elapsed());
            let chunk = tokio::time::timeout(left, resp.chunk())
                .await
                .unwrap_or_else(|_| panic!("the stream of {id} did not end in {within:?}"))
                .expect("chunk");
            let Some(chunk) = chunk else { break };
            buf.push_str(&String::from_utf8_lossy(&chunk));
            while let Some(end) = buf.find("\n\n") {
                let frame: String = buf.drain(..end + 2).collect();
                let mut event = "message";
                let mut data = String::new();
                for line in frame.lines() {
                    if let Some(e) = line.strip_prefix("event:") {
                        event = if e.trim() == "delta" {
                            "delta"
                        } else if e.trim() == "done" {
                            "done"
                        } else if e.trim() == "error" {
                            "error"
                        } else {
                            "other"
                        };
                    } else if let Some(d) = line.strip_prefix("data:") {
                        data.push_str(d.trim_start());
                    }
                }
                match event {
                    "delta" => {
                        let v: Value = serde_json::from_str(&data).expect("delta json");
                        out.deltas.push((
                            Instant::now(),
                            v["text"].as_str().unwrap_or_default().to_owned(),
                        ));
                    }
                    "done" | "error" => {
                        out.end = Some((event.to_owned(), data));
                    }
                    _ => {}
                }
            }
            if out.end.is_some() {
                break;
            }
        }
        out
    }
}

#[derive(Default)]
struct Sse {
    deltas: Vec<(Instant, String)>,
    end: Option<(String, String)>,
}

// --- the gatePos scale -----------------------------------------------------------

struct GateOrder {
    story_order: Vec<String>,
    eras: Vec<Value>,
    pos: HashMap<String, i64>,
}

fn gate_order() -> GateOrder {
    let path = trevor_dir().join("artifacts/chrono/gate_order.json");
    let v: Value = serde_json::from_str(
        &std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display())),
    )
    .expect("gate order json");
    let story_order: Vec<String> = serde_json::from_value(v["storyOrder"].clone()).expect("order");
    let mut pos: HashMap<String, i64> = story_order
        .iter()
        .enumerate()
        .map(|(i, s)| (s.clone(), i as i64 + 1))
        .collect();
    for (k, p) in v["sourcePos"].as_object().expect("sourcePos") {
        pos.insert(k.clone(), p.as_i64().expect("pos"));
    }
    GateOrder {
        story_order,
        eras: v["eras"].as_array().expect("eras").clone(),
        pos,
    }
}

fn panel(story: &str, pos: i64) -> Value {
    json!({ "schema": 1, "storyId": story, "position": pos, "recap": { "text": "e2e" } })
}

/// The gatePos of every cited passage (None for one with no place on the scale).
fn cited_positions(order: &GateOrder, answer: &Value) -> Vec<(String, Option<i64>)> {
    answer["citations"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| {
            let story = c["storyId"].as_str().unwrap_or_default().to_owned();
            let p = order.pos.get(&story).copied();
            (story, p)
        })
        .collect()
}

// --- the test --------------------------------------------------------------------

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "needs the local Postgres (throwaway trevor_test), trevor/target/release/ask, its artifacts and the Gemma 12B"]
async fn trevor_end_to_end_with_the_real_worker() {
    let order = gate_order();
    let pool = fresh_db().await;
    let s = state_on(&pool);
    let base = serve(&s).await;
    let client = Client {
        http: reqwest::Client::builder()
            .timeout(Duration::from_secs(600))
            .build()
            .expect("client"),
        base: base.clone(),
    };
    eprintln!("backend router on {base} (in-process), database {TEST_DB}");

    // 1. A worker with no model: the passages rung only.
    wait_for_memory(25, "loading a worker").await;
    let no_model = format!("http://127.0.0.1:{}", free_port());
    let t_load = Instant::now();
    let mut w = Worker::spawn(
        "passages",
        &base,
        &[
            "--server".into(),
            no_model,
            "--no-spawn".into(),
            "--ignore-lock".into(),
        ],
    );
    let version = await_worker(&pool, &mut w, false, Duration::from_secs(300)).await;
    let load_secs = t_load.elapsed().as_secs_f64();
    let st = client.status().await;
    assert_eq!(st["online"], true, "{st}");
    assert_eq!(st["rungs"], json!({ "cf": false, "cpu": false }));
    let (_, _, _, seen1) = heartbeat_row(&pool, &w.id()).await.unwrap();
    tokio::time::sleep(Duration::from_secs(HEARTBEAT_SECS * 2 + 1)).await;
    let (_, _, _, seen2) = heartbeat_row(&pool, &w.id()).await.unwrap();
    assert!(seen2 > seen1, "last_seen advances: {seen1} then {seen2}");
    eprintln!(
        "1. worker online in {load_secs:.1} s, corpus {version}; last_seen advanced {} ms in {} s",
        (seen2 - seen1).num_milliseconds(),
        HEARTBEAT_SECS * 2 + 1
    );

    // 2. Publish the worker's corpus version: three panels at their gatePos,
    // the real story order and eras.
    let era = |i: usize| order.eras[i]["position"].as_i64().expect("era position");
    let story_at = |p: i64| order.story_order[(p - 1) as usize].clone();
    let early_era = era(1);
    let late_era = era(order.eras.len() - 1);
    let (first, early, late) = (story_at(1), story_at(early_era), story_at(late_era));
    let mut panels = serde_json::Map::new();
    for (sid, p) in [(&first, 1), (&early, early_era), (&late, late_era)] {
        panels.insert(sid.clone(), panel(sid, p));
    }
    let st = client
        .publish(&json!({
            "version": version, "activate": true, "gateReport": { "spoilerLeaks": 0, "e2e": true },
            "manifest": { "builtFrom": "e2e", "storyOrder": order.story_order, "eras": order.eras },
            "panels": panels
        }))
        .await;
    assert_eq!(st, 200, "publish");
    assert_eq!(client.status().await["activeVersion"], json!(version));
    eprintln!(
        "2. published {version}: {first} at 1, {early} at {early_era}, {late} at {late_era}, {} eras",
        order.eras.len()
    );

    // 3. Passages: the early reader's era and passages; the late reader's.
    let question = "What happened to Talulah?";
    let (early_user, early_bearer) = new_user(&pool).await;
    set_progress(&pool, early_user, &[&first, &early], &[]).await;
    let (late_user, late_bearer) = new_user(&pool).await;
    set_progress(&pool, late_user, &[&late], &[]).await;

    let mut seen_late_pos = 0;
    for (who, bearer, want) in [
        ("early", &early_bearer, early_era),
        ("late", &late_bearer, late_era),
    ] {
        let t = Instant::now();
        let (st, r) = client.ask(bearer, question).await;
        assert_eq!(st, 202, "{who} ask: {r}");
        assert_eq!(r["rung"], "passages", "{r}");
        assert_eq!(r["spoilerSafeUpTo"]["position"], json!(want), "{r}");
        let id = r["jobId"].as_str().expect("job id").to_owned();
        let sse = client.stream(bearer, &id, Duration::from_secs(120)).await;
        let secs = t.elapsed().as_secs_f64();
        assert_eq!(sse.end.as_ref().map(|e| e.0.as_str()), Some("done"));
        let j = client.job(bearer, &id).await;
        assert_eq!(j["status"], "done", "{j}");
        let ans = &j["answer"];
        assert_eq!(ans["source"], "passages");
        assert_eq!(ans["eraPos"], json!(want));
        let horizon: Option<i32> =
            sqlx::query_scalar("SELECT horizon_pos FROM trevor_jobs WHERE id = $1::uuid")
                .bind(&id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(horizon, Some(want as i32), "the job's horizonPos");
        let cited = cited_positions(&order, &ans["answer"]);
        assert!(!cited.is_empty(), "{who}: passages cited");
        for (story, p) in &cited {
            let p = p.unwrap_or_else(|| panic!("{who}: {story} has no gatePos"));
            assert!(
                p <= want,
                "{who}: {story} at {p} is past the horizon {want}"
            );
            if who == "late" {
                seen_late_pos = seen_late_pos.max(p);
            }
        }
        eprintln!(
            "3. {who}: era {} ({want}), {} passages at gatePos <= {}, {secs:.2} s ask to done",
            r["spoilerSafeUpTo"]["eraId"],
            cited.len(),
            cited.iter().filter_map(|c| c.1).max().unwrap_or(0)
        );
    }
    // A passages answer is stored for its job but never served from the cache.
    let (st, again) = client.ask(&early_bearer, question).await;
    assert_eq!(
        (st, again["rung"].as_str()),
        (202, Some("passages")),
        "{again}"
    );
    let id = again["jobId"].as_str().expect("job").to_owned();
    client
        .stream(&early_bearer, &id, Duration::from_secs(120))
        .await;
    assert!(
        seen_late_pos > early_era,
        "the late reader sees a passage past the early horizon ({seen_late_pos})"
    );
    w.stop();
    drop(w);

    // 4. The cpu rung with the local Gemma 12B.
    if std::env::var("TREVOR_E2E_CPU").as_deref() == Ok("0") {
        eprintln!("4. skipped (TREVOR_E2E_CPU=0)");
        return;
    }
    let free = wait_for_memory(30, "starting the 12B").await;
    assert!(
        !sh("pgrep", &["-x", "llama-server"]).0,
        "a llama-server is already running"
    );
    assert_eq!(lock_holder(), None, "the model lock is held");
    let llama_port = free_port();
    let t_load = Instant::now();
    let mut w = Worker::spawn(
        "cpu",
        &base,
        &[
            "--server".into(),
            format!("http://127.0.0.1:{llama_port}"),
            "--no-wait".into(),
        ],
    );
    let v2 = await_worker(&pool, &mut w, true, Duration::from_secs(420)).await;
    assert_eq!(v2, version);
    assert!(
        !w.llama.is_empty(),
        "the worker started its own llama-server"
    );
    assert!(
        lock_holder().is_some_and(|l| l.starts_with(&w.child.id().to_string())),
        "the worker holds the model lock: {:?}",
        lock_holder()
    );
    eprintln!(
        "4. cpu worker online in {:.1} s (llama-server {:?} on :{llama_port}, free memory {free}% before)",
        t_load.elapsed().as_secs_f64(),
        w.llama
    );

    let question = "Who leads Reunion?";
    let t0 = Instant::now();
    let (st, r) = client.ask(&late_bearer, question).await;
    let ask_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(st, 202, "{r}");
    assert_eq!(r["rung"], "cpu", "{r}");
    let id = r["jobId"].as_str().expect("job").to_owned();
    let depth = client.status().await["queueDepth"]["cpu"].clone();
    assert_eq!(depth, json!(1), "the job counts on the cpu queue");
    // busyJobId while the job runs (heartbeats every 2 s, from their own task).
    let busy_watch = {
        let pool = pool.clone();
        let wid = w.id();
        let id = id.clone();
        tokio::spawn(async move {
            let t = Instant::now();
            while t.elapsed() < Duration::from_secs(300) {
                if let Some((_, _, Some(b), _)) = heartbeat_row(&pool, &wid).await
                    && b.to_string() == id
                {
                    return true;
                }
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            false
        })
    };
    let sse = client
        .stream(&late_bearer, &id, Duration::from_secs(600))
        .await;
    let done_s = t0.elapsed().as_secs_f64();
    let (event, data) = sse.end.clone().expect("a terminal event");
    assert_eq!(event, "done", "{data}");
    assert!(!sse.deltas.is_empty(), "SSE deltas arrived");
    let first_delta_s = (sse.deltas[0].0 - t0).as_secs_f64();
    let streamed: String = sse.deltas.iter().map(|d| d.1.as_str()).collect();
    assert!(
        busy_watch.await.unwrap(),
        "the heartbeat named the job as busy"
    );

    let j = client.job(&late_bearer, &id).await;
    assert_eq!(j["status"], "done", "{j}");
    let ans = &j["answer"];
    assert_eq!(
        (ans["rung"].as_str(), ans["source"].as_str()),
        (Some("cpu"), Some("live")),
        "{ans}"
    );
    assert_eq!(ans["answer"]["profile"], "slim");
    let text = ans["answer"]["answer"]
        .as_str()
        .unwrap_or_default()
        .to_owned();
    assert!(!text.trim().is_empty(), "an answer");
    for (story, p) in cited_positions(&order, &ans["answer"]) {
        let p = p.unwrap_or_else(|| panic!("{story} has no gatePos"));
        assert!(p <= late_era, "{story} at {p} past {late_era}");
    }
    let (rung, cached): (String, bool) = sqlx::query_as(
        "SELECT a.rung, a.cached FROM trevor_jobs j JOIN trevor_answers a ON a.id = j.answer_id WHERE j.id = $1::uuid",
    )
    .bind(&id)
    .fetch_one(&pool)
    .await
    .expect("stored answer");
    assert_eq!((rung.as_str(), cached), ("cpu", true));
    let timings = &ans["answer"]["timings"];
    eprintln!(
        "4. cpu: ask {ask_ms:.0} ms, first delta {first_delta_s:.2} s, done {done_s:.2} s after the ask; \
         {} deltas, {} chars streamed, answer {} chars ({}), worker timings {timings}, tokens {}/{}",
        sse.deltas.len(),
        streamed.chars().count(),
        text.chars().count(),
        if streamed.trim() == text.trim() {
            "equal to the stream"
        } else {
            "differs from the stream"
        },
        ans["answer"]["tokensIn"],
        ans["answer"]["tokensOut"]
    );
    eprintln!("   answer: {text}");

    // The identical second ask is a cache hit.
    let t1 = Instant::now();
    let (st, again) = client.ask(&late_bearer, question).await;
    let hit_ms = t1.elapsed().as_secs_f64() * 1000.0;
    assert_eq!(st, 200, "{again}");
    assert_eq!(again["rung"], "cache");
    assert_eq!(again["jobId"], Value::Null);
    assert_eq!(
        again["answer"]["answer"]["answer"].as_str(),
        Some(text.as_str())
    );
    eprintln!("4. cache hit in {hit_ms:.1} ms");

    // Stop the worker and its llama-server by PID.
    let llama = w.llama.clone();
    w.stop();
    drop(w);
    for pid in &llama {
        assert!(!alive(*pid), "llama-server {pid} stopped");
    }
    assert!(
        lock_holder().is_none(),
        "the model lock is released: {:?}",
        lock_holder()
    );
    eprintln!("4. worker and llama-server {llama:?} stopped; the model lock is free");
}
