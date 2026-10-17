//! The reader-facing request and response bodies.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use ts_rs::TS;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{AnswerSource, CfBudget, GuardState, Mode, Rung, RungFlags, SpoilerSafe, WorkerCounts};

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

/// `GET /trevor/popular`: the reader's era and its popular questions.
#[derive(Debug, Clone, Serialize, TS, ToSchema)]
#[serde(rename_all = "camelCase")]
#[ts(export, rename = "TrevorPopular")]
#[schema(as = TrevorPopular)]
pub struct Popular {
    /// The era the list is drawn from; null signed out and before the first
    /// era (nothing released), when the list is empty.
    pub spoiler_safe_up_to: Option<SpoilerSafe>,
    /// Most askers first, then most hits.
    pub items: Vec<PopularQuestion>,
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
