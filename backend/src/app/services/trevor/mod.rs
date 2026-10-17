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

pub mod alerts;
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
/// Queued jobs per server before ask answers "Trevor is busy" (503); only
/// under `TREVOR_ADMISSION=0`. Admission control replaces it with
/// `TREVOR_GLOBAL_QUEUE_MAX` and a passages answer instead of the 503.
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

/// The question as the cache saw it before contract v1.1 (kept verbatim for
/// `TREVOR_NORMQ=0`): case-folded, whitespace collapsed, typographic quotes
/// and dashes made plain, trailing question marks and full stops dropped.
pub fn normalize_question_v1(question: &str) -> String {
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

/// The compatibility fold of one character: the NFKC mappings a typed
/// question meets (full-width ASCII, the ideographic and other Unicode
/// spaces, ligatures, super/subscript digits, the ellipsis). Since
/// 2026-10-17 [`normalize_question`] runs full NFKC (`unicode-normalization`)
/// before it, so this keeps only what NFKC leaves alone.
fn nfkc_fold(c: char, out: &mut String) {
    match c {
        // Full-width ASCII block -> ASCII.
        '\u{FF01}'..='\u{FF5E}' => {
            out.push(char::from_u32(c as u32 - 0xFF01 + 0x21).unwrap_or(c));
        }
        '\u{00A0}'
        | '\u{1680}'
        | '\u{2000}'..='\u{200A}'
        | '\u{202F}'
        | '\u{205F}'
        | '\u{3000}' => out.push(' '),
        '\u{200B}' | '\u{200C}' | '\u{200D}' | '\u{2060}' | '\u{FEFF}' | '\u{00AD}' => {}
        '\u{FB00}' => out.push_str("ff"),
        '\u{FB01}' => out.push_str("fi"),
        '\u{FB02}' => out.push_str("fl"),
        '\u{FB03}' => out.push_str("ffi"),
        '\u{FB04}' => out.push_str("ffl"),
        '\u{2026}' => out.push_str("..."),
        '\u{00B2}' => out.push('2'),
        '\u{00B3}' => out.push('3'),
        '\u{00B9}' => out.push('1'),
        '\u{2070}' => out.push('0'),
        '\u{2074}'..='\u{2079}' => {
            out.push(char::from_u32(c as u32 - 0x2074 + u32::from(b'4')).unwrap_or(c));
        }
        '\u{2080}'..='\u{2089}' => {
            out.push(char::from_u32(c as u32 - 0x2080 + u32::from(b'0')).unwrap_or(c));
        }
        '\u{2160}'..='\u{2169}' => {
            const ROMAN: [&str; 10] = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];
            out.push_str(ROMAN[(c as u32 - 0x2160) as usize]);
        }
        '\u{3002}' | '\u{FF61}' => out.push('.'),
        c => out.push(c),
    }
}

/// An apostrophe inside a word joins it ("Kal'tsit" = "Kaltsit"); every
/// other punctuation mark is a word break.
const fn is_apostrophe(c: char) -> bool {
    matches!(
        c,
        '\'' | '\u{2018}' | '\u{2019}' | '\u{02BC}' | '\u{2032}' | '`' | '\u{00B4}'
    )
}

/// normQ (contract v1.1 section 1a), the question as the cache, the twin
/// join and the open-job index see it: the NFKC compatibility fold
/// ([`nfkc_fold`]), lowercase, apostrophes dropped, every other punctuation
/// or symbol mark folded to a space, whitespace collapsed, which also drops
/// a trailing "?". Word order and digits are kept, so "Did W kill Theresa"
/// and "Did Theresa kill W" stay two questions. `TREVOR_NORMQ=0` restores
/// [`normalize_question_v1`].
pub fn normalize_question(question: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    // Full NFKC first (composes "e" + U+0301 into "é", folds compatibility
    // forms); `nfkc_fold` then applies the mappings NFKC leaves alone.
    let question: String = question.nfkc().collect();
    let mut folded = String::with_capacity(question.len());
    for c in question.chars() {
        nfkc_fold(c, &mut folded);
    }
    let mut out = String::with_capacity(folded.len());
    for c in folded.chars().flat_map(char::to_lowercase) {
        if is_apostrophe(c) {
            continue;
        }
        // Combining marks stay with their letter; anything else that is not
        // a letter or digit breaks the word.
        if c.is_alphanumeric() || is_combining(c) {
            out.push(c);
        } else {
            out.push(' ');
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// The combining diacritical marks (U+0300 to U+036F), which belong to the
/// letter before them rather than breaking the word.
const fn is_combining(c: char) -> bool {
    matches!(c, '\u{0300}'..='\u{036F}')
}

/// The normalized question under the configured rule (`TREVOR_NORMQ`).
pub fn normalize_with(question: &str, normq: bool) -> String {
    if normq {
        normalize_question(question)
    } else {
        normalize_question_v1(question)
    }
}

/// The cache key of a normalized question: 32 hex chars of its SHA-256.
pub fn qkey(normalized: &str) -> String {
    hex::encode(&Sha256::digest(normalized.as_bytes())[..16])
}

/// Admit a question, or say why not; `normq` picks the normalization
/// (`TREVOR_NORMQ`).
pub fn validate_question(question: &str, normq: bool) -> Result<String, ApiError> {
    let normalized = normalize_with(question, normq);
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

    /// The pipeline profile of a job on this rung under contract v1: `cf`
    /// runs the full pipeline, `cpu` the slim one; `passages` ignores it
    /// (stored `full`).
    pub const fn profile(self) -> Profile {
        match self {
            Self::Cpu => Profile::Slim,
            _ => Profile::Full,
        }
    }

    /// The profile under the configured CPU profile (contract v1.1 section
    /// 4: `cpu` runs `mid` by default; `TREVOR_CPU_PROFILE=slim` restores
    /// [`Rung::profile`]).
    pub const fn profile_with(self, cpu: Profile) -> Profile {
        match self {
            Self::Cpu => cpu,
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
    /// The E2B eval winner (contract v1.1 section 4): keyword router, k 8, a
    /// 3,600-token passage budget, 250 output tokens, one model call.
    Mid,
}

impl Profile {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Slim => "slim",
            Self::Mid => "mid",
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
    /// Jobs queued or running on the `cpu` rung (normal jobs only once the
    /// deferred queue exists).
    pub cpu_depth: i64,
}

/// What admission control reads besides [`RungInputs`] (contract v1.1
/// section 1b).
#[derive(Debug, Clone, Copy, Default)]
pub struct AdmissionInputs {
    pub rung: RungInputs,
    /// Normal jobs queued or running on the `cf` rung.
    pub cf_depth: i64,
    /// Online workers whose heartbeat offers `cpu`.
    pub cpu_workers: i64,
    /// Jobs open (queued or running, normal or deferred) on every server.
    pub open_total: i64,
}

/// Admission control's verdict for a new question.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Admission {
    /// A model rung (or, with no model rung offered at all, the passages
    /// rung) takes it now.
    Now(Rung),
    /// Every offered model rung is full: a quick passages job now, and the
    /// full answer as a deferred job on `deferred` when the global cap
    /// allows (`None` when it does not).
    Overflow { deferred: Option<Rung> },
}

/// The CPU rung's cap: `TREVOR_CPU_QUEUE_MAX` per online cpu worker. A
/// heartbeat that offers `cpu` while no cpu worker is online (a restart)
/// counts as one, so a single worker keeps the v1 cap exactly.
pub fn cpu_cap(cfg: &TrevorConfig, cpu_workers: i64) -> i64 {
    cfg.cpu_queue_max.saturating_mul(cpu_workers.max(1))
}

/// Admission control (contract v1.1 section 1b): `cf` while the heartbeat
/// offers it, the day's ledger is under budget and fewer than
/// `TREVOR_CF_QUEUE_MAX` normal jobs are on it (`0`: no cap); else `cpu`
/// while offered and under [`cpu_cap`]; else, when a model rung is offered
/// but full, an overflow (deferred on the rung that will serve it, unless
/// `TREVOR_GLOBAL_QUEUE_MAX` jobs are already open); else `passages`.
pub fn admit_rung(inputs: &AdmissionInputs, cfg: &TrevorConfig) -> Admission {
    let r = &inputs.rung;
    let cf_usable = r.rungs.cf && r.neurons_today < cf_serving_budget(cfg);
    let cf_room = cfg.cf_queue_max <= 0 || inputs.cf_depth < cfg.cf_queue_max;
    if cf_usable && cf_room {
        return Admission::Now(Rung::Cf);
    }
    if r.rungs.cpu && r.cpu_depth < cpu_cap(cfg, inputs.cpu_workers) {
        return Admission::Now(Rung::Cpu);
    }
    let full = if r.rungs.cpu {
        Some(Rung::Cpu)
    } else if cf_usable {
        Some(Rung::Cf)
    } else {
        None
    };
    match full {
        None => Admission::Now(Rung::Passages),
        Some(rung) => Admission::Overflow {
            deferred: (inputs.open_total < cfg.global_queue_max).then_some(rung),
        },
    }
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
    // TREVOR_MODE=passages (or off): no model runs, whatever was queued.
    if cfg.mode != Mode::On {
        return Rung::Passages;
    }
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

// --- master switch and guard (contract v1.1 sections 2.1, 2.3) --------------

/// `TREVOR_MODE`: `on` (default), `passages` (every ask answered from
/// passages at once, no model job queued), `off` (ask is a 503 "Trevor is
/// resting" unless the answer is cached; panels and cached answers serve).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorMode")]
#[schema(as = TrevorMode)]
pub enum Mode {
    On,
    Passages,
    Off,
}

impl Mode {
    /// The env value; anything unknown reads as `on`, with a warning.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "on" | "" => Some(Self::On),
            "passages" => Some(Self::Passages),
            "off" => Some(Self::Off),
            _ => None,
        }
    }
}

/// A worker's power guard state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "lowercase")]
#[ts(export, rename = "TrevorGuardState")]
#[schema(as = TrevorGuardState)]
pub enum GuardState {
    #[default]
    Ok,
    Tripped,
}

/// The heartbeat's `guard` (contract v1.1 section 2.1).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct Guard {
    pub state: GuardState,
    pub reason: Option<String>,
    /// Unix seconds the state began.
    pub since: Option<i64>,
}

/// Online workers per model rung.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, TS, ToSchema)]
#[ts(export, rename = "TrevorWorkerCounts")]
#[schema(as = TrevorWorkerCounts)]
pub struct WorkerCounts {
    #[ts(type = "number")]
    pub cf: i64,
    #[ts(type = "number")]
    pub cpu: i64,
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
    /// The cited chunk's text sha (contract v1.1 section 3b); answers whose
    /// every citation carries one can be carried to a new version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha: Option<String>,
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
    /// The full answer was deferred (contract v1.1 section 1b): this job
    /// gives the quick passages answer now, and the full one shows in "My
    /// questions" (`GET /trevor/me/jobs`) under `deferredJobId` when ready.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub deferred_job_id: Option<Uuid>,
    /// "Quick look" while the answer is made: the same question's cached
    /// answer from an earlier era (so it is spoiler-safe for this reader).
    /// Sent only with an overflowed or passages-only ask, and only when one
    /// exists.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub quick_look: Option<TrevorAnswer>,
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
    /// Online workers offering each model rung (contract v1.1 section 1d).
    pub workers: WorkerCounts,
    /// `tripped` while any online worker's power guard is tripped.
    pub guard: GuardState,
    /// The master switch, `TREVOR_MODE`.
    pub mode: Mode,
}

/// One of the reader's questions, for "My questions" (`GET /trevor/me/jobs`).
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorMyQuestion")]
#[schema(as = TrevorMyQuestion)]
pub struct MyQuestion {
    /// The job that makes (or made) the full answer: the deferred job of an
    /// overflowed ask, else the ask's job; null for a cache hit.
    pub job_id: Option<Uuid>,
    pub question: String,
    pub status: JobStatus,
    /// `cache` for a cache hit, else the job's rung.
    pub rung: Rung,
    /// Whether a model wrote the answer; null until there is one.
    pub source: Option<AnswerSource>,
    #[ts(type = "string")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[ts(type = "string | null")]
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Present once done.
    pub answer: Option<TrevorAnswer>,
}

/// One popular question with its answer (`GET /trevor/popular`).
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorPopularQuestion")]
#[schema(as = TrevorPopularQuestion)]
pub struct PopularQuestion {
    pub question: String,
    /// Distinct accounts that asked it (at least 2).
    #[ts(type = "number")]
    pub askers: i64,
    pub answer: TrevorAnswer,
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
    fn normalize_v1_folds_case_space_quotes_and_trailing_marks() {
        assert_eq!(
            normalize_question_v1("  Who is   Kal\u{2019}tsit?? "),
            "who is kal'tsit"
        );
        assert_eq!(normalize_question_v1("Who is Kal'tsit"), "who is kal'tsit");
        assert_eq!(normalize_question_v1("Why?\u{FF1F}"), "why");
        assert_eq!(
            normalize_question_v1("\u{201C}Ch\u{2014}en\u{201D}!"),
            "\"ch-en\""
        );
        assert_eq!(normalize_question_v1("a, b"), "a, b");
        // TREVOR_NORMQ=0 is exactly the v1 rule.
        assert_eq!(
            normalize_with("A, b?", false),
            normalize_question_v1("A, b?")
        );
    }

    #[test]
    fn normq_folds_nfkc_case_punctuation_space_and_the_trailing_mark() {
        let n = normalize_question;
        assert_eq!(n("  Who is   Kal\u{2019}tsit?? "), "who is kaltsit");
        assert_eq!(n("Who is Kal'tsit"), "who is kaltsit");
        assert_eq!(n("who is kaltsit"), "who is kaltsit");
        // Full-width letters, ideographic space and question mark (NFKC).
        assert_eq!(
            n("\u{FF37}ho\u{3000}is \u{FF21}miya\u{FF1F}"),
            "who is amiya"
        );
        assert_eq!(n("Why?\u{FF1F}"), "why");
        assert_eq!(n("\u{201C}Ch\u{2014}en\u{201D}!"), "ch en");
        assert_eq!(n("a, b"), "a b");
        assert_eq!(n("a,b"), "a b");
        assert_eq!(n("Who is Amiya\u{2026}"), "who is amiya");
        assert_eq!(n("the \u{FB01}nal battle"), "the final battle");
        assert_eq!(n("Episode \u{2167}"), "episode viii");
        assert_eq!(n("zero\u{200B}width"), "zerowidth");
        assert_eq!(n("Wi\u{0161}'adel"), "wi\u{0161}adel");
        // Full NFKC (2026-10-17): a decomposed accent composes onto its letter.
        assert_eq!(n("Wis\u{030C}'adel"), "wi\u{0161}adel");
    }

    #[test]
    fn normq_keeps_what_changes_meaning() {
        let n = normalize_question;
        assert_ne!(n("Did W kill Theresa?"), n("Did Theresa kill W?"));
        assert_ne!(n("chapter 10"), n("chapter 1"));
        assert_ne!(n("chapter 1-10"), n("chapter 110"));
        assert_eq!(n("chapter 1-10"), "chapter 1 10");
    }

    #[test]
    fn qkey_is_stable_and_shared_by_equivalent_questions() {
        let a = qkey(&normalize_question("Who is Amiya?"));
        let b = qkey(&normalize_question("who is amiya"));
        assert_eq!(a, b);
        assert_eq!(a.len(), 32);
        assert_ne!(a, qkey(&normalize_question("Who is Kal'tsit?")));
        // Full NFKC (2026-10-17): a decomposed accent keys like the precomposed letter.
        assert_eq!(
            normalize_question("Who is Ve\u{0301}ronique?"),
            normalize_question("Who is V\u{E9}ronique?")
        );
        assert_eq!(normalize_question("\u{FB01}nal battle"), "final battle");
    }

    #[test]
    fn validate_refuses_empty_and_long() {
        for normq in [true, false] {
            assert!(validate_question(" ?? ", normq).is_err());
            assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS + 1), normq).is_err());
            assert!(validate_question(&"a".repeat(MAX_QUESTION_CHARS), normq).is_ok());
        }
        // Punctuation only is empty under normQ.
        assert!(validate_question("!?,.", true).is_err());
    }

    #[test]
    fn admission_caps_per_rung_and_overflows_to_deferred() {
        let c = cfg();
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let cpu_only = RungFlags {
            cf: false,
            cpu: true,
        };
        let adm = |rungs, neurons_today, cpu_depth, cf_depth, cpu_workers, open_total| {
            admit_rung(
                &AdmissionInputs {
                    rung: RungInputs {
                        rungs,
                        neurons_today,
                        cpu_depth,
                    },
                    cf_depth,
                    cpu_workers,
                    open_total,
                },
                &c,
            )
        };
        assert_eq!(adm(both, 0, 0, 0, 1, 0), Admission::Now(Rung::Cf));
        // TREVOR_CF_QUEUE_MAX=4: the fifth goes to cpu.
        assert_eq!(adm(both, 0, 0, 3, 1, 0), Admission::Now(Rung::Cf));
        assert_eq!(adm(both, 0, 0, 4, 1, 0), Admission::Now(Rung::Cpu));
        // 6 per online cpu worker.
        assert_eq!(adm(cpu_only, 0, 5, 0, 1, 0), Admission::Now(Rung::Cpu));
        assert_eq!(
            adm(cpu_only, 0, 6, 0, 1, 10),
            Admission::Overflow {
                deferred: Some(Rung::Cpu)
            }
        );
        assert_eq!(adm(cpu_only, 0, 6, 0, 2, 10), Admission::Now(Rung::Cpu));
        assert_eq!(adm(cpu_only, 0, 11, 0, 2, 10), Admission::Now(Rung::Cpu));
        // A restart (no cpu worker online) keeps the one-worker cap.
        assert_eq!(adm(cpu_only, 0, 5, 0, 0, 0), Admission::Now(Rung::Cpu));
        // The global cap drops the deferred job, never the passages answer.
        assert_eq!(
            adm(cpu_only, 0, 6, 0, 1, 200),
            Admission::Overflow { deferred: None }
        );
        // cf only and full: deferred on cf.
        let cf_only = RungFlags {
            cf: true,
            cpu: false,
        };
        assert_eq!(
            adm(cf_only, 0, 0, 4, 0, 0),
            Admission::Overflow {
                deferred: Some(Rung::Cf)
            }
        );
        // cf spent and no cpu: plain passages, nothing to defer to.
        assert_eq!(
            adm(cf_only, 8000, 0, 0, 0, 0),
            Admission::Now(Rung::Passages)
        );
        assert_eq!(
            adm(RungFlags::default(), 0, 0, 0, 0, 0),
            Admission::Now(Rung::Passages)
        );
        // TREVOR_CF_QUEUE_MAX=0 lifts the cf cap.
        let nocap = TrevorConfig {
            cf_queue_max: 0,
            ..cfg()
        };
        let inputs = AdmissionInputs {
            rung: RungInputs {
                rungs: both,
                neurons_today: 0,
                cpu_depth: 0,
            },
            cf_depth: 1000,
            cpu_workers: 1,
            open_total: 0,
        };
        assert_eq!(admit_rung(&inputs, &nocap), Admission::Now(Rung::Cf));
    }

    #[test]
    fn mode_parses_and_steps_every_claim_down_unless_on() {
        assert_eq!(Mode::parse("on"), Some(Mode::On));
        assert_eq!(Mode::parse(" Passages "), Some(Mode::Passages));
        assert_eq!(Mode::parse("off"), Some(Mode::Off));
        assert_eq!(Mode::parse("maybe"), None);
        let both = RungFlags {
            cf: true,
            cpu: true,
        };
        let p = TrevorConfig {
            mode: Mode::Passages,
            ..cfg()
        };
        assert_eq!(settle_rung(Rung::Cf, both, 0, &p), Rung::Passages);
        assert_eq!(settle_rung(Rung::Cpu, both, 0, &p), Rung::Passages);
        assert_eq!(settle_rung(Rung::Cf, both, 0, &cfg()), Rung::Cf);
    }

    #[test]
    fn the_cpu_profile_is_mid_unless_switched_to_slim() {
        assert_eq!(Rung::Cpu.profile_with(Profile::Mid), Profile::Mid);
        assert_eq!(Rung::Cpu.profile_with(Profile::Slim), Rung::Cpu.profile());
        assert_eq!(Rung::Cf.profile_with(Profile::Mid), Profile::Full);
        assert_eq!(cfg().cpu_profile, Profile::Mid);
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
