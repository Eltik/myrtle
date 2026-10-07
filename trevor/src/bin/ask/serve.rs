//! `ask --serve` and `ask --worker URL`: the ask runtime loaded once (tools, indexes, embedder, reranker, a
//! llama-server) and every question a library call, so a job costs the answer alone and not the ~15 s reload of a
//! one-shot `ask` (design/trevor-integration.md sections 3 and 8, step 1).
//!
//! `--serve` binds 127.0.0.1:`--port` (8090) and speaks a minimal HTTP/1.1 JSON API, one request per connection:
//! `GET /health` and `POST /v1/ask {question, horizon?, server?, flags?}`. Jobs run one at a time (one model slot);
//! a connection that arrives during an answer waits in the listen backlog. `--worker URL` instead pulls jobs from a
//! myrtle backend (section 4's worker protocol, not yet exercised against a backend): `POST {URL}/api/trevor/worker/next`
//! (204 or an empty body = no job; 200 = `{jobId, question, horizon?, server?, flags?}`), `POST .../worker/result`
//! `{jobId, workerId, ok, result | error}` and `POST .../worker/heartbeat` every 30 s, each with the `x-service-key`
//! header from `TREVOR_SERVICE_KEY`. Both work the same on the Mac and on a VPS: nothing here names a host.
//!
//! Without a horizon a job takes exactly the path of the one-shot `ask "question"` (route first, P4 only when the route
//! or the source keywords ask for it, then `answer_one` with the route), so the answers match it byte for byte under
//! `--no-cache-prompt`. With a horizon (a story or group id in the EN release order) retrieval returns only chunks
//! released at or before it, and everything built from every story is left out: table answers (the question is
//! answered from the passages instead), topic summaries and entries, dossiers, the game-data passage, the overview,
//! canon evidence, the identity and chronology notes and the evidence composition. Per-era rebuilds of those
//! generated summaries are later work.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use trevor::goldgen::llm::Llm;
use trevor::search::horizon::ReleaseOrder;
use trevor::search::runtime::Runtime;
use trevor::tools::Route;

use crate::answer::{Tables, answer_one};
use crate::chrono::Chrono;
use crate::cli::{Args, RouterKind, deep_for, p4_dir, retrieval_plan, router_source};
use crate::detect::source_kind;
use crate::retrieval::{ART_DIR, Names, Runtimes, load_names};
use crate::server::{Spawned, connect_or_spawn};

/// The only server with a corpus today; CN is a second corpus version stream later (section 10).
const SERVER: &str = "en";

/// Request flags that would change what the service loaded; a job may not give them.
const LOAD_FLAGS: &[&str] = &["--corpus", "--lore", "--router", "--serve", "--worker", "--port", "--batch", "--out", "--server",
                              "--model", "--no-game-text", "--art", "--rerank-add", "--model-dir", "--query-onnx"];

/// One question: `POST /v1/ask` body, or a worker job's fields.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AskRequest {
    pub(crate) question: String,
    /// A story id or group id: only what was released at or before it may be used.
    #[serde(default)]
    pub(crate) horizon: Option<String>,
    /// "en" (the default) is the only server served.
    #[serde(default)]
    pub(crate) server: Option<String>,
    /// Extra `ask` flags for this job (`["--no-answer-check"]`); flags that change what is loaded are refused.
    #[serde(default)]
    pub(crate) flags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Citation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) story_id: Option<String>,
    pub(crate) chunk_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) line_end: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Timings {
    pub(crate) route_ms: f64,
    pub(crate) answer_ms: f64,
    pub(crate) total_ms: f64,
}

/// The structured answer of one job.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AskResult {
    pub(crate) answer: String,
    /// The cited passages, in citation order; generated passages (topic, dossier) carry no story id.
    pub(crate) citations: Vec<Citation>,
    /// Every passage given to the model, in passage order (the `[n]` of the answer is its position + 1).
    pub(crate) passages: Vec<String>,
    /// The router's decision; None under `--no-route`.
    pub(crate) route: Option<Route>,
    /// True when a table tool answered (no model answer call).
    pub(crate) table: bool,
    /// The job's extra flags, as applied.
    pub(crate) flags: Vec<String>,
    /// The question form and evidence query, when the form call ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) form: Option<String>,
    pub(crate) horizon: Option<String>,
    pub(crate) server: String,
    pub(crate) corpus_version: String,
    pub(crate) invalid_citations: usize,
    pub(crate) prompt_tokens: u64,
    pub(crate) timings: Timings,
}

/// A refused or failed job: the HTTP status and the message.
type JobError = (u16, String);

fn bad(e: impl std::fmt::Display) -> JobError {
    (400, e.to_string())
}

fn failed(e: impl std::fmt::Display) -> JobError {
    (500, format!("{e:#}"))
}

/// The loaded service.
pub(crate) struct Service {
    a: Args,
    argv: Vec<String>,
    corpus_default: bool,
    tables: Tables,
    rts: Runtimes,
    llm: Llm,
    names: HashMap<String, Names>,
    chrono: Chrono,
    order: Arc<ReleaseOrder>,
    corpus_version: String,
    /// `tools.topics_deep` as loaded, restored for a `--no-route` job (the one-shot `ask` never sets it then).
    deep0: bool,
    main_corpus: PathBuf,
    _server: Option<Spawned>,
}

/// The corpus version: the first 16 hex of the sha256 of "dir chunks-sha" lines for every corpus a job may read (the
/// main corpus, P4/P4x and the art corpus when built), prefixed by the server.
pub(crate) fn corpus_version(parts: &[(String, String)]) -> String {
    let mut h = Sha256::new();
    for (dir, sha) in parts {
        h.update(format!("{dir} {sha}\n"));
    }
    let hex: String = h.finalize().iter().take(8).map(|b| format!("{b:02x}")).collect();
    format!("{SERVER}-{hex}")
}

/// The `chunksSha` a corpus directory's vectors were built from (every derived index records it).
fn recorded_sha(dir: &str) -> Option<String> {
    let meta: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(PathBuf::from(dir).join("vectors.meta.json")).ok()?).ok()?;
    meta["chunksSha"].as_str().map(str::to_owned)
}

impl Service {
    /// Load everything once: the indexes of the main corpus (P3b by default), the llama-server (connected or started
    /// under the model lock, stopped when the service ends) and the release order.
    pub(crate) async fn load(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<Self> {
        let main = Runtime::load(&a.runtime, true, true, a.rerank_add > 0)?;
        let mut parts = vec![(a.runtime.corpus.display().to_string(), main.store.chunks_sha.clone())];
        for dir in std::iter::once(p4_dir(&a)).chain([ART_DIR].into_iter().filter(|_| a.art)) {
            if let Some(sha) = recorded_sha(dir).filter(|_| std::path::Path::new(dir) != a.runtime.corpus) {
                parts.push((dir.to_owned(), sha));
            }
        }
        let corpus_version = corpus_version(&parts);
        let rts = Runtimes { main, p4: None, art: None, p4_dir: p4_dir(&a), switch: corpus_default && router_source(&a),
                             args: a.runtime.clone(), rerank: a.rerank_add > 0, horizon: None };
        let (llm, server) = connect_or_spawn(&a).await?;
        let order = Arc::new(ReleaseOrder::load(std::path::Path::new("."))?);
        eprintln!("horizon: {} unplaced sources dated (artifacts/chrono/source_dates.json)", order.dated_sources());
        Ok(Self { argv: std::env::args().collect(), corpus_default, deep0: tables.tools.topics_deep, main_corpus: a.runtime.corpus.clone(),
                  tables, rts, llm, names: load_names(), chrono, order, corpus_version, a, _server: server })
    }

    /// The job's arguments: the service's own, plus the job's flags (refusing the ones that change what is loaded).
    fn args_for(&self, flags: &[String]) -> Result<Args, JobError> {
        if flags.is_empty() {
            return Ok(self.a.clone());
        }
        for f in flags {
            let name = f.split('=').next().unwrap_or_default();
            if !f.starts_with("--") || LOAD_FLAGS.contains(&name) {
                return Err(bad(format!("flag {f:?} is not allowed in a job")));
            }
        }
        let m = <Args as clap::CommandFactory>::command().try_get_matches_from(self.argv.iter().chain(flags)).map_err(bad)?;
        let mut a = <Args as clap::FromArgMatches>::from_arg_matches(&m).map_err(bad)?;
        a.relation_rule = !a.no_relation_rule;
        a.runtime = self.a.runtime.clone();
        Ok(a)
    }

    /// Answer one job.
    pub(crate) async fn ask(&mut self, req: AskRequest) -> Result<AskResult, JobError> {
        let t0 = Instant::now();
        let server = req.server.as_deref().unwrap_or(SERVER);
        if server != SERVER {
            return Err(bad(format!("server {server:?} has no corpus (only \"{SERVER}\")")));
        }
        let q = req.question.trim().to_owned();
        if q.is_empty() {
            return Err(bad("empty question"));
        }
        let mut a = self.args_for(&req.flags)?;
        let horizon = match &req.horizon {
            Some(h) => Some(Arc::new(self.order.horizon(h).map_err(bad)?)),
            None => None,
        };
        if horizon.is_some() {
            // Built from every story: identity links, the chronology of the subject, the evidence composition.
            a.identity_notes = false;
            a.identity_labels = false;
            a.identity_expand = false;
            a.compose_evidence = false;
            a.no_chrono_subject = true;
        }
        self.rts.set_horizon(horizon);
        self.tables.topic = None;
        self.tables.route = None;
        self.tables.tools.topics_deep = self.deep0;
        let mut out = AskResult { answer: String::new(), citations: Vec::new(), passages: Vec::new(), route: None, table: false,
                                  flags: req.flags.clone(), form: None, horizon: req.horizon.clone(), server: SERVER.to_owned(),
                                  corpus_version: self.corpus_version.clone(), invalid_citations: 0, prompt_tokens: 0,
                                  timings: Timings { route_ms: 0.0, answer_ms: 0.0, total_ms: 0.0 } };
        // As the one-shot `ask "question"` does in `main`: P4 for a source question when the router names no source,
        // then route, then P4 when the route's source asks for it.
        let mut use_p4 = self.corpus_default && !router_source(&a) && source_kind(&q).is_some();
        if !a.no_route {
            let llm = matches!(a.router, RouterKind::Model | RouterKind::Hybrid).then_some(&self.llm);
            self.tables.tools.topics_deep = deep_for(&a, &self.tables.tools, &q);
            let r = crate::routing::route_question(&q, &a, &self.tables.tools, self.tables.knn.as_mut(), llm).await.map_err(failed)?;
            out.timings.route_ms = r.ms;
            let route = match r.text {
                Some(text) if !self.rts.horizon_set() => {
                    out.answer = text;
                    out.route = Some(r.route);
                    out.table = true;
                    out.timings.total_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    return Ok(out);
                }
                // Under a horizon a table answer is not served; the passages answer instead.
                Some(_) => Route::retrieve(),
                None => {
                    self.tables.topic = if self.rts.horizon_set() { None } else { r.topic };
                    r.route
                }
            };
            if self.corpus_default && retrieval_plan(&q, &a, Some(&route)).0 == Some(true) {
                use_p4 = true;
            }
            out.route = Some(route.clone());
            self.tables.route = Some(route);
        }
        let mut once = a.clone();
        once.routed = true;
        let swapped = if use_p4 { self.swap_p4().map_err(failed)? } else { false };
        let res = answer_one(&mut self.rts, &self.llm, &self.names, &self.chrono, &q, &once, &mut self.tables).await;
        if swapped {
            self.swap_back();
        }
        let ans = res.map_err(failed)?;
        out.citations = ans.cited.iter().map(|id| self.citation(id)).collect();
        out.answer = ans.answer;
        out.passages = ans.passages;
        out.form = ans.form;
        out.invalid_citations = ans.invalid_citations;
        out.prompt_tokens = ans.prompt_tokens;
        out.timings.answer_ms = ans.ms;
        out.timings.total_ms = t0.elapsed().as_secs_f64() * 1000.0;
        Ok(out)
    }

    /// Make P4 the main corpus for this job, as the one-shot `ask` loads it as its only corpus: loaded on first use,
    /// swapped with the main corpus, swapped back after. False when the main corpus already is P4.
    fn swap_p4(&mut self) -> Result<bool> {
        let dir = PathBuf::from(self.rts.p4_dir);
        if self.rts.args.corpus == dir {
            return Ok(false);
        }
        if self.rts.p4.is_none() {
            let mut ra = self.rts.args.clone();
            ra.corpus.clone_from(&dir);
            self.rts.p4 = Some(Runtime::load(&ra, true, true, self.rts.rerank)?);
        }
        let p4 = self.rts.p4.as_mut().expect("loaded above");
        std::mem::swap(&mut self.rts.main, p4);
        self.rts.args.corpus = dir;
        // A P4 loaded just now has no horizon yet; `answer_one` also reads `rts.main` directly, not only through `pick`.
        self.rts.set_horizon(self.rts.horizon.clone());
        Ok(true)
    }

    fn swap_back(&mut self) {
        if let Some(p4) = self.rts.p4.as_mut() {
            std::mem::swap(&mut self.rts.main, p4);
        }
        self.rts.args.corpus.clone_from(&self.main_corpus);
    }

    /// A cited passage's story and line span, from whichever loaded corpus holds it.
    fn citation(&self, id: &str) -> Citation {
        let rts = std::iter::once(&self.rts.main).chain(self.rts.p4.as_ref()).chain(self.rts.art.as_ref());
        for rt in rts {
            if let Some(c) = rt.store.row(id).map(|r| &rt.store.chunks[r]) {
                return Citation { story_id: Some(c.story_id.clone()), chunk_id: id.to_owned(), line_start: Some(c.line_start),
                                  line_end: Some(c.line_end) };
            }
        }
        Citation { story_id: None, chunk_id: id.to_owned(), line_start: None, line_end: None }
    }

    fn health(&self) -> serde_json::Value {
        serde_json::json!({"ok": true, "server": SERVER, "corpusVersion": self.corpus_version, "model": self.llm.model})
    }
}

/// `--serve` or `--worker`: load once, then serve until SIGINT or SIGTERM (the llama-server this process started, if
/// any, stops with it).
pub(crate) async fn run(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<()> {
    let worker = a.worker.clone();
    let port = a.port;
    let mut svc = Service::load(a, corpus_default, tables, chrono).await?;
    match worker {
        Some(base) => pull(&mut svc, &base).await,
        None => listen(&mut svc, port).await,
    }
}

async fn listen(svc: &mut Service, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await.with_context(|| format!("binding 127.0.0.1:{port}"))?;
    eprintln!("serve: http://127.0.0.1:{port} ({}, model {})", svc.corpus_version, svc.llm.model);
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        let mut s = tokio::select! {
            r = listener.accept() => r?.0,
            _ = tokio::signal::ctrl_c() => break,
            _ = term.recv() => break,
        };
        if let Err(e) = handle(svc, &mut s).await {
            eprintln!("serve: {e:#}");
        }
    }
    eprintln!("serve: stopping");
    Ok(())
}

/// One request: read it, answer it, close.
async fn handle(svc: &mut Service, s: &mut TcpStream) -> Result<()> {
    let (method, path, body) = tokio::time::timeout(Duration::from_secs(10), read_request(s)).await.context("request timed out")??;
    let (code, json) = match (method.as_str(), path.as_str()) {
        ("GET", "/health") => (200, svc.health()),
        ("POST", "/v1/ask") => match serde_json::from_slice::<AskRequest>(&body) {
            Err(e) => (400, serde_json::json!({"error": e.to_string()})),
            Ok(req) => match svc.ask(req).await {
                Ok(r) => (200, serde_json::to_value(r)?),
                Err((code, e)) => (code, serde_json::json!({"error": e})),
            },
        },
        _ => (404, serde_json::json!({"error": "not found"})),
    };
    let body = serde_json::to_vec(&json)?;
    let reason = match code { 200 => "OK", 400 => "Bad Request", 404 => "Not Found", _ => "Internal Server Error" };
    let head = format!("HTTP/1.1 {code} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
    s.write_all(head.as_bytes()).await?;
    s.write_all(&body).await?;
    s.shutdown().await?;
    Ok(())
}

/// The request line's method and path and the body (by `Content-Length`, at most 64 KiB).
pub(crate) async fn read_request(s: &mut (impl AsyncReadExt + Unpin)) -> Result<(String, String, Vec<u8>)> {
    const MAX: usize = 64 * 1024;
    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];
    let end = loop {
        let n = s.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed before the headers ended");
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        if buf.len() > MAX {
            bail!("headers too large");
        }
    };
    let head = String::from_utf8_lossy(&buf[..end]).into_owned();
    let mut lines = head.lines();
    let mut first = lines.next().unwrap_or_default().split_whitespace();
    let (method, path) = (first.next().unwrap_or_default().to_owned(), first.next().unwrap_or_default().to_owned());
    let len = lines.filter_map(|l| l.split_once(':')).find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
        .map_or(Ok(0), |(_, v)| v.trim().parse::<usize>()).context("bad Content-Length")?;
    if len > MAX {
        bail!("body too large");
    }
    let mut body = buf[end + 4..].to_vec();
    while body.len() < len {
        let n = s.read(&mut tmp).await?;
        if n == 0 {
            bail!("connection closed before the body ended");
        }
        body.extend_from_slice(&tmp[..n]);
    }
    body.truncate(len);
    Ok((method, path, body))
}

/// A worker job: the job id and the question's fields.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkerJob {
    job_id: String,
    question: String,
    #[serde(default)]
    horizon: Option<String>,
    #[serde(default)]
    server: Option<String>,
    #[serde(default)]
    flags: Vec<String>,
}

/// `--worker`: long-poll the backend for jobs, answer each, post the result; a heartbeat at most every 30 s.
async fn pull(svc: &mut Service, base: &str) -> Result<()> {
    let key = std::env::var("TREVOR_SERVICE_KEY").context("--worker needs TREVOR_SERVICE_KEY")?;
    let base = base.trim_end_matches('/');
    let http = reqwest::Client::builder().timeout(Duration::from_secs(90)).build()?;
    let worker_id = format!("trevor-{}", std::process::id());
    let post = |path: &str, body: serde_json::Value| {
        http.post(format!("{base}/api/trevor/worker/{path}")).header("x-service-key", &key).json(&body).send()
    };
    eprintln!("worker {worker_id}: polling {base} ({})", svc.corpus_version);
    let mut beat: Option<Instant> = None;
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        if beat.is_none_or(|t| t.elapsed() >= Duration::from_secs(30)) {
            let hb = serde_json::json!({"workerId": worker_id, "server": SERVER, "corpusVersion": svc.corpus_version, "slots": 1});
            if let Err(e) = post("heartbeat", hb).await {
                eprintln!("worker: heartbeat failed: {e}");
            }
            beat = Some(Instant::now());
        }
        let next = serde_json::json!({"workerId": worker_id, "server": SERVER, "corpusVersion": svc.corpus_version, "slots": 1});
        let resp = tokio::select! {
            r = post("next", next) => r,
            _ = tokio::signal::ctrl_c() => break,
            _ = term.recv() => break,
        };
        let job = match resp {
            Ok(r) if r.status() == reqwest::StatusCode::NO_CONTENT => continue,
            Ok(r) if r.status().is_success() => match r.bytes().await {
                Ok(b) if b.is_empty() => continue,
                Ok(b) => serde_json::from_slice::<WorkerJob>(&b),
                Err(e) => { eprintln!("worker: reading a job failed: {e}"); continue; }
            },
            Ok(r) => { eprintln!("worker: next answered {}", r.status()); tokio::time::sleep(Duration::from_secs(10)).await; continue; }
            Err(e) => { eprintln!("worker: next failed: {e}"); tokio::time::sleep(Duration::from_secs(10)).await; continue; }
        };
        let job = match job {
            Ok(j) => j,
            Err(e) => { eprintln!("worker: unreadable job: {e}"); continue; }
        };
        let req = AskRequest { question: job.question, horizon: job.horizon, server: job.server, flags: job.flags };
        let body = match svc.ask(req).await {
            Ok(r) => serde_json::json!({"jobId": job.job_id, "workerId": worker_id, "ok": true, "result": r}),
            Err((code, e)) => serde_json::json!({"jobId": job.job_id, "workerId": worker_id, "ok": false, "status": code, "error": e}),
        };
        if let Err(e) = post("result", body).await {
            eprintln!("worker: posting the result of {} failed: {e}", job.job_id);
        }
    }
    eprintln!("worker: stopping");
    Ok(())
}
