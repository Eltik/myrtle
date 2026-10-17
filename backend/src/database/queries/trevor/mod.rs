//! Trevor's tables (v039): versions and their panels, the job queue, answers,
//! worker heartbeats, the Neuron ledger, asks and feedback. Callers live in
//! `app/services/trevor`.
//!
//! - [`versions`]: corpus versions, their panels and eras, the publish.
//! - [`answers`]: the answer cache.
//! - [`admission`]: the counts and lock admission control reads.
//! - [`jobs`]: the queue: insert, claim, ETAs, the stale sweep.
//! - [`complete`]: a job's end: its answer, the Neuron ledger, failure.
//! - [`workers`]: heartbeats and what they say about the fleet.
//! - [`asks`]: the ask log ("My questions", popular, warm-up) and feedback.

pub mod admission;
pub mod answers;
pub mod asks;
pub mod complete;
pub mod jobs;
pub mod versions;
pub mod workers;

pub use admission::*;
pub use answers::*;
pub use asks::*;
pub use complete::*;
pub use jobs::*;
pub use versions::*;
pub use workers::*;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow)]
pub struct AnswerRow {
    pub id: Uuid,
    pub version: String,
    pub server: String,
    pub era_pos: i32,
    pub rung: String,
    pub source: String,
    pub answer: Value,
    pub created_at: DateTime<Utc>,
}

const ANSWER_COLUMNS: &str = "id, version, server, era_pos, rung, source, answer, created_at";

#[derive(Debug, Clone, FromRow)]
pub struct JobRow {
    pub id: Uuid,
    pub kind: String,
    pub user_id: Option<Uuid>,
    pub question: String,
    pub qkey: String,
    pub horizon_pos: Option<i32>,
    pub server: String,
    pub corpus_version: Option<String>,
    pub rung: String,
    pub profile: String,
    pub status: String,
    pub worker_id: Option<String>,
    pub error: Option<String>,
    pub attempts: i16,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub answer_id: Option<Uuid>,
    pub deferred: bool,
}

const JOB_COLUMNS: &str = "id, kind, user_id, question, qkey, horizon_pos, server, corpus_version, rung, profile, \
     status, worker_id, error, attempts, created_at, started_at, finished_at, answer_id, deferred";

/// The UTC day the Neuron ledger books against.
const UTC_DAY: &str = "(now() AT TIME ZONE 'utc')::date";
