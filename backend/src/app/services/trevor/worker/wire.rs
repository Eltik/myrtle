//! The worker-facing request and response bodies.

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::app::services::trevor::{Guard, Profile, Rung, RungFlags, WorkerAnswer};

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
    /// (internal jobs only).
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
