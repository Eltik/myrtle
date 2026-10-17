//! `ask --serve` and `ask --worker URL`: the ask runtime loaded once (tools, indexes, embedder, reranker, a
//! llama-server) and every question a library call, so a job costs the answer alone and not the ~15 s reload of a
//! one-shot `ask` (design/trevor-integration.md sections 3 and 8, step 1).
//!
//! `--serve` binds 127.0.0.1:`--port` (8090) and speaks a minimal HTTP/1.1 JSON API, one request per connection:
//! `GET /health` and `POST /v1/ask {question, horizon?, horizonPos?, server?, flags?, profile?}`. Jobs run one at a
//! time (one model slot); a connection that arrives during an answer waits in the listen backlog.
//!
//! `--worker URL` pulls jobs from a myrtle backend by design/trevor-contract-v1.md section 3, with the
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
//! its own server when none answers. Two workers on one `-np 2` server answer more jobs a minute than one worker on
//! `-np 1`, each job taking longer.
//!
//! Without a horizon a job takes exactly the path of the one-shot `ask "question"` (route first, P4 only when the route
//! or the source keywords ask for it, then `answer_one` with the route), so the answers match it byte for byte under
//! `--no-cache-prompt`. With a horizon (a gatePos, or a story or group id turned into one; src/search/horizon.rs)
//! retrieval returns only chunks released at or before it, and everything built from every story is left out: table
//! answers (the question is answered from the passages instead), topic summaries and entries, dossiers, the game-data
//! passage, canon evidence and the chronology notes.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use trevor::llm::{Llm, UsageSnapshot};
use trevor::search::horizon::ReleaseOrder;
use trevor::tools::{Route, ToolOptions};

use crate::answer::Tables;
use crate::chrono::Chrono;
use crate::cli::{Args, Profile};
use crate::retrieval::{Names, Runtimes};
use crate::llama_server::Spawned;

pub(crate) mod canary;
pub(crate) mod checks;
pub(crate) mod delivery;
pub(crate) mod http;
pub(crate) mod service;
pub(crate) mod worker;

pub(crate) use canary::run_canary;
pub(crate) use checks::{horizon_check, json_check};
pub(crate) use http::run;


/// The only server with a corpus today; CN is a second corpus version stream later (section 10).
const SERVER: &str = "en";

/// Request flags that would change what the service loaded; a job may not give them.
const LOAD_FLAGS: &[&str] = &["--corpus", "--lore", "--router", "--serve", "--worker", "--port", "--batch", "--out",
                              "--server", "--model", "--no-game-text", "--model-dir", "--query-onnx",
                              "--allow-query-model-mismatch", "--retrieval", "--backend", "--canary", "--gate-order",
                              "--horizon-check", "--json-check", "--no-stream"];

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
    let lines: String = parts.iter().map(|(dir, sha)| format!("{dir} {sha}\n")).collect();
    format!("{SERVER}-{}", trevor::util::sha16(&lines))
}

/// The `chunksSha` a corpus directory's vectors were built from (every derived index records it).
fn recorded_sha(dir: &str) -> Option<String> {
    let meta: serde_json::Value = trevor::util::read_json_opt(&PathBuf::from(dir).join("vectors.meta.json"))?;
    meta["chunksSha"].as_str().map(str::to_owned)
}

const PASSAGES_HEAD: &str = "Trevor could not run its model for this question, so this is not an answer: these are \
                             the passages that match the question best.";
const PASSAGES_NONE: &str = "Trevor could not run its model for this question, and no passage released at or before \
                             this reading position matches it.";
