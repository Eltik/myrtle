//! Trevor on the backend: the light plumbing between readers and a worker
//! (design: `trevor/design/trevor-integration.md`; binding interface:
//! `trevor/design/trevor-contract-v1.md`).
//!
//! The backend never runs a model. It normalizes a question into a cache key,
//! answers from the Postgres cache when it can, and otherwise queues a job with
//! a rung (Cloudflare, local CPU, or passages only) and a spoiler horizon that
//! a worker pulls (`worker/next`), answers, streams (`worker/jobs/{id}/delta`)
//! and posts back (`worker/result`). Workers connect out with the scoped
//! `TREVOR_WORKER_KEY`, never the global service key.
//!
//! - [`ask`]: the reader side (ask, job status, panels, feedback, status).
//! - [`worker`]: the worker side (claim, heartbeat, delta, result, publish,
//!   the stale sweep).
//! - [`stream`]: the per-job SSE fan-out.
//! - [`runtime`]: the in-process state (config, wake-up, broadcast registry).
//! - [`turnstile`]: the Cloudflare Turnstile check on ask.

pub mod ask;
pub mod runtime;
pub mod stream;
pub mod turnstile;
pub mod worker;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::error::ApiError;
pub use runtime::TrevorConfig;

/// Free questions a signed-in user may ask per clock hour (Ian, 2026-10-07).
pub const ASK_PER_HOUR: u64 = 30;
/// Open (queued or running) jobs one user may hold at once.
pub const MAX_OPEN_PER_USER: i64 = 3;
/// Queued jobs per server before ask answers "Trevor is busy" (503).
pub const MAX_QUEUED: i64 = 200;
/// The longest question accepted, in characters.
pub const MAX_QUESTION_CHARS: usize = 500;
/// A running job whose worker has not answered in this long goes back to the
/// queue once, and fails the second time (`MAX_ATTEMPTS` claims).
pub const RUNNING_STALE_SECS: i64 = 300;
pub const MAX_ATTEMPTS: i16 = 2;
/// How long `worker/next` holds an empty poll open, under the 30 s handler timeout.
pub const LONG_POLL_SECS: u64 = 20;
/// A worker counts as online while its last heartbeat is this recent (two
/// missed 30 s beats plus slack).
pub const ONLINE_SECS: i64 = 75;
/// The stale sweep's tick.
pub const SWEEP_EVERY_SECS: u64 = 60;
/// The SSE keep-alive comment interval (Cloudflare drops a proxied stream
/// that is silent for 100 s).
pub const KEEPALIVE_SECS: u64 = 15;
/// A cited passage's text is trimmed to this many characters, at a line break.
pub const CITATION_TEXT_CHARS: usize = 600;
/// The ETA reads the last this-many finished jobs of a rung ...
pub const ETA_WINDOW: i64 = 50;
/// ... and falls back to [`Rung::fallback_eta_secs`] below this many.
pub const ETA_MIN_SAMPLES: i64 = 5;
/// `trevor_answers.era_pos` of an internal job, whose horizon is unlimited.
/// No reader maps here: every reader's eraPos is a story's gatePos.
pub const NO_LIMIT_ERA: i32 = i32::MAX;

/// The servers Trevor may serve. EN has a corpus today; CN is accepted so a CN
/// version stream needs no code change, and simply has no worker or version
/// until one is published.
pub const SERVERS: &[&str] = &["en", "cn"];

/// `server` as sent, defaulted to EN and checked against [`SERVERS`].
pub fn parse_server(server: Option<&str>) -> Result<&'static str, ApiError> {
    let s = server.unwrap_or("en");
    SERVERS
        .iter()
        .copied()
        .find(|known| known.eq_ignore_ascii_case(s))
        .ok_or_else(|| ApiError::BadRequest(format!("server must be one of {SERVERS:?}")))
}

/// The question as the cache sees it: case-folded, whitespace collapsed,
/// typographic quotes and dashes made plain, trailing question marks and full
/// stops dropped. Two askers who type the same question differently share one
/// answer; anything that could change the meaning (word order, other
/// punctuation, digits) is kept.
pub fn normalize_question(question: &str) -> String {
    let mapped: String = question
        .chars()
        .map(|c| match c {
            '\u{2018}' | '\u{2019}' | '\u{2032}' => '\'',
            '\u{201C}' | '\u{201D}' => '"',
            '\u{2013}' | '\u{2014}' => '-',
            '\u{3000}' => ' ',
            '\u{FF1F}' => '?',
            '\u{3002}' => '.',
            c => c,
        })
        .collect();
    let collapsed = mapped
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();
    collapsed.trim_end_matches(['?', '.', '!', ' ']).to_owned()
}

/// The cache key of a normalized question: 32 hex chars of its SHA-256.
pub fn qkey(normalized: &str) -> String {
    hex::encode(&Sha256::digest(normalized.as_bytes())[..16])
}

/// Admit a question, or say why not.
pub fn validate_question(question: &str) -> Result<String, ApiError> {
    let normalized = normalize_question(question);
    if normalized.is_empty() {
        return Err(ApiError::BadRequest("question is empty".into()));
    }
    if question.chars().count() > MAX_QUESTION_CHARS {
        return Err(ApiError::BadRequest(format!(
            "question is longer than {MAX_QUESTION_CHARS} characters"
        )));
    }
    Ok(normalized)
}

// --- rungs (contract section 2) --------------------------------------------

/// Where an answer comes from. A job's rung is never `cache`; a response's is
/// `cache` when the answer was already stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorRung")]
#[schema(as = TrevorRung)]
pub enum Rung {
    Cache,
    Cf,
    Cpu,
    Passages,
}

impl Rung {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cache => "cache",
            Self::Cf => "cf",
            Self::Cpu => "cpu",
            Self::Passages => "passages",
        }
    }

    /// A job rung as stored; anything unknown reads as the model-free rung.
    pub fn parse_job(s: &str) -> Self {
        match s {
            "cf" => Self::Cf,
            "cpu" => Self::Cpu,
            _ => Self::Passages,
        }
    }

    /// The pipeline profile of a job on this rung: `cf` runs the full
    /// pipeline, `cpu` the slim one; `passages` ignores it (stored `full`).
    pub const fn profile(self) -> Profile {
        match self {
            Self::Cpu => Profile::Slim,
            _ => Profile::Full,
        }
    }

    /// The ETA while fewer than [`ETA_MIN_SAMPLES`] jobs finished on the rung.
    pub const fn fallback_eta_secs(self) -> i64 {
        match self {
            Self::Cf => 15,
            Self::Cpu => 120,
            Self::Cache | Self::Passages => 1,
        }
    }
}

/// The worker pipeline profile for a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorProfile")]
#[schema(as = TrevorProfile)]
pub enum Profile {
    Full,
    Slim,
}

impl Profile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slim => "slim",
        }
    }
}

/// The model rungs a worker reports live in its heartbeat.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(default)]
#[ts(export, rename = "TrevorRungFlags")]
#[schema(as = TrevorRungFlags)]
pub struct RungFlags {
    pub cf: bool,
    pub cpu: bool,
}

/// What the rung choice reads (contract section 2).
#[derive(Debug, Clone, Copy, Default)]
pub struct RungInputs {
    /// The rungs of the most recent heartbeat, online or not.
    pub rungs: RungFlags,
    /// Neurons spent today (UTC) by `cf` answers.
    pub neurons_today: i64,
    /// Jobs queued or running on the `cpu` rung.
    pub cpu_depth: i64,
}

/// Neurons `cf` answers may spend today before the summary reserve.
pub fn cf_serving_budget(cfg: &TrevorConfig) -> i64 {
    (cfg.cf_daily_neurons - cfg.cf_summary_reserve).max(0)
}

/// The rung a new job gets: `cf` while the heartbeat reports it and the
/// day's ledger is under the serving budget, else `cpu` while the heartbeat
/// reports it and its queue is short, else `passages`.
pub fn select_rung(inputs: &RungInputs, cfg: &TrevorConfig) -> Rung {
    if inputs.rungs.cf && inputs.neurons_today < cf_serving_budget(cfg) {
        Rung::Cf
    } else if inputs.rungs.cpu && inputs.cpu_depth < cfg.cpu_queue_max {
        Rung::Cpu
    } else {
        Rung::Passages
    }
}

/// The rung a claimed job runs on: the one it was queued with, stepped down
/// when the claiming worker no longer offers it or the Neuron budget ran out
/// while it waited. Never stepped up: a job queued as `passages` because the
/// CPU queue was full stays the quick answer its asker was promised.
pub fn settle_rung(
    assigned: Rung,
    worker: RungFlags,
    neurons_today: i64,
    cfg: &TrevorConfig,
) -> Rung {
    match assigned {
        Rung::Cf if worker.cf && neurons_today < cf_serving_budget(cfg) => Rung::Cf,
        Rung::Cf | Rung::Cpu if worker.cpu => Rung::Cpu,
        _ => Rung::Passages,
    }
}

/// Cloudflare Neurons of one `cf` answer:
/// `ceil(tokensIn * 9091 / 1e6 + tokensOut * 27273 / 1e6)`.
pub fn neurons(tokens_in: u64, tokens_out: u64) -> i64 {
    let micro = u128::from(tokens_in) * 9091 + u128::from(tokens_out) * 27273;
    i64::try_from(micro.div_ceil(1_000_000)).unwrap_or(i64::MAX)
}

/// The day's `cf` budget as a bucket, never a number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorCfBudget")]
#[schema(as = TrevorCfBudget)]
pub enum CfBudget {
    Ok,
    Low,
    Spent,
}

/// `low` under 20% of the serving budget left, `spent` at none.
pub fn cf_budget(neurons_today: i64, cfg: &TrevorConfig) -> CfBudget {
    let cap = cf_serving_budget(cfg);
    let left = cap - neurons_today;
    if left <= 0 {
        CfBudget::Spent
    } else if left * 5 < cap {
        CfBudget::Low
    } else {
        CfBudget::Ok
    }
}

// --- spoiler horizon (contract section 1) -----------------------------------

/// An era boundary from the active manifest: `position` is a gatePos.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorEra")]
#[schema(as = TrevorEra)]
pub struct Era {
    pub id: String,
    pub name: String,
    pub position: i32,
}

/// The horizon gatePos: the profile's, lowered (never raised) by the story
/// being read.
pub fn horizon_pos(profile_pos: i32, story_pos: Option<i32>) -> i32 {
    story_pos.map_or(profile_pos, |s| s.min(profile_pos)).max(0)
}

/// The era boundary at or before `pos`, if any.
pub fn era_at(eras: &[Era], pos: i32) -> Option<&Era> {
    eras.iter()
        .filter(|e| e.position > 0 && e.position <= pos)
        .max_by_key(|e| e.position)
}

/// "Spoiler-safe up to" in a response.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorSpoilerSafe")]
#[schema(as = TrevorSpoilerSafe)]
pub struct SpoilerSafe {
    pub era_id: String,
    pub name: String,
    pub position: i32,
}

impl From<&Era> for SpoilerSafe {
    fn from(e: &Era) -> Self {
        Self {
            era_id: e.id.clone(),
            name: e.name.clone(),
            position: e.position,
        }
    }
}

/// A resolved horizon: the gatePos a job runs under (and its answer is
/// cached by), and the label the reader sees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Horizon {
    pub era_pos: i32,
    pub spoiler_safe: Option<SpoilerSafe>,
}

/// Bucket `pos` to its era boundary. With `buckets` off
/// (`TREVOR_ERA_BUCKETS=0`) the job and the cache use `pos` exactly, the
/// pre-era behaviour; the label still names the era at or before it.
pub fn bucket_horizon(eras: &[Era], pos: i32, buckets: bool) -> Horizon {
    let era = era_at(eras, pos);
    Horizon {
        era_pos: if buckets {
            era.map_or(0, |e| e.position)
        } else {
            pos
        },
        spoiler_safe: era.map(SpoilerSafe::from),
    }
}

// --- results (contract section 3) --------------------------------------------

/// Trim a cited passage to at most `max` characters, at the last line break
/// inside the limit when there is one.
pub fn trim_citation_text(text: &str, max: usize) -> String {
    let Some((cut, _)) = text.char_indices().nth(max) else {
        return text.to_owned();
    };
    let head = &text[..cut];
    match head.rfind('\n') {
        Some(i) if i > 0 => head[..i].trim_end().to_owned(),
        _ => head.to_owned(),
    }
}

/// One cited passage.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    pub n: u32,
    pub story_id: String,
    pub chunk_id: String,
    pub line_start: u32,
    pub line_end: u32,
    pub text: String,
}

/// Whether a model wrote the answer or it is the top passages only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorAnswerSource")]
#[schema(as = TrevorAnswerSource)]
pub enum AnswerSource {
    Live,
    Passages,
}

impl AnswerSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Passages => "passages",
        }
    }

    pub fn parse(s: &str) -> Self {
        if s == "live" {
            Self::Live
        } else {
            Self::Passages
        }
    }
}

/// A worker's answer as stored and served. Unknown fields (the old full
/// `passages` list) are dropped on the way in.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerAnswer {
    pub answer: String,
    pub rung: Rung,
    pub profile: Profile,
    pub source: AnswerSource,
    /// Only the cited passages, each trimmed to 600 characters.
    pub citations: Vec<Citation>,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub route: Value,
    #[serde(default)]
    #[schema(value_type = Object)]
    pub form: Value,
    pub corpus_version: String,
    #[serde(default)]
    pub tokens_in: u64,
    #[serde(default)]
    pub tokens_out: u64,
    /// `{routeMs, answerMs, totalMs}`.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub timings: Value,
}

// --- wire types ----------------------------------------------------------

/// `POST /trevor/ask`.
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AskRequest {
    pub question: String,
    /// `en` (the default) or `cn`.
    #[serde(default)]
    #[ts(optional)]
    pub server: Option<String>,
    /// The story being read: lowers the horizon to it when it is before the
    /// furthest story the account has read. It never raises the horizon.
    #[serde(default)]
    #[ts(optional)]
    pub horizon_story: Option<String>,
    /// The Turnstile widget's token; required when the server checks Turnstile.
    #[serde(default)]
    #[ts(optional)]
    pub turnstile_token: Option<String>,
}

/// One answer as the reader sees it. `answer` is the worker's structured
/// result (answer text, cited passages, route, corpus version, timings).
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrevorAnswer {
    pub id: Uuid,
    pub version: String,
    pub server: String,
    /// The era boundary (a gatePos) the answer is spoiler-safe up to.
    pub era_pos: i32,
    /// The rung that made it (`cf`, `cpu` or `passages`).
    pub rung: Rung,
    pub source: AnswerSource,
    #[schema(value_type = Object)]
    pub answer: Value,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

impl JobStatus {
    pub fn parse(s: &str) -> Self {
        match s {
            "running" => Self::Running,
            "done" => Self::Done,
            "failed" => Self::Failed,
            _ => Self::Queued,
        }
    }

    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Done | Self::Failed)
    }
}

/// What `POST /trevor/ask` answers: a cached answer (`status: done`,
/// `rung: cache`, no job), or the job that will answer.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct AskResponse {
    /// Null for a cached answer: no job was queued.
    pub job_id: Option<Uuid>,
    pub status: JobStatus,
    pub rung: Rung,
    /// Null when the horizon is before the first era (nothing released).
    pub spoiler_safe_up_to: Option<SpoilerSafe>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub answer: Option<TrevorAnswer>,
    /// Seconds until the answer is expected; null while no worker is online
    /// and for a cached answer.
    #[ts(type = "number | null")]
    pub eta_secs: Option<i64>,
    /// `GET` this for the answer as it is written (SSE); null for a cached answer.
    pub stream_url: Option<String>,
}

/// `GET /trevor/jobs/{id}`, and the SSE `done` event.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JobView {
    pub job_id: Uuid,
    pub status: JobStatus,
    pub rung: Rung,
    /// 1-based place in the server's queue; 0 once a worker took it.
    #[ts(type = "number")]
    pub position: i64,
    /// Seconds until the answer is expected; null once finished and while no
    /// worker is online.
    #[ts(type = "number | null")]
    pub eta_secs: Option<i64>,
    /// False when no worker has sent a heartbeat in the last 75 s.
    pub worker_online: bool,
    /// Present once `status` is `done`.
    pub answer: Option<TrevorAnswer>,
    /// Present once `status` is `failed`.
    pub error: Option<String>,
}

/// Live jobs per model rung.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS, ToSchema)]
#[ts(export, rename = "TrevorQueueDepth")]
#[schema(as = TrevorQueueDepth)]
pub struct QueueDepth {
    #[ts(type = "number")]
    pub cf: i64,
    #[ts(type = "number")]
    pub cpu: i64,
}

/// `GET /trevor/status`.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrevorStatus {
    /// A worker sent a heartbeat in the last 75 s.
    pub online: bool,
    /// The model rungs online workers report (both false while offline).
    pub rungs: RungFlags,
    pub active_version: Option<String>,
    /// Jobs queued or running per model rung.
    pub queue_depth: QueueDepth,
    pub cf_budget: CfBudget,
}

/// `GET /trevor/panels/{storyId}`: the published panel JSON verbatim.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct StoryPanels {
    pub story_id: String,
    pub server: String,
    pub version: String,
    #[schema(value_type = Object)]
    pub panels: Value,
}

/// `POST /trevor/answers/{id}/feedback`.
#[derive(Debug, Clone, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FeedbackRequest {
    /// 1 up, -1 down, 0 withdraws the vote but keeps a note.
    pub vote: i16,
    /// "This is wrong" and why; at most 2,000 characters.
    #[serde(default)]
    pub note: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> TrevorConfig {
        TrevorConfig::default()
    }

    #[test]
    fn normalize_folds_case_space_quotes_and_trailing_marks() {
        assert_eq!(
            normalize_question("  Who is   Kal\u{2019}tsit?? "),
            "who is kal'tsit"
        );
        assert_eq!(normalize_question("Who is Kal'tsit"), "who is kal'tsit");
        assert_eq!(normalize_question("Why?\u{FF1F}"), "why");
        assert_eq!(
            normalize_question("\u{201C}Ch\u{2014}en\u{201D}!"),
            "\"ch-en\""
        );
    }

    #[test]
    fn normalize_keeps_what_changes_meaning() {
        assert_ne!(
            normalize_question("Did W kill Theresa?"),
            normalize_question("Did Theresa kill W?")
        );
        assert_ne!(
            normalize_question("chapter 10"),
            normalize_question("chapter 1")
        );
        assert_eq!(normalize_question("a, b"), "a, b");
    }

    #[test]
    fn qkey_is_stable_and_shared_by_equivalent_questions() {
        let a = qkey(&normalize_question("Who is Amiya?"));
        let b = qkey(&normalize_question("who is amiya"));
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        assert_ne!(a, qkey(&normalize_question("Who is Kal'tsit?")));
    }

    #[test]
    fn validate_refuses_empty_and_long() {
        assert!(validate_question(" ?? ").is_err());
        assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS + 1)).is_err());
        assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS)).is_ok());
    }

    #[test]
    fn server_defaults_to_en_and_refuses_unknown() {
        assert_eq!(parse_server(None).unwrap(), "en");
        assert_eq!(parse_server(Some("CN")).unwrap(), "cn");
        assert!(parse_server(Some("jp")).is_err());
    }

    #[test]
    fn rung_selection_follows_heartbeat_budget_and_cpu_depth() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let pick = |rungs, neurons_today, cpu_depth| {
            select_rung(
                &RungInputs {
                    rungs,
                    neurons_today,
                    cpu_depth,
                },
                &c,
            )
        };
        assert_eq!(pick(both, 0, 0), Rung::Cf);
        // 9000 - 1000 = 8000 serving Neurons: at 7999 spent cf still runs.
        assert_eq!(pick(both, 7999, 0), Rung::Cf);
        assert_eq!(pick(both, 8000, 0), Rung::Cpu);
        assert_eq!(pick(both, 8000, 5), Rung::Cpu);
        assert_eq!(pick(both, 8000, 6), Rung::Passages);
        assert_eq!(
            pick(
                RungFlags {
                    cf: false,
                    cpu: true
                },
                0,
                0
            ),
            Rung::Cpu
        );
        assert_eq!(pick(RungFlags::default(), 0, 0), Rung::Passages);
        // TREVOR_CF_DAILY_NEURONS=0 turns the cf rung off.
        let off = TrevorConfig {
            cf_daily_neurons: 0,
            ..cfg()
        };
        let inputs = RungInputs {
            rungs: both,
            neurons_today: 0,
            cpu_depth: 0,
        };
        assert_eq!(select_rung(&inputs, &off), Rung::Cpu);
        assert_eq!(cf_budget(0, &off), CfBudget::Spent);
    }

    #[test]
    fn a_claimed_job_steps_down_never_up() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let cpu_only = RungFlags {
            cf: false,
            cpu: true,
        };
        let cf_only = RungFlags {
            cf: true,
            cpu: false,
        };
        assert_eq!(settle_rung(Rung::Cf, both, 0, &c), Rung::Cf);
        assert_eq!(settle_rung(Rung::Cf, both, 8000, &c), Rung::Cpu);
        assert_eq!(settle_rung(Rung::Cf, cpu_only, 0, &c), Rung::Cpu);
        assert_eq!(
            settle_rung(Rung::Cf, RungFlags::default(), 0, &c),
            Rung::Passages
        );
        assert_eq!(settle_rung(Rung::Cpu, both, 0, &c), Rung::Cpu);
        assert_eq!(settle_rung(Rung::Cpu, cf_only, 0, &c), Rung::Passages);
        assert_eq!(settle_rung(Rung::Passages, both, 0, &c), Rung::Passages);
        assert_eq!(Rung::Cf.profile(), Profile::Full);
        assert_eq!(Rung::Cpu.profile(), Profile::Slim);
    }

    #[test]
    fn neurons_round_up_per_the_contract_formula() {
        assert_eq!(neurons(0, 0), 0);
        assert_eq!(neurons(1, 0), 1);
        // 2000 in, 250 out: 18.182 + 6.81825 = 25.00025 -> 26.
        assert_eq!(neurons(2000, 250), 26);
        // 9000 in, 400 out: 81.819 + 10.9092 = 92.7282 -> 93.
        assert_eq!(neurons(9000, 400), 93);
    }

    #[test]
    fn cf_budget_is_a_bucket() {
        let c = cfg();
        assert_eq!(cf_budget(0, &c), CfBudget::Ok);
        // 20% of 8000 is 1600: 1600 left is still ok, 1599 is low.
        assert_eq!(cf_budget(6400, &c), CfBudget::Ok);
        assert_eq!(cf_budget(6401, &c), CfBudget::Low);
        assert_eq!(cf_budget(8000, &c), CfBudget::Spent);
        assert_eq!(cf_budget(9500, &c), CfBudget::Spent);
    }

    fn eras() -> Vec<Era> {
        [("main_0", 1), ("main_1", 21), ("main_2", 40)]
            .into_iter()
            .map(|(id, position)| Era {
                id: id.into(),
                name: id.to_uppercase(),
                position,
            })
            .collect()
    }

    #[test]
    fn the_request_story_lowers_the_horizon_and_never_raises_it() {
        assert_eq!(horizon_pos(30, None), 30);
        assert_eq!(horizon_pos(30, Some(12)), 12);
        assert_eq!(horizon_pos(30, Some(500)), 30);
        assert_eq!(horizon_pos(0, Some(5)), 0);
    }

    #[test]
    fn the_horizon_buckets_to_the_era_at_or_before_it() {
        let e = eras();
        assert_eq!(bucket_horizon(&e, 0, true).era_pos, 0);
        assert_eq!(bucket_horizon(&e, 0, true).spoiler_safe, None);
        assert_eq!(bucket_horizon(&e, 1, true).era_pos, 1);
        assert_eq!(bucket_horizon(&e, 20, true).era_pos, 1);
        let at_21 = bucket_horizon(&e, 21, true);
        assert_eq!(at_21.era_pos, 21);
        assert_eq!(at_21.spoiler_safe.unwrap().era_id, "main_1");
        assert_eq!(bucket_horizon(&e, 39, true).era_pos, 21);
        assert_eq!(bucket_horizon(&e, 1000, true).era_pos, 40);
    }

    #[test]
    fn era_buckets_off_keys_the_exact_position() {
        let e = eras();
        let h = bucket_horizon(&e, 39, false);
        assert_eq!(h.era_pos, 39);
        assert_eq!(h.spoiler_safe.unwrap().position, 21);
    }

    #[test]
    fn citations_trim_at_a_line_break_inside_the_limit() {
        assert_eq!(trim_citation_text("short", 600), "short");
        let text = format!("{}\n{}", "a".repeat(500), "b".repeat(200));
        assert_eq!(trim_citation_text(&text, 600), "a".repeat(500));
        // No break inside the limit: a hard cut at the character limit.
        let long = "\u{00e9}".repeat(700);
        assert_eq!(trim_citation_text(&long, 600).chars().count(), 600);
        let exact = "x".repeat(600);
        assert_eq!(trim_citation_text(&exact, 600), exact);
    }
}
