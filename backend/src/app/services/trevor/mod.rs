//! Trevor on the backend: the light plumbing between readers and a GPU worker
//! (design: `trevor/design/trevor-integration.md`, sections 2, 4, 6 and 10).
//!
//! The backend never runs a model. It normalizes a question into a cache key,
//! answers from the cache (Redis, then Postgres) when it can, and otherwise
//! queues a job that a worker pulls (`worker/next`), answers and posts back
//! (`worker/result`). Workers connect out with the service key, so a worker on
//! the Mac, an always-on box or the VPS itself is the same poller.
//!
//! - [`ask`]: the reader side (ask, job status, panels, feedback).
//! - [`worker`]: the worker side (claim, result, heartbeat, version publish).

pub mod ask;
pub mod worker;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::error::ApiError;

/// Free questions a signed-in user may ask per clock hour (Ian, 2026-10-07).
pub const ASK_PER_HOUR: u64 = 30;
/// Open (queued or running) jobs one user may hold at once.
pub const MAX_OPEN_PER_USER: i64 = 3;
/// Queued jobs per server before ask answers "Trevor is busy" (503).
pub const MAX_QUEUED: i64 = 200;
/// The longest question accepted, in characters.
pub const MAX_QUESTION_CHARS: usize = 500;
/// Seconds one uncached answer takes on one slot (22 to 28 s measured on the
/// M5 Pro, design section 3); only the ETA uses it.
pub const ANSWER_SECS: i64 = 25;
/// A running job whose worker has not answered in this long goes back to the
/// queue (or fails once it has had `MAX_ATTEMPTS`).
pub const RUNNING_STALE_SECS: i64 = 300;
pub const MAX_ATTEMPTS: i16 = 2;
/// A queued job nobody claimed in this long fails with "expired in the queue".
pub const QUEUED_EXPIRE_SECS: i64 = 1800;
/// How long `worker/next` holds an empty poll open, under the 30 s handler timeout.
pub const LONG_POLL_SECS: u64 = 20;

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

/// The spoiler horizon as stored: a story id, or '' for none.
pub fn horizon_key(horizon: Option<&str>) -> &str {
    horizon.map_or("", str::trim)
}

/// Seconds until a job at `position` (1-based) is answered, with `slots`
/// workers draining the queue. Jobs ahead plus this one, each one answer long.
pub fn eta_secs(position: i64, slots: i64) -> i64 {
    let slots = slots.max(1);
    // Ceiling division: the job at position `slots + 1` waits a full round.
    (position.max(1) + slots - 1) / slots * ANSWER_SECS
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
    pub server: Option<String>,
    /// The story being read, if any: the horizon when the user has no read
    /// progress yet.
    #[serde(default)]
    pub story_id: Option<String>,
    /// An explicit spoiler horizon (a story id). Absent: the user's
    /// furthest-read story.
    #[serde(default)]
    pub horizon: Option<String>,
    /// True lifts the horizon: the answer may use every released story.
    #[serde(default)]
    pub no_horizon: bool,
}

/// One answer as the reader sees it. `answer` is the worker's structured
/// result verbatim (answer text, citations, route, corpus version, timings).
#[derive(Debug, Clone, Serialize, Deserialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct TrevorAnswer {
    pub id: Uuid,
    pub version: String,
    pub server: String,
    /// The horizon the answer was made under, null for none.
    pub horizon: Option<String>,
    #[schema(value_type = Object)]
    pub answer: Value,
}

/// A queued question.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct QueuedJob {
    pub job_id: Uuid,
    /// 1-based place in the server's queue; 0 once a worker took it.
    #[ts(type = "number")]
    pub position: i64,
    /// Seconds until an answer is expected, assuming a worker is online.
    #[ts(type = "number")]
    pub eta: i64,
    /// False when no worker has sent a heartbeat recently: the job waits.
    pub worker_online: bool,
}

/// What `POST /trevor/ask` answers: an answer at once, or a job to poll.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(tag = "status", rename_all = "camelCase")]
#[ts(export)]
pub enum AskResponse {
    Answered(TrevorAnswer),
    Queued(QueuedJob),
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
}

/// `GET /trevor/jobs/{id}`.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct JobView {
    pub job_id: Uuid,
    pub status: JobStatus,
    #[ts(type = "number")]
    pub position: i64,
    #[ts(type = "number")]
    pub eta: i64,
    pub worker_online: bool,
    /// Present once `status` is `done`.
    pub answer: Option<TrevorAnswer>,
    /// Present once `status` is `failed`.
    pub error: Option<String>,
}

/// `GET /trevor/story/{storyId}/panels`: the published panel JSON verbatim.
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
    fn horizon_key_maps_none_to_empty() {
        assert_eq!(horizon_key(None), "");
        assert_eq!(horizon_key(Some(" main_01 ")), "main_01");
    }

    #[test]
    fn eta_rounds_up_to_whole_rounds() {
        assert_eq!(eta_secs(1, 1), ANSWER_SECS);
        assert_eq!(eta_secs(3, 1), 3 * ANSWER_SECS);
        assert_eq!(eta_secs(2, 2), ANSWER_SECS);
        assert_eq!(eta_secs(3, 2), 2 * ANSWER_SECS);
        assert_eq!(eta_secs(0, 0), ANSWER_SECS);
    }
}
