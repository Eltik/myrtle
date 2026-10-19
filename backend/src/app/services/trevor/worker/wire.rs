//! The worker-facing request and response bodies.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::services::trevor::{Citation, Guard, Profile, Rung, RungFlags, WorkerAnswer};

/// `worker/next`: `{workerId}`. `server` (default `en`) picks the queue.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHello {
    pub worker_id: String,
    #[serde(default)]
    pub server: Option<String>,
}

/// `worker/heartbeat`, every 30 s from the worker's own task (mid-job too).
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Heartbeat {
    pub worker_id: String,
    #[serde(default)]
    pub rungs: RungFlags,
    #[serde(default)]
    pub corpus_version: Option<String>,
    /// The job the worker is answering, or null when idle.
    #[serde(default)]
    pub busy_job_id: Option<String>,
    /// The power guard (contract v1.1 section 2.1); absent from an older
    /// worker, which reads as no guard.
    #[serde(default)]
    pub guard: Option<Guard>,
}

/// A claimed job, as `ask --worker` reads it.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerJob {
    pub job_id: Uuid,
    pub question: String,
    pub server: String,
    /// Keep only chunks at or before this gatePos; null = no limit
    /// (internal jobs and spoiler-mode asks, contract v1.1 section 8).
    pub horizon_pos: Option<i32>,
    /// `cf`, `cpu` or `passages`.
    pub rung: Rung,
    pub profile: Profile,
    pub flags: Vec<String>,
}

/// `worker/jobs/{jobId}/delta`: streamed answer text.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerDelta {
    pub worker_id: String,
    pub seq: u64,
    pub text: String,
}

/// `worker/result`: `ok` with the structured `result`, or not with `error`.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerResult {
    pub job_id: Uuid,
    pub worker_id: String,
    pub ok: bool,
    #[serde(default)]
    pub result: Option<WorkerAnswer>,
    #[serde(default)]
    pub error: Option<String>,
}

/// `versions`: publish a corpus version's manifest and panels.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishVersion {
    pub version: String,
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub built_at: Option<DateTime<Utc>>,
    /// Required on the activating call.
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub gate_report: Option<Value>,
    /// `{builtFrom, storyOrder, eras, ...}`; required on the activating call.
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub manifest: Option<Value>,
    /// Story id -> that story's panel JSON (its `position` is the gatePos).
    /// Sent in several calls with `activate: false` and a final call that
    /// activates and resends `manifest` and `gateReport`.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub panels: BTreeMap<String, Value>,
    /// Make this the server's active version (default true).
    #[serde(default = "yes")]
    pub activate: bool,
    /// `{chunkId: sha}` of the version's chunks (contract v1.1 section 3b),
    /// in any of the calls (they accumulate). Activation carries a cached
    /// answer of the previous version over when every chunk it cites is
    /// here with the same sha; a version published without it carries none.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub chunk_shas: BTreeMap<String, String>,
    /// The bank's answers (contract v1.1 section 3a, addition 2026-10-19),
    /// read on the activating call only: each is imported into the answer
    /// cache of this version under its normalized question, so the next
    /// asker gets it as a cache hit. Ignored under `TREVOR_BANK_IMPORT=0`.
    #[serde(default)]
    pub bank_answers: Vec<BankAnswer>,
}

/// Where a bank answer comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
#[schema(as = TrevorBankSource)]
pub enum BankSource {
    /// A pre-generated question (`scripts/answer_bank.py`).
    Bank,
    /// A reader's live question answered again (`scripts/reanswer.py`).
    Reanswer,
}

impl BankSource {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bank => "bank",
            Self::Reanswer => "reanswer",
        }
    }
}

/// One bank answer as the publish sends it.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BankAnswer {
    /// The question as asked; the backend normalizes it into the cache key.
    pub question: String,
    /// Other wordings of the same question, each cached under its own key.
    #[serde(default)]
    pub aliases: Vec<String>,
    pub answer: String,
    /// The cited passages, each with its chunk sha (every one must match the
    /// version's `chunkShas`, or the answer is not imported).
    pub citations: Vec<Citation>,
    /// The gatePos a reader must have reached: the highest gatePos over the
    /// answer's gate stories and cited sources, as `scripts/panels.py` gates
    /// a bank entry. Null when one of them is undatable: the answer is then
    /// cached for spoiler mode only.
    #[serde(default)]
    pub gate_pos: Option<i32>,
    pub source: BankSource,
    /// The profile that made it (default `full`).
    #[serde(default)]
    pub profile: Option<Profile>,
    /// When the answer was made, if the bank recorded it. An imported answer
    /// replaces a worker's cached answer under the same key only when it is
    /// newer than that answer; without this time it never does.
    #[serde(default)]
    pub answered_at: Option<DateTime<Utc>>,
}

/// What the bank import of a publish did.
#[derive(Debug, Clone, Default, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BankOutcome {
    /// Bank answers the call carried.
    pub sent: usize,
    /// Answers refused before the database: an empty answer or question, no
    /// citation, a citation without a sha, or a gatePos below 1.
    pub invalid: usize,
    /// Cache rows the valid answers make: one per wording and era bucket
    /// (the first era at or after the gatePos, and the spoiler-mode bucket).
    pub rows: usize,
    /// Rows written.
    pub imported: u64,
    /// Rows dropped because a cited chunk's sha is not the version's.
    pub stale: u64,
    /// Rows left out because a worker's newer answer holds the key.
    pub kept: u64,
    /// The import failed and was rolled back; the publish went on.
    pub failed: bool,
}

const fn yes() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublishOutcome {
    pub version: String,
    pub server: String,
    pub panels: usize,
    pub active: bool,
    /// Chunk shas stored by this call.
    pub chunk_shas: usize,
    /// Cached answers carried over from the previous version on activation.
    pub carried: u64,
    /// The bank import (contract v1.1 section 3a, addition 2026-10-19).
    pub bank: BankOutcome,
}

/// `worker/popular`: one question to re-answer after a publish.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarmQuestion {
    /// The latest asked text.
    pub question: String,
    /// The cache's normalized form (contract v1.1 section 1a).
    pub norm_q: String,
    /// The era boundary readers asked it under: warm it with this horizon.
    pub era_pos: i32,
    /// Asks in the last 30 days, twins included.
    pub hits: i64,
}

/// `GET /trevor/worker/popular`.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarmList {
    pub version: String,
    pub items: Vec<WarmQuestion>,
}

/// `POST /trevor/worker/warm`: queue internal deferred re-answers (no user).
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarmRequest {
    /// Must be the server's active version (409 otherwise).
    pub version: String,
    #[serde(default)]
    pub server: Option<String>,
    pub items: Vec<WarmItem>,
}

#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarmItem {
    pub question: String,
    pub era_pos: i32,
}

#[derive(Debug, Clone, Default, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WarmOutcome {
    /// New deferred internal jobs.
    pub queued: usize,
    /// Already cached in the version under that era: nothing queued.
    pub cached: usize,
    /// An open job (normal or deferred) already asks it: joined.
    pub joined: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_heartbeat_takes_the_guard_and_an_old_one_has_none() {
        let hb: Heartbeat = serde_json::from_value(serde_json::json!({
            "workerId": "w", "rungs": {"cf": false, "cpu": false},
            "guard": {"state": "tripped", "reason": "mem 700 MB < 800", "since": 1760000000}
        }))
        .unwrap();
        let g = hb.guard.unwrap();
        assert_eq!(g.state, crate::app::services::trevor::GuardState::Tripped);
        assert_eq!(g.since, Some(1_760_000_000));
        let old: Heartbeat =
            serde_json::from_value(serde_json::json!({"workerId": "w", "rungs": {"cf": true}}))
                .unwrap();
        assert!(old.guard.is_none());
    }
}
