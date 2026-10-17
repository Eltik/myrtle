//! `ask --serve` and `ask --worker URL`: the ask runtime loaded once (tools, indexes, embedder, reranker, a
//! llama-server) and every question a library call, so a job costs the answer alone and not the ~15 s reload of a
//! one-shot `ask` (design/trevor-integration.md sections 3 and 8, step 1).
//!
//! `--serve` binds 127.0.0.1:`--port` (8090) and speaks a minimal HTTP/1.1 JSON API, one request per connection:
//! `GET /health` and `POST /v1/ask {question, horizon?, horizonPos?, server?, flags?, profile?}`. Jobs run one at a
//! time (one model slot); a connection that arrives during an answer waits in the listen backlog.
//!
//! `--worker URL` pulls jobs from a myrtle backend by design/trevor-contract-v1.md section 3 (2026-10-16), with the
//! `x-trevor-worker-key` header from `TREVOR_WORKER_KEY`:
//! - `POST {URL}/api/trevor/worker/next {workerId}` long-polls; 204 = no job; 200 = `{jobId, question, server,
//!   horizonPos, rung, profile, flags}`.
//! - `POST .../worker/heartbeat {workerId, rungs: {cf, cpu}, corpusVersion, busyJobId}` every 30 s
//!   (`TREVOR_HEARTBEAT_SECS`) from its own task, so it keeps going mid-job. `cf` = a Cloudflare model is configured and
//!   the last canary (`ask --canary`, artifacts/canary/cf.json, at most 8 days old) passed one of them; `cpu` = the
//!   local llama-server answers `/health`.
//! - `POST .../worker/jobs/{jobId}/delta {workerId, seq, text}` as the answer streams (`--no-stream` turns it off). A
//!   later model call of the same job (an answer extension, a check's second ask) is streamed only where it continues
//!   the text already sent; the result is authoritative.
//! - `POST .../worker/result {jobId, workerId, ok, error?, result?}`, result = `{answer, rung, profile, source,
//!   citations [{n, storyId, chunkId, lineStart, lineEnd, text}], route, form, corpusVersion, tokensIn, tokensOut,
//!   timings}`: only the cited passages, each text cut to 600 characters at a line break. A failed POST is retried after
//!   1, 4 and 16 s, then spooled to `TREVOR_SPOOL_DIR` (artifacts/spool/) and sent again when the worker starts.
//!
//! Rungs: `cf` answers with the Cloudflare model (full profile; router, form and flags as validated JSON prompts),
//! `cpu` with the local llama-server (the job's profile; one that names none gets `mid` since contract v1.1 section 4,
//! `TREVOR_CPU_PROFILE=slim` restores the v1 default), `passages` with no model: the top `--passages-n` (5) passages
//! within the horizon, `source: "passages"`, which is also the answer when a model rung fails. Every citation carries
//! `sha`, the chunk's `contentSha` (sha256 of its text, first 16 hex; the same hash of the passage text for a generated
//! passage), for the cache carry-over of contract v1.1 section 3b.
//!
//! Contract v1.1 section 2.1, the power guard (`guard.rs`, `TREVOR_GUARD=0` turns it off): while it is tripped the
//! heartbeat reports `rungs.cpu = false` and `guard {state, reason, since}` (sent at once on every change, not only every
//! 30 s), a claimed `cpu` job is answered with passages without touching the model, a trip during a `cpu` answer drops
//! the model call and answers with passages, and a memory trip stops the llama-server this worker started (started again
//! on recovery). The worker keeps claiming: the backend settles a claimed job's rung by the last heartbeat, and `cf`
//! and `passages` jobs run no local model.
//!
//! Several workers on one host (contract v1.1 section 1d): run ONE llama-server with `-np N` (N parallel slots;
//! `--kv-unified-per-slot` keeps each slot's context, 8,192 is enough for `mid`, 12,288 for `mid6k`) under its own
//! supervisor, and N `ask --worker URL --share-server` processes with distinct `TREVOR_WORKER_ID`s. `--share-server`
//! connects without waiting for the model lock, never starts a server and never stops one (so a memory trip cannot stop
//! a shared server; the guard still stops claiming model work). Without it each worker waits for the lock and starts
//! its own server when none answers, as before. Measured on the M5 Pro (Metal) with E2B and `mid`, 10 gold questions,
//! two runs each (2026-10-17): one worker on `-np 1` 26.4 and 28.8 answers a minute (median 2.1 to 2.3 s a job), two
//! workers on `-np 2` 33.8 and 33.4 (median 3.6 to 3.7 s a job), so +16% to +28% throughput for +60% latency; a VPS core
//! without a GPU is not measured.
//!
//! Without a horizon a job takes exactly the path of the one-shot `ask "question"` (route first, P4 only when the route
//! or the source keywords ask for it, then `answer_one` with the route), so the answers match it byte for byte under
//! `--no-cache-prompt`. With a horizon (a gatePos, or a story or group id turned into one; src/search/horizon.rs)
//! retrieval returns only chunks released at or before it, and everything built from every story is left out: table
//! answers (the question is answered from the passages instead), topic summaries and entries, dossiers, the game-data
//! passage, the overview, canon evidence, the identity and chronology notes and the evidence composition.

use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc::UnboundedReceiver;
use trevor::goldgen::llm::{Llm, OpenAiConfig, StreamEvent, UsageSnapshot};
use trevor::search::horizon::{Horizon, ReleaseOrder};
use trevor::search::pipeline::{Mode, RetrievalConfig};
use trevor::search::runtime::Runtime;
use trevor::tools::{Route, ToolOptions};

use crate::answer::{Tables, answer_one, passage_label, route_checked};
use crate::chrono::Chrono;
use crate::cli::{Args, Profile, RetrievalKind, RouterKind, apply_profile, deep_for, p4_dir, retrieval_plan, router_source, tool_options};
use crate::detect::source_kind;
use crate::guard::{self, GuardConfig, GuardView};
use crate::retrieval::{ART_DIR, Names, Runtimes, load_names, load_runtime};
use crate::server::{Spawned, cloudflare, connect_local, connect_or_spawn, openai_llm};

/// The only server with a corpus today; CN is a second corpus version stream later (section 10).
const SERVER: &str = "en";

/// Request flags that would change what the service loaded; a job may not give them.
const LOAD_FLAGS: &[&str] = &["--corpus", "--lore", "--router", "--serve", "--worker", "--port", "--batch", "--out", "--server",
                              "--model", "--no-game-text", "--art", "--rerank-add", "--model-dir", "--query-onnx", "--allow-query-model-mismatch", "--retrieval",
                              "--backend", "--canary", "--gate-order", "--horizon-check", "--json-check", "--no-stream"];

/// One question: `POST /v1/ask` body, or a worker job's fields.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AskRequest {
    pub(crate) question: String,
    /// A story id or group id: only what was released at or before it may be used.
    #[serde(default)]
    pub(crate) horizon: Option<String>,
    /// A gatePos (contract section 1); with `horizon` too, the lower of the two.
    #[serde(default, rename = "horizonPos")]
    pub(crate) horizon_pos: Option<u32>,
    /// "en" (the default) is the only server served.
    #[serde(default)]
    pub(crate) server: Option<String>,
    /// Extra `ask` flags for this job (`["--no-answer-check"]`); flags that change what is loaded are refused.
    #[serde(default)]
    pub(crate) flags: Vec<String>,
    /// "full", "slim", "mid" or "mid6k"; the service's `--profile` when absent.
    #[serde(default)]
    pub(crate) profile: Option<String>,
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

/// One answered job: the `/v1/ask` result and what a worker result adds to it.
pub(crate) struct Job {
    pub(crate) result: AskResult,
    /// The text of each generated passage, by id.
    generated: BTreeMap<String, String>,
    /// The model calls, tokens and JSON validations of this job.
    pub(crate) usage: UsageSnapshot,
    profile: Profile,
    /// "live" (a model or table answer) or "passages" (no model).
    source: &'static str,
}

/// A refused or failed job: the HTTP status and the message.
type JobError = (u16, String);

fn bad(e: impl std::fmt::Display) -> JobError {
    (400, e.to_string())
}

fn failed(e: impl std::fmt::Display) -> JobError {
    (500, format!("{e:#}"))
}

/// What a service connects to at load.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Need {
    /// `--serve`: the answer model (`--backend`), or fail.
    Model,
    /// `--worker`: the local llama-server when it can be had; the `cpu` rung is off without it.
    Local,
    /// `--canary`: no local model.
    Nothing,
}

/// The loaded service.
pub(crate) struct Service {
    a: Args,
    argv: Vec<String>,
    corpus_default: bool,
    tables: Tables,
    rts: Runtimes,
    /// The answer model of `--serve`, or the worker's local llama-server (the `cpu` rung).
    llm: Option<Llm>,
    /// The Cloudflare model list (the `cf` rung), when configured.
    cf: Option<Llm>,
    names: HashMap<String, Names>,
    chrono: Chrono,
    order: Arc<ReleaseOrder>,
    corpus_version: String,
    /// `tools.topics_deep` as loaded, restored for a `--no-route` job (the one-shot `ask` never sets it then).
    deep0: bool,
    /// `tools.opts` as loaded, restored after a job whose profile changed the router.
    opts0: ToolOptions,
    main_corpus: PathBuf,
    server: Option<Spawned>,
    /// The power guard stopped the llama-server this worker started (a memory trip); it starts again on recovery.
    guard_stopped: bool,
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

/// Generated corpus kinds a `passages` answer leaves out: it serves source text only.
const GENERATED_KINDS: &[&str] = &["profile", "summary", "topic"];
const PASSAGES_HEAD: &str = "Trevor could not run its model for this question, so this is not an answer: these are the passages \
that match the question best.";
const PASSAGES_NONE: &str = "Trevor could not run its model for this question, and no passage released at or before this \
reading position matches it.";

impl Service {
    /// Load everything once: the indexes of the main corpus (P3b by default; BM25 only with `--retrieval bm25`), the
    /// models `need` asks for (a llama-server is connected or started under the model lock and stopped when the service
    /// ends), the Cloudflare model list when configured, and the release order.
    async fn load(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono, need: Need) -> Result<Self> {
        let dense = a.retrieval == RetrievalKind::Hybrid;
        let main = load_runtime(&a.runtime, dense, a.rerank_add > 0, None)?;
        let mut parts = vec![(a.runtime.corpus.display().to_string(), main.store.chunks_sha.clone())];
        for dir in std::iter::once(p4_dir(&a)).chain([ART_DIR].into_iter().filter(|_| a.art)) {
            if let Some(sha) = recorded_sha(dir).filter(|_| std::path::Path::new(dir) != a.runtime.corpus) {
                parts.push((dir.to_owned(), sha));
            }
        }
        let corpus_version = corpus_version(&parts);
        let rts = Runtimes { main, p4: None, art: None, p4_dir: p4_dir(&a), switch: corpus_default && router_source(&a),
                             args: a.runtime.clone(), rerank: a.rerank_add > 0, horizon: None, dense };
        let (llm, server) = match need {
            Need::Model => {
                let (l, s) = connect_or_spawn(&a).await?;
                (Some(l), s)
            }
            Need::Local => match connect_local(&a).await {
                Ok((l, s)) => (Some(l), s),
                Err(e) => {
                    eprintln!("worker: no local model ({e:#}); the cpu rung is off until {} answers", a.server);
                    (None, None)
                }
            },
            Need::Nothing => (None, None),
        };
        let cf = cloudflare(&a)?;
        let order = Arc::new(ReleaseOrder::load(std::path::Path::new("."))?);
        eprintln!("horizon: {} stories on the gatePos scale, {} unplaced sources dated, {} eras", order.gate().story_order.len(),
                  order.dated_sources(), order.gate().eras.len());
        Ok(Self { argv: std::env::args().collect(), corpus_default, deep0: tables.tools.topics_deep, opts0: tables.tools.opts,
                  main_corpus: a.runtime.corpus.clone(), tables, rts, llm, cf, names: load_names(), chrono, order, corpus_version, a,
                  server, guard_stopped: false })
    }

    /// The job's arguments: the service's own, or (with job flags or another profile) the command line again plus the
    /// job's flags (refusing the ones that change what is loaded), with the job's profile applied.
    fn args_for(&self, flags: &[String], profile: Option<Profile>) -> Result<Args, JobError> {
        if flags.is_empty() && profile.is_none_or(|p| p == self.a.profile) {
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
        a.retrieval = self.a.retrieval;
        let p = profile.unwrap_or(a.profile);
        apply_profile(&mut a, p);
        Ok(a)
    }

    /// The job's horizon: its gatePos, its story or group id, or the lower of the two.
    fn horizon_for(&self, req: &AskRequest) -> Result<Option<Arc<Horizon>>, JobError> {
        let by_id = req.horizon.as_deref().map(|h| self.order.horizon(h)).transpose().map_err(bad)?;
        let by_pos = req.horizon_pos.map(|p| self.order.horizon_at(p)).transpose().map_err(bad)?;
        Ok(match (by_id, by_pos) {
            (Some(a), Some(b)) => Some(if b.pos < a.pos { b } else { a }),
            (a, b) => a.or(b),
        }.map(Arc::new))
    }

    fn profile_of(req: &AskRequest) -> Result<Option<Profile>, JobError> {
        req.profile.as_deref().map(|p| Profile::parse(p).ok_or_else(|| bad(format!("profile {p:?} is not one of {}", Profile::NAMES.join(", "))))).transpose()
    }

    /// Answer one job with `llm` (the service's own model when None).
    pub(crate) async fn ask(&mut self, req: AskRequest, llm: Option<Llm>) -> Result<Job, JobError> {
        let t0 = Instant::now();
        let server = req.server.as_deref().unwrap_or(SERVER);
        if server != SERVER {
            return Err(bad(format!("server {server:?} has no corpus (only \"{SERVER}\")")));
        }
        let q = req.question.trim().to_owned();
        if q.is_empty() {
            return Err(bad("empty question"));
        }
        let a = self.args_for(&req.flags, Self::profile_of(&req)?)?;
        let horizon = self.horizon_for(&req)?;
        let llm = llm.or_else(|| self.llm.clone()).ok_or((503, "no model is connected".to_owned()))?;
        let before = llm.usage();
        self.tables.tools.opts = tool_options(&a);
        let profile = a.profile;
        let res = self.answer(&q, &req, a, horizon, &llm, t0).await;
        self.tables.tools.opts = self.opts0;
        let (result, generated) = res?;
        Ok(Job { result, generated, usage: llm.usage() - before, profile, source: "live" })
    }

    async fn answer(&mut self, q: &str, req: &AskRequest, mut a: Args, horizon: Option<Arc<Horizon>>, llm: &Llm, t0: Instant)
                    -> Result<(AskResult, BTreeMap<String, String>), JobError> {
        if horizon.is_some() {
            // Built from every story: identity links, the chronology of the subject, the evidence composition.
            a.identity_notes = false;
            a.identity_labels = false;
            a.identity_expand = false;
            a.compose_evidence = false;
            a.no_chrono_subject = true;
        }
        let horizon_id = horizon.as_ref().map(|h| h.id.clone());
        self.rts.set_horizon(horizon);
        self.tables.topic = None;
        self.tables.route = None;
        self.tables.tools.topics_deep = self.deep0;
        let mut out = AskResult { answer: String::new(), citations: Vec::new(), passages: Vec::new(), route: None, table: false,
                                  flags: req.flags.clone(), form: None, horizon: horizon_id, server: SERVER.to_owned(),
                                  corpus_version: self.corpus_version.clone(), invalid_citations: 0, prompt_tokens: 0,
                                  timings: Timings { route_ms: 0.0, answer_ms: 0.0, total_ms: 0.0 } };
        // As the one-shot `ask "question"` does in `main`: P4 for a source question when the router names no source,
        // then route, then P4 when the route's source asks for it.
        let mut use_p4 = self.corpus_default && !router_source(&a) && source_kind(q).is_some();
        if !a.no_route {
            let router_llm = matches!(a.router, RouterKind::Model | RouterKind::Hybrid).then_some(llm);
            self.tables.tools.topics_deep = deep_for(&a, &self.tables.tools, q);
            let r = route_checked(q, &a, &mut self.tables, router_llm).await.map_err(failed)?;
            out.timings.route_ms = r.ms;
            let route = match r.text {
                Some(text) if !self.rts.horizon_set() => {
                    out.answer = text;
                    out.route = Some(r.route);
                    out.table = true;
                    out.timings.total_ms = t0.elapsed().as_secs_f64() * 1000.0;
                    return Ok((out, BTreeMap::new()));
                }
                // Under a horizon a table answer is not served; the passages answer instead.
                Some(_) => Route::retrieve(),
                None => {
                    self.tables.topic = if self.rts.horizon_set() { None } else { r.topic };
                    r.route
                }
            };
            if self.corpus_default && retrieval_plan(q, &a, Some(&route)).0 == Some(true) {
                use_p4 = true;
            }
            out.route = Some(route.clone());
            self.tables.route = Some(route);
        }
        let mut once = a.clone();
        once.routed = true;
        let swapped = if use_p4 { self.swap_p4().map_err(failed)? } else { false };
        let res = answer_one(&mut self.rts, llm, &self.names, &self.chrono, q, &once, &mut self.tables).await;
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
        Ok((out, ans.generated))
    }

    /// The `passages` rung (contract section 2): no model, the top `--passages-n` source passages within the horizon.
    pub(crate) fn passages(&mut self, req: &AskRequest) -> Result<Job, JobError> {
        let t0 = Instant::now();
        let q = req.question.trim().to_owned();
        if q.is_empty() {
            return Err(bad("empty question"));
        }
        let profile = Self::profile_of(req)?.unwrap_or(self.a.profile);
        let horizon = self.horizon_for(req)?;
        let horizon_id = horizon.as_ref().map(|h| h.id.clone());
        self.rts.set_horizon(horizon);
        let n = self.a.passages_n.max(1);
        let rt = self.rts.pick(None).map_err(failed)?;
        let hits = rt.retrieve(&q, &RetrievalConfig { mode: Mode::Hybrid, k: n * 4, ..RetrievalConfig::default() }).map_err(failed)?;
        let rows: Vec<usize> = hits.iter().map(|h| h.row).filter(|&r| !GENERATED_KINDS.contains(&rt.store.chunks[r].group_id.as_str()))
            .take(n).collect();
        let labelled: Vec<(String, String)> = rows.iter().map(|&r| {
            let c = &rt.store.chunks[r];
            (c.chunk_id.clone(), passage_label(&self.names, c))
        }).collect();
        let answer = if labelled.is_empty() { PASSAGES_NONE.to_owned() } else {
            let lines: Vec<String> = labelled.iter().enumerate().map(|(i, (_, l))| format!("[{}] {l}", i + 1)).collect();
            format!("{PASSAGES_HEAD}\n\n{}", lines.join("\n"))
        };
        let ids: Vec<String> = labelled.into_iter().map(|(id, _)| id).collect();
        let ms = t0.elapsed().as_secs_f64() * 1000.0;
        let result = AskResult { answer, citations: ids.iter().map(|id| self.citation(id)).collect(), passages: ids, route: None,
                                 table: false, flags: req.flags.clone(), form: None, horizon: horizon_id, server: SERVER.to_owned(),
                                 corpus_version: self.corpus_version.clone(), invalid_citations: 0, prompt_tokens: 0,
                                 timings: Timings { route_ms: 0.0, answer_ms: ms, total_ms: ms } };
        Ok(Job { result, generated: BTreeMap::new(), usage: UsageSnapshot::default(), profile, source: "passages" })
    }

    /// Make P4 the main corpus for this job, as the one-shot `ask` loads it as its only corpus: loaded on first use
    /// (sharing the main corpus's query embedder), swapped with the main corpus, swapped back after. False when the main
    /// corpus already is P4.
    fn swap_p4(&mut self) -> Result<bool> {
        let dir = PathBuf::from(self.rts.p4_dir);
        if self.rts.args.corpus == dir {
            return Ok(false);
        }
        if self.rts.p4.is_none() {
            let mut ra = self.rts.args.clone();
            ra.corpus.clone_from(&dir);
            self.rts.p4 = Some(load_runtime(&ra, self.rts.dense, self.rts.rerank, self.rts.main.shared_embedder())?);
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

    fn runtimes(&self) -> impl Iterator<Item = &Runtime> {
        std::iter::once(&self.rts.main).chain(self.rts.p4.as_ref()).chain(self.rts.art.as_ref())
    }

    /// A cited passage's story and line span, from whichever loaded corpus holds it.
    fn citation(&self, id: &str) -> Citation {
        for rt in self.runtimes() {
            if let Some(c) = rt.store.row(id).map(|r| &rt.store.chunks[r]) {
                return Citation { story_id: Some(c.story_id.clone()), chunk_id: id.to_owned(), line_start: Some(c.line_start),
                                  line_end: Some(c.line_end) };
            }
        }
        Citation { story_id: None, chunk_id: id.to_owned(), line_start: None, line_end: None }
    }

    /// A passage's text: the chunk's, a generated passage's, or a topic summary's.
    fn passage_text(&self, id: &str, generated: &BTreeMap<String, String>) -> String {
        for rt in self.runtimes() {
            if let Some(r) = rt.store.row(id) {
                return rt.store.chunks[r].text.clone();
            }
        }
        if let Some(t) = generated.get(id) {
            return t.clone();
        }
        id.strip_prefix("topic:").and_then(|t| self.tables.tools.topic(t).ok()).map(|t| t.text).unwrap_or_default()
    }

    /// The cited passage's text hash for the cache carry-over (contract v1.1 section 3b): the chunk's own `contentSha`
    /// (sha256 of its text, first 16 hex) when a loaded corpus holds it, else the same hash of the passage text (a
    /// generated passage: topic, dossier, game-data note).
    fn passage_sha(&self, id: &str, text: &str) -> String {
        self.runtimes().find_map(|rt| rt.store.row(id).map(|r| rt.store.chunks[r].content_sha.clone())).filter(|s| !s.is_empty())
            .unwrap_or_else(|| trevor::util::sha16(text))
    }

    fn health(&self) -> serde_json::Value {
        serde_json::json!({"ok": true, "server": SERVER, "corpusVersion": self.corpus_version,
                           "model": self.llm.as_ref().map(|l| l.model.clone())})
    }

    /// The contract's `result` of an answered job.
    fn worker_result(&self, job: &Job, rung: &str) -> serde_json::Value {
        let r = &job.result;
        let citations: Vec<serde_json::Value> = r.citations.iter().map(|c| {
            let n = r.passages.iter().position(|p| *p == c.chunk_id).map_or(0, |i| i + 1);
            let text = self.passage_text(&c.chunk_id, &job.generated);
            serde_json::json!({"n": n, "storyId": c.story_id, "chunkId": c.chunk_id, "lineStart": c.line_start,
                               "lineEnd": c.line_end, "sha": self.passage_sha(&c.chunk_id, &text), "text": trim_at_line(&text, CITE_CHARS)})
        }).collect();
        serde_json::json!({"answer": r.answer, "rung": rung, "profile": job.profile.name(), "source": job.source,
                           "citations": citations, "route": r.route, "form": r.form, "corpusVersion": r.corpus_version,
                           "tokensIn": job.usage.tokens_in, "tokensOut": job.usage.tokens_out, "timings": r.timings})
    }
}

/// The longest citation text a result carries (contract section 3).
const CITE_CHARS: usize = 600;
/// The backend's result limit.
const RESULT_MAX: usize = 64 * 1024;

/// `text` cut to at most `max` characters at a line break, or at a word break when the last line break inside `max`
/// would keep under half of it (`TREVOR_CITE_WORD_CUT=0`: the line break wherever it is, else `max` characters).
pub(crate) fn trim_at_line(text: &str, max: usize) -> String {
    trim_text(text, max, std::env::var("TREVOR_CITE_WORD_CUT").as_deref() != Ok("0"))
}

/// [`trim_at_line`] with the word-break rule chosen. Without it, an operator file (a title line, then one paragraph of
/// 1,000 characters) kept only its 25-character title: on P4x, 785 of the 19,604 chunks over 600 characters kept
/// under 100 of them and 2,021 under 300 (2026-10-16).
pub(crate) fn trim_text(text: &str, max: usize, word_cut: bool) -> String {
    if text.chars().count() <= max {
        return text.to_owned();
    }
    let end = text.char_indices().nth(max).map_or(text.len(), |(i, _)| i);
    let head = &text[..end];
    let floor = if word_cut { max / 2 } else { 0 };
    let keeps = |i: usize| i > 0 && (!word_cut || head[..i].trim_end().chars().count() >= floor);
    if let Some(i) = head.rfind('\n').filter(|&i| keeps(i)) {
        return head[..i].trim_end().to_owned();
    }
    if word_cut && let Some(i) = head.rfind(char::is_whitespace).filter(|&i| keeps(i)) {
        return head[..i].trim_end().to_owned();
    }
    head.to_owned()
}

/// A result body under the backend's 64 KB limit: citation texts are cut to 200 characters, then dropped, when needed.
pub(crate) fn fit_result(mut body: serde_json::Value) -> serde_json::Value {
    for cut in [Some(200), None] {
        if serde_json::to_vec(&body).map_or(0, |b| b.len()) <= RESULT_MAX {
            break;
        }
        if let Some(cs) = body["result"]["citations"].as_array_mut() {
            for c in cs {
                let t = c["text"].as_str().unwrap_or_default().to_owned();
                c["text"] = serde_json::json!(cut.map_or(String::new(), |n| trim_at_line(&t, n)));
            }
        }
    }
    body
}

/// `--serve` or `--worker`: load once, then serve until SIGINT or SIGTERM (the llama-server this process started, if
/// any, stops with it).
pub(crate) async fn run(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<()> {
    let worker = a.worker.clone();
    let port = a.port;
    let need = if worker.is_some() { Need::Local } else { Need::Model };
    let mut svc = Service::load(a, corpus_default, tables, chrono, need).await?;
    match worker {
        Some(base) => pull(&mut svc, &base).await,
        None => listen(&mut svc, port).await,
    }
}

async fn listen(svc: &mut Service, port: u16) -> Result<()> {
    let listener = TcpListener::bind(("127.0.0.1", port)).await.with_context(|| format!("binding 127.0.0.1:{port}"))?;
    eprintln!("serve: http://127.0.0.1:{port} ({}, model {})", svc.corpus_version, svc.llm.as_ref().map_or("none", |l| l.model.as_str()));
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
            Ok(req) => match svc.ask(req, None).await {
                Ok(j) => (200, serde_json::to_value(j.result)?),
                Err((code, e)) => (code, serde_json::json!({"error": e})),
            },
        },
        _ => (404, serde_json::json!({"error": "not found"})),
    };
    let body = serde_json::to_vec(&json)?;
    let reason = match code { 200 => "OK", 400 => "Bad Request", 404 => "Not Found", 503 => "Service Unavailable", _ => "Internal Server Error" };
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

/// A worker job (contract section 3).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkerJob {
    pub(crate) job_id: String,
    pub(crate) question: String,
    #[serde(default)]
    pub(crate) server: Option<String>,
    #[serde(default)]
    pub(crate) horizon_pos: Option<u32>,
    #[serde(default)]
    pub(crate) rung: Option<String>,
    #[serde(default)]
    pub(crate) profile: Option<String>,
    #[serde(default)]
    pub(crate) flags: Vec<String>,
}

/// The worker's side of the backend API.
#[derive(Clone)]
pub(crate) struct Api {
    pub(crate) http: reqwest::Client,
    pub(crate) base: String,
    pub(crate) key: String,
    pub(crate) worker_id: String,
    /// The waits before each retry of a result POST (1, 4, 16 s).
    pub(crate) backoff: Vec<Duration>,
}

/// One POST's outcome.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Posted {
    Ok,
    /// A 4xx other than 408 and 429: sending it again would not help.
    Refused(u16, String),
    Failed(String),
}

impl Api {
    pub(crate) fn url(&self, path: &str) -> String {
        format!("{}/api/trevor/worker/{path}", self.base)
    }

    pub(crate) async fn post(&self, path: &str, body: &serde_json::Value, timeout: Duration) -> Posted {
        let r = self.http.post(self.url(path)).header("x-trevor-worker-key", &self.key).timeout(timeout).json(body).send().await;
        match r {
            Ok(r) if r.status().is_success() => Posted::Ok,
            Ok(r) if r.status().is_client_error() && !matches!(r.status().as_u16(), 408 | 429) => {
                let s = r.status().as_u16();
                Posted::Refused(s, r.text().await.unwrap_or_default())
            }
            Ok(r) => Posted::Failed(format!("HTTP {}", r.status())),
            Err(e) => Posted::Failed(e.to_string()),
        }
    }

    /// POST with the result retries; the last outcome.
    pub(crate) async fn post_retrying(&self, path: &str, body: &serde_json::Value) -> Posted {
        let mut out = self.post(path, body, Duration::from_secs(30)).await;
        for wait in &self.backoff {
            if !matches!(out, Posted::Failed(_)) {
                break;
            }
            tokio::time::sleep(*wait).await;
            out = self.post(path, body, Duration::from_secs(30)).await;
        }
        out
    }
}

/// The worker id: `TREVOR_WORKER_ID`, else "trevor-<host name>" (stable across restarts, so a spooled result names the
/// worker that claimed its job).
fn worker_id() -> String {
    if let Ok(id) = std::env::var("TREVOR_WORKER_ID") {
        if !id.is_empty() {
            return id;
        }
    }
    let host = std::process::Command::new("hostname").output().ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned()).filter(|h| !h.is_empty()).unwrap_or_else(|| "local".into());
    format!("trevor-{host}")
}

fn spool_dir() -> PathBuf {
    PathBuf::from(std::env::var("TREVOR_SPOOL_DIR").ok().filter(|d| !d.is_empty()).unwrap_or_else(|| "artifacts/spool".into()))
}

/// Post a result; when every retry fails, write it to the spool directory.
pub(crate) async fn deliver(api: &Api, spool: &Path, body: &serde_json::Value) -> Posted {
    let out = api.post_retrying("result", body).await;
    match &out {
        Posted::Ok => {}
        Posted::Refused(s, t) => eprintln!("worker: the backend refused the result of {} (HTTP {s}): {t}", body["jobId"]),
        Posted::Failed(e) => {
            let id: String = body["jobId"].as_str().unwrap_or("job").chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
            let path = spool.join(format!("{id}.json"));
            let written = std::fs::create_dir_all(spool).and_then(|()| std::fs::write(&path, serde_json::to_vec(body).unwrap_or_default()));
            match written {
                Ok(()) => eprintln!("worker: posting the result of {id} failed ({e}); spooled to {}", path.display()),
                Err(w) => eprintln!("worker: posting the result of {id} failed ({e}) and spooling it failed too ({w})"),
            }
        }
    }
    out
}

/// Send every spooled result once; a sent or refused one is deleted. Returns (sent, kept).
pub(crate) async fn resend_spool(api: &Api, spool: &Path) -> (usize, usize) {
    let Ok(dir) = std::fs::read_dir(spool) else { return (0, 0) };
    let mut files: Vec<PathBuf> = dir.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.extension().is_some_and(|x| x == "json")).collect();
    files.sort();
    let (mut sent, mut kept) = (0, 0);
    for f in files {
        let Some(body) = std::fs::read(&f).ok().and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).ok()) else { continue };
        match api.post("result", &body, Duration::from_secs(30)).await {
            Posted::Ok | Posted::Refused(..) => {
                let _ = std::fs::remove_file(&f);
                sent += 1;
            }
            Posted::Failed(_) => kept += 1,
        }
    }
    (sent, kept)
}

/// What the stream of one job has sent: later calls of the job are sent only where they continue it.
#[derive(Default)]
pub(crate) struct DeltaState {
    sent: String,
    cur: String,
    stopped: bool,
}

impl DeltaState {
    /// The new text an event adds to what was sent (empty when none, or once a call has diverged).
    pub(crate) fn feed(&mut self, ev: StreamEvent) -> String {
        match ev {
            StreamEvent::Start => self.cur.clear(),
            StreamEvent::Piece(p) => self.cur.push_str(&p),
        }
        if self.stopped {
            return String::new();
        }
        if self.cur.len() > self.sent.len() {
            if self.cur.starts_with(&self.sent) {
                let add = self.cur[self.sent.len()..].to_owned();
                self.sent.push_str(&add);
                return add;
            }
            self.stopped = true;
        } else if !self.sent.starts_with(&self.cur) {
            self.stopped = true;
        }
        String::new()
    }
}

/// Post a job's deltas as they arrive (the events waiting at each send go in one delta). Ends when the sender is
/// dropped; returns the deltas sent.
pub(crate) async fn forward_deltas(api: Api, job_id: String, mut rx: UnboundedReceiver<StreamEvent>) -> u64 {
    let mut state = DeltaState::default();
    let mut seq = 0u64;
    let mut gone = false;
    while let Some(ev) = rx.recv().await {
        let mut text = state.feed(ev);
        while let Ok(ev) = rx.try_recv() {
            text.push_str(&state.feed(ev));
        }
        if text.is_empty() || gone {
            continue;
        }
        let body = serde_json::json!({"workerId": api.worker_id, "seq": seq, "text": text});
        match api.post(&format!("jobs/{job_id}/delta"), &body, Duration::from_secs(10)).await {
            Posted::Ok => seq += 1,
            Posted::Refused(404, _) => gone = true,
            other => {
                eprintln!("worker: delta {seq} of {job_id}: {other:?}");
                seq += 1;
            }
        }
    }
    seq
}

/// The canary result (`ask --canary`).
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanaryFile {
    /// Unix seconds.
    pub(crate) checked_at: u64,
    pub(crate) models: Vec<CanaryModel>,
    /// The first model id that passed, None when none did.
    pub(crate) passing: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CanaryModel {
    pub(crate) id: String,
    pub(crate) passed: bool,
    /// Answers naming the expected term with at least one valid citation and none invalid.
    pub(crate) hits: usize,
    pub(crate) errors: Vec<String>,
    pub(crate) json_calls: u64,
    pub(crate) json_failures: u64,
    pub(crate) tokens_in: u64,
    pub(crate) tokens_out: u64,
    pub(crate) secs: f64,
    pub(crate) answers: Vec<String>,
}

/// The 10 fixed canary questions and a term a right answer names (any of them).
const CANARY: [(&str, &[&str]); 10] = [
    ("Who is the leader of Reunion?", &["talulah"]),
    ("What is the name of the creature that protects Kal'tsit?", &["mon3tr"]),
    ("Which city's Guard Department does Ch'en work for?", &["lungmen"]),
    ("What disease do the Infected suffer from?", &["oripathy"]),
    ("What is the name of the Ursus city Reunion attacks at the start of the story?", &["chernobog"]),
    ("What material causes Oripathy?", &["originium"]),
    ("Who is the boss of Penguin Logistics?", &["emperor"]),
    ("What is the name of the company the Doctor works with?", &["rhodes island"]),
    ("Who was the leader of Babel?", &["theresa"]),
    ("What race is Amiya?", &["cautus", "chimera"]),
];
/// A model passes with no failed call and at least this many hits.
const CANARY_PASS: usize = 8;

fn canary_path() -> PathBuf {
    PathBuf::from(std::env::var("TREVOR_CANARY_FILE").ok().filter(|p| !p.is_empty()).unwrap_or_else(|| "artifacts/canary/cf.json".into()))
}

fn unix_now() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// The model id the last canary passed, when it is in `models` and the canary is at most `TREVOR_CANARY_MAX_AGE_DAYS`
/// (8) days old.
pub(crate) fn canary_passing(path: &Path, models: &[String], now: u64) -> Option<String> {
    let f: CanaryFile = trevor::util::read_json_opt(path)?;
    let max_days: u64 = std::env::var("TREVOR_CANARY_MAX_AGE_DAYS").ok().and_then(|d| d.parse().ok()).unwrap_or(8);
    let id = f.passing?;
    (now.saturating_sub(f.checked_at) <= max_days * 86_400 && models.contains(&id)).then_some(id)
}

/// `ask --canary`: the 10 questions on each configured Cloudflare model id in order (full profile, at the horizon of the
/// last dated story, so every answer comes from the model and not a table), until one passes.
pub(crate) async fn run_canary(a: Args, corpus_default: bool, tables: Tables, chrono: Chrono) -> Result<()> {
    let base = OpenAiConfig::cloudflare_from_env()
        .context("--canary needs TREVOR_CF_MODELS and TREVOR_CF_ACCOUNT_ID (or TREVOR_CF_BASE_URL)")?;
    let mut svc = Service::load(a, corpus_default, tables, chrono, Need::Nothing).await?;
    let gate = svc.order.gate();
    let last = u32::try_from(gate.story_order.len() - gate.never.len()).unwrap_or(u32::MAX);
    let mut out = CanaryFile { checked_at: unix_now(), ..CanaryFile::default() };
    for id in &base.models {
        let llm = openai_llm(&svc.a, OpenAiConfig { models: vec![id.clone()], ..base.clone() })?;
        let t = Instant::now();
        let mut m = CanaryModel { id: id.clone(), ..CanaryModel::default() };
        for (q, expect) in CANARY {
            let req = AskRequest { question: q.into(), horizon: None, horizon_pos: Some(last), server: None, flags: Vec::new(),
                                   profile: Some("full".into()) };
            match svc.ask(req, Some(llm.clone())).await {
                Ok(job) => {
                    let ans = job.result.answer.to_lowercase();
                    let cited = !job.result.citations.is_empty() && job.result.invalid_citations == 0;
                    if cited && expect.iter().any(|e| ans.contains(e)) {
                        m.hits += 1;
                    }
                    m.answers.push(trim_at_line(&job.result.answer, 300));
                }
                Err((_, e)) => {
                    m.errors.push(format!("{q}: {e}"));
                    m.answers.push(String::new());
                }
            }
        }
        let u = llm.usage();
        (m.json_calls, m.json_failures, m.tokens_in, m.tokens_out) = (u.json_calls, u.json_failures, u.tokens_in, u.tokens_out);
        m.secs = t.elapsed().as_secs_f64();
        m.passed = m.errors.is_empty() && m.hits >= CANARY_PASS;
        eprintln!("canary {id}: {} of {} hits, {} errors, {} of {} JSON calls failed, {:.0} s: {}", m.hits, CANARY.len(), m.errors.len(),
                  m.json_failures, m.json_calls, m.secs, if m.passed { "PASS" } else { "fail" });
        let passed = m.passed;
        out.models.push(m);
        if passed {
            out.passing = Some(id.clone());
            break;
        }
    }
    let path = canary_path();
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&out)?)?;
    std::fs::rename(&tmp, &path)?;
    println!("{}: passing {:?}", path.display(), out.passing);
    Ok(())
}

/// The heartbeat's view of the rungs.
#[derive(Clone)]
pub(crate) struct Probe {
    pub(crate) http: reqwest::Client,
    pub(crate) llama: String,
    pub(crate) cf_models: Option<Vec<String>>,
    pub(crate) canary: PathBuf,
}

impl Probe {
    /// `cpu` is false while the power guard is tripped (contract v1.1 section 2.1).
    async fn rungs(&self, guard: Option<&GuardView>) -> serde_json::Value {
        let cf = self.cf_models.as_ref().is_some_and(|m| canary_passing(&self.canary, m, unix_now()).is_some());
        let cpu = !guard.is_some_and(GuardView::tripped)
            && matches!(self.http.get(format!("{}/health", self.llama)).timeout(Duration::from_secs(5)).send().await,
                        Ok(r) if r.status().is_success());
        serde_json::json!({"cf": cf, "cpu": cpu})
    }
}

/// The heartbeat body; `guard` is None when the guard is off (`TREVOR_GUARD=0`), which leaves the v1 body unchanged.
pub(crate) fn heartbeat_body(worker_id: &str, rungs: serde_json::Value, corpus_version: &str, busy: Option<String>,
                             guard: Option<&GuardView>) -> serde_json::Value {
    let mut body = serde_json::json!({"workerId": worker_id, "rungs": rungs, "corpusVersion": corpus_version, "busyJobId": busy});
    if let Some(g) = guard {
        body["guard"] = serde_json::to_value(g.status()).unwrap_or_default();
    }
    body
}

/// A heartbeat every `every`, and one at once whenever the guard trips or recovers (the backend settles a claimed job's
/// rung by the last heartbeat, so a trip must not wait up to 30 s to be seen).
pub(crate) async fn heartbeat_loop(api: Api, probe: Probe, corpus_version: String, busy: Arc<Mutex<Option<String>>>, every: Duration,
                                   mut guard: Option<tokio::sync::watch::Receiver<GuardView>>) {
    loop {
        let view = guard.as_mut().map(|g| g.borrow_and_update().clone());
        let rungs = probe.rungs(view.as_ref()).await;
        let job = busy.lock().ok().and_then(|b| b.clone());
        let body = heartbeat_body(&api.worker_id, rungs, &corpus_version, job, view.as_ref());
        if std::env::var("TREVOR_HEARTBEAT_LOG").as_deref() == Ok("1") {
            eprintln!("worker: heartbeat {body}");
        }
        match api.post("heartbeat", &body, Duration::from_secs(15)).await {
            Posted::Ok => {}
            other => eprintln!("worker: heartbeat: {other:?}"),
        }
        match guard.as_mut() {
            Some(g) => {
                tokio::select! {
                    () = tokio::time::sleep(every) => {}
                    r = g.changed() => if r.is_err() { guard = None; },
                }
            }
            None => tokio::time::sleep(every).await,
        }
    }
}

/// The profile of a job that names none: `full` on the cf rung; on the others `TREVOR_CPU_PROFILE` (default `mid`,
/// contract v1.1 section 4; `slim` is the v1 default, the kill switch).
pub(crate) fn default_profile(rung: &str, cpu_profile: Option<&str>) -> String {
    if rung == "cf" {
        return "full".into();
    }
    cpu_profile.map(str::trim).filter(|p| Profile::parse(p).is_some()).unwrap_or("mid").to_owned()
}

impl Service {
    /// The local llama-server for the `cpu` rung: the one connected at load, or (when it was down then) a connection
    /// made now.
    async fn local(&mut self) -> Option<Llm> {
        if self.llm.is_none() {
            if let Ok(l) = Llm::connect(&self.a.server).await {
                self.llm = Some(if self.a.no_cache_prompt { l.without_prompt_cache() } else { l });
            }
        }
        self.llm.clone()
    }

    /// One worker job: the result body to post. With the power guard on, a `cpu` job never runs the local model while
    /// it is tripped, and a trip during the answer stops it: the job is answered with passages instead.
    async fn run_job(&mut self, api: &Api, job: WorkerJob, guard: Option<&tokio::sync::watch::Receiver<GuardView>>) -> serde_json::Value {
        let rung = job.rung.clone().unwrap_or_else(|| "cpu".into());
        let fail = |e: String| serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": false, "error": e});
        let profile = match job.profile.as_deref() {
            Some(p) => p.to_owned(),
            None => default_profile(&rung, std::env::var("TREVOR_CPU_PROFILE").ok().as_deref()),
        };
        let guarded = (rung == "cpu").then_some(guard).flatten();
        if guarded.is_some_and(|g| g.borrow().tripped()) {
            eprintln!("worker: the power guard is tripped; job {} gets passages instead of the cpu rung", job.job_id);
            return self.passages_body(api, &job, &AskRequest { question: job.question.clone(), horizon: None, horizon_pos: job.horizon_pos,
                                                                server: job.server.clone(), flags: job.flags.clone(), profile: Some(profile) });
        }
        let req = AskRequest { question: job.question.clone(), horizon: None, horizon_pos: job.horizon_pos, server: job.server.clone(),
                               flags: job.flags.clone(), profile: Some(profile) };
        let llm = match rung.as_str() {
            "cf" => self.cf.clone().inspect(|l| {
                if let Some(id) = canary_passing(&canary_path(), &l.model_ids(), unix_now()) {
                    l.prefer_model(&id);
                }
            }),
            "cpu" => self.local().await,
            "passages" => None,
            other => return fail(format!("unknown rung {other:?}")),
        };
        let answered = match llm {
            Some(llm) => {
                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
                let fwd = (!self.a.no_stream).then(|| {
                    llm.set_stream(Some(tx));
                    tokio::spawn(forward_deltas(api.clone(), job.job_id.clone(), rx))
                });
                let r = match guarded {
                    None => self.ask(req.clone(), Some(llm.clone())).await,
                    Some(g) => {
                        let mut g = g.clone();
                        let r = tokio::select! {
                            r = self.ask(req.clone(), Some(llm.clone())) => Some(r),
                            _ = g.wait_for(GuardView::tripped) => None,
                        };
                        match r {
                            Some(r) => r,
                            None => {
                                // The answer future was dropped mid-await: undo what `ask` restores after it.
                                self.reset_after_cancel();
                                Err((503, "the power guard tripped during the answer".to_owned()))
                            }
                        }
                    }
                };
                llm.set_stream(None);
                if let Some(f) = fwd {
                    let _ = tokio::time::timeout(Duration::from_secs(30), f).await;
                }
                match r {
                    Ok(j) => Ok((j, rung.clone())),
                    Err((400, e)) => return fail(e),
                    Err((_, e)) => {
                        eprintln!("worker: job {} failed on the {rung} rung ({e}); answering with passages", job.job_id);
                        self.passages(&req).map(|j| (j, "passages".to_owned()))
                    }
                }
            }
            None => {
                if rung != "passages" {
                    eprintln!("worker: no model for the {rung} rung; job {} gets passages", job.job_id);
                }
                self.passages(&req).map(|j| (j, "passages".to_owned()))
            }
        };
        match answered {
            Ok((j, used)) => fit_result(serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": true,
                                                           "result": self.worker_result(&j, &used)})),
            Err((_, e)) => fail(e),
        }
    }

    fn passages_body(&mut self, api: &Api, job: &WorkerJob, req: &AskRequest) -> serde_json::Value {
        match self.passages(req) {
            Ok(j) => fit_result(serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": true,
                                                   "result": self.worker_result(&j, "passages")})),
            Err((_, e)) => serde_json::json!({"jobId": job.job_id, "workerId": api.worker_id, "ok": false, "error": e}),
        }
    }

    /// After an answer future was dropped: the tool options and the main corpus as `ask` would have restored them.
    fn reset_after_cancel(&mut self) {
        self.tables.tools.opts = self.opts0;
        if self.rts.args.corpus != self.main_corpus {
            self.swap_back();
        }
    }

    /// Act on the guard between jobs: on a memory trip stop the llama-server this worker started; on recovery start it
    /// again (without waiting for the model lock: when another job holds it, the next turn tries again).
    async fn follow_guard(&mut self, view: &GuardView) {
        match &view.trip {
            Some(t) if t.memory && !self.guard_stopped => {
                if self.server.take().is_some() {
                    eprintln!("worker: memory trip: stopped the llama-server this worker started");
                    self.llm = None;
                } else {
                    eprintln!("worker: memory trip, but the llama-server at {} was not started by this worker (--share-server or \
                               already running); it is left running and no cpu job uses it until the guard recovers", self.a.server);
                }
                self.guard_stopped = true;
            }
            None if self.guard_stopped => {
                let mut a = self.a.clone();
                a.no_wait = true;
                match connect_local(&a).await {
                    Ok((l, s)) => {
                        eprintln!("worker: guard recovered; llama-server {}", if s.is_some() { "started again" } else { "connected" });
                        self.llm = Some(l);
                        self.server = s;
                        self.guard_stopped = false;
                    }
                    Err(e) => eprintln!("worker: guard recovered, but no llama-server yet ({e:#}); trying again after the next poll"),
                }
            }
            _ => {}
        }
    }
}

/// `--worker`: long-poll the backend for jobs, answer each, post the result; the heartbeat runs in its own task.
async fn pull(svc: &mut Service, base: &str) -> Result<()> {
    let key = std::env::var("TREVOR_WORKER_KEY").ok().filter(|k| !k.is_empty()).context("--worker needs TREVOR_WORKER_KEY")?;
    let http = reqwest::Client::builder().timeout(Duration::from_secs(90)).build()?;
    let api = Api { http: http.clone(), base: base.trim_end_matches('/').to_owned(), key, worker_id: worker_id(),
                    backoff: [1, 4, 16].map(Duration::from_secs).to_vec() };
    let spool = spool_dir();
    let (sent, kept) = resend_spool(&api, &spool).await;
    if sent + kept > 0 {
        eprintln!("worker: spool {}: {sent} results sent, {kept} kept", spool.display());
    }
    let busy = Arc::new(Mutex::new(None::<String>));
    let every = Duration::from_secs(std::env::var("TREVOR_HEARTBEAT_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(30));
    let probe = Probe { http, llama: svc.llm.as_ref().and_then(Llm::llama_base).unwrap_or(&svc.a.server).trim_end_matches('/').to_owned(),
                        cf_models: svc.cf.as_ref().map(Llm::model_ids), canary: canary_path() };
    let guard = GuardConfig::from_env().map(|cfg| {
        eprintln!("worker: power guard on ({cfg:?})");
        let (tx, rx) = tokio::sync::watch::channel(GuardView::default());
        tokio::spawn(guard::run(cfg, tx));
        rx
    });
    if guard.is_none() {
        eprintln!("worker: power guard off (TREVOR_GUARD=0)");
    }
    let hb = tokio::spawn(heartbeat_loop(api.clone(), probe, svc.corpus_version.clone(), Arc::clone(&busy), every, guard.clone()));
    eprintln!("worker {}: polling {} ({}; cf {}, local model {})", api.worker_id, api.base, svc.corpus_version,
              svc.cf.as_ref().map_or("off".to_owned(), |l| l.model_ids().join(",")), svc.llm.as_ref().map_or("none", |l| l.model.as_str()));
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    loop {
        if let Some(g) = &guard {
            let view = g.borrow().clone();
            svc.follow_guard(&view).await;
        }
        let next = serde_json::json!({"workerId": api.worker_id});
        let resp = tokio::select! {
            r = api.http.post(api.url("next")).header("x-trevor-worker-key", &api.key).json(&next).send() => r,
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
        if let Ok(mut b) = busy.lock() {
            *b = Some(job.job_id.clone());
        }
        let body = svc.run_job(&api, job, guard.as_ref()).await;
        deliver(&api, &spool, &body).await;
        if let Ok(mut b) = busy.lock() {
            *b = None;
        }
    }
    hb.abort();
    eprintln!("worker: stopping");
    Ok(())
}

/// `--horizon-check --batch FILE`: the gatePos horizon against the old story-id horizon. First every chunk of the main
/// corpus and of P4x at every placed story and group of the release order, then each question's 2,000 retrieved
/// candidates at the tie stories and 32 other horizons; every difference is printed.
pub(crate) fn horizon_check(a: &Args, batch: &Path) -> Result<()> {
    let order = Arc::new(ReleaseOrder::load(Path::new("."))?);
    let gate = order.gate();
    let horizons: Vec<String> = gate.story_order.iter().filter(|s| order.legacy_horizon(s).is_ok()).cloned()
        .chain(gate.eras.iter().map(|e| e.id.clone())).collect();
    let ties: Vec<String> = {
        let mut seen: HashMap<(String, i64), Vec<String>> = HashMap::new();
        for l in std::fs::read_to_string("artifacts/spoiler.jsonl")?.lines().filter(|l| !l.trim().is_empty()) {
            let v: serde_json::Value = serde_json::from_str(l)?;
            if let (Some(s), Some(g), Some(k)) = (v["storyId"].as_str(), v["groupId"].as_str(), v["storySort"].as_i64()) {
                seen.entry((g.to_owned(), k)).or_default().push(s.to_owned());
            }
        }
        let mut t: Vec<String> = seen.into_values().filter(|v| v.len() > 1).flatten().filter(|s| order.legacy_horizon(s).is_ok()).collect();
        t.sort();
        t
    };
    println!("tie stories in placed groups: {} {:?}", ties.len(), ties);
    let mut corpora = vec![a.runtime.corpus.clone()];
    if Path::new(crate::cli::GAME_TEXT_DIR).join("chunks.jsonl").exists() && a.runtime.corpus != Path::new(crate::cli::GAME_TEXT_DIR) {
        corpora.push(PathBuf::from(crate::cli::GAME_TEXT_DIR));
    }
    for dir in &corpora {
        let mut ra = a.runtime.clone();
        ra.corpus.clone_from(dir);
        let rt = Runtime::load(&ra, false, false, false)?;
        let (mut checks, mut diffs) = (0u64, BTreeMap::<(String, String, String), usize>::new());
        for h in &horizons {
            let (old, new) = (order.legacy_horizon(h)?, order.horizon(h)?);
            for c in &rt.store.chunks {
                checks += 1;
                if old.allows(&c.story_id, &c.group_id) != new.allows(&c.story_id, &c.group_id) {
                    *diffs.entry((h.clone(), c.story_id.clone(), c.group_id.clone())).or_default() += 1;
                }
            }
        }
        println!("{}: {} chunks x {} horizons = {checks} checks, {} (horizon, chunk story) pairs differ:", dir.display(),
                 rt.store.len(), horizons.len(), diffs.len());
        for ((h, s, g), n) in &diffs {
            let tie = ties.contains(h) && ties.contains(s);
            println!("  horizon {h} chunk story {s}: {n} chunks, old {} new {}{}", order.legacy_horizon(h)?.allows(s, g),
                     order.horizon(h)?.allows(s, g), if tie { " (tie)" } else { " (NOT A TIE)" });
        }
    }
    // The questions: hybrid candidates (BM25 with --retrieval bm25) at the tie stories and 32 spread horizons.
    let dense = a.retrieval == RetrievalKind::Hybrid;
    let mut rt = load_runtime(&a.runtime, dense, false, None)?;
    let step = (horizons.len() / 32).max(1);
    let sample: Vec<String> = ties.iter().cloned().chain(horizons.iter().step_by(step).cloned()).collect();
    let (mut kept_same, mut diff_rows) = (0usize, 0usize);
    for line in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let (qid, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
        let hits = rt.retrieve(q, &RetrievalConfig { mode: Mode::Hybrid, candidates: 2000, k: 2000, ..RetrievalConfig::default() })?;
        for h in &sample {
            let (old, new) = (order.legacy_horizon(h)?, order.horizon(h)?);
            let keep = |hz: &Horizon| -> Vec<usize> { hits.iter().map(|x| x.row).filter(|&r| rt.allows_row(hz, r)).collect() };
            let (ko, kn) = (keep(&old), keep(&new));
            if ko == kn {
                kept_same += 1;
                continue;
            }
            let only_old: Vec<&str> = ko.iter().filter(|r| !kn.contains(r)).map(|&r| rt.store.chunks[r].chunk_id.as_str()).collect();
            let only_new: Vec<&str> = kn.iter().filter(|r| !ko.contains(r)).map(|&r| rt.store.chunks[r].chunk_id.as_str()).collect();
            diff_rows += 1;
            println!("  {qid} at {h}: old keeps {} new keeps {}; old only {only_old:?}; new only {only_new:?}", ko.len(), kn.len());
        }
    }
    println!("questions: {kept_same} (question, horizon) pairs keep the same candidates, {diff_rows} differ");
    Ok(())
}

/// `--json-check --batch FILE`: the router and question-form calls of every question (no answer), with the usage
/// totals; on `--backend openai` that is the JSON validation rate. Prints `qid<TAB>route<TAB>form`.
pub(crate) async fn json_check(a: &Args, tables: &mut Tables, batch: &Path) -> Result<()> {
    let (llm, _guard) = connect_or_spawn(a).await?;
    let mut n = 0;
    for line in std::fs::read_to_string(batch)?.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line)?;
        let (qid, q) = (v["qid"].as_str().unwrap_or_default(), v["question"].as_str().unwrap_or_default());
        tables.tools.topics_deep = deep_for(a, &tables.tools, q);
        let r = route_checked(q, a, tables, Some(&llm)).await?;
        let form = match crate::form::question_form(&llm, q, a.form_v1, a.model_flags).await {
            Ok(f) => format!("{}{}", f.form, f.evidence.map_or(String::new(), |e| format!(": {e}"))),
            Err(e) if trevor::goldgen::llm::is_json_invalid(&e) => "INVALID".to_owned(),
            Err(e) => return Err(e),
        };
        println!("{qid}\t{}\t{form}", serde_json::to_string(&r.route)?);
        n += 1;
    }
    let u = llm.usage();
    eprintln!("json-check: {n} questions, {} calls, {} JSON calls, {} needed a second ask, {} failed twice; {} tokens in, {} out",
              u.calls, u.json_calls, u.json_retries, u.json_failures, u.tokens_in, u.tokens_out);
    Ok(())
}
