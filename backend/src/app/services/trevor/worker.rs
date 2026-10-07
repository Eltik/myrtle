//! The worker side, behind the service key: claim a job (long-poll), post its
//! result, heartbeat, publish a corpus version.
//!
//! The shapes match `trevor/src/bin/ask/serve.rs` (`ask --worker URL`): every
//! call carries `{workerId, server, corpusVersion, slots}`, `next` answers 204
//! or a [`WorkerJob`], `result` takes [`WorkerResult`].
use std::collections::BTreeMap;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

use super::{LONG_POLL_SECS, MAX_ATTEMPTS, QUEUED_EXPIRE_SECS, RUNNING_STALE_SECS, parse_server};
use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

/// Worker routes take the service key only, never a user token, whatever its role.
pub fn require_service(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.user_id == "service" {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// `worker/next` and `worker/heartbeat` bodies.
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerHello {
    pub worker_id: String,
    #[serde(default)]
    pub server: Option<String>,
    #[serde(default)]
    pub corpus_version: Option<String>,
    #[serde(default = "one")]
    pub slots: u16,
}

const fn one() -> u16 {
    1
}

/// What the heartbeat key holds.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Heartbeat {
    pub worker_id: String,
    pub corpus_version: Option<String>,
    pub slots: u16,
    pub at: u64,
}

/// A claimed job, as `ask --worker` reads it.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerJob {
    pub job_id: Uuid,
    pub question: String,
    pub horizon: Option<String>,
    pub server: String,
    pub flags: Vec<String>,
}

/// `worker/result`: `ok` with the structured `result`, or not with `error`
/// (and the HTTP-style `status` the worker would have answered).
#[derive(Debug, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkerResult {
    pub job_id: Uuid,
    pub worker_id: String,
    pub ok: bool,
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub status: Option<u16>,
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
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub gate_report: Option<Value>,
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub manifest: Option<Value>,
    /// Story id -> that story's panel JSON. May be sent in several calls with
    /// `activate: false` and a final call that activates.
    #[serde(default)]
    #[schema(value_type = Object)]
    pub panels: BTreeMap<String, Value>,
    /// Make this the server's active version (default true).
    #[serde(default = "yes")]
    pub activate: bool,
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
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

fn check_worker_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() || id.len() > 100 {
        return Err(ApiError::BadRequest("workerId is empty or too long".into()));
    }
    Ok(())
}

/// `worker/heartbeat`; `next` also counts as one.
pub async fn heartbeat(state: &AppState, hello: &WorkerHello) -> Result<(), ApiError> {
    check_worker_id(&hello.worker_id)?;
    let server = parse_server(hello.server.as_deref())?;
    let beat = Heartbeat {
        worker_id: hello.worker_id.clone(),
        corpus_version: hello.corpus_version.clone(),
        slots: hello.slots.max(1),
        at: now_secs(),
    };
    state
        .cache
        .set(&CacheKey::TrevorWorker { server }, &beat)
        .await;
    Ok(())
}

/// `worker/next`: claim the oldest queued job of the worker's server, holding
/// an empty poll open up to [`LONG_POLL_SECS`] (under the 30 s handler
/// timeout). Stale running jobs are requeued first, so a worker that died
/// mid-answer costs one timeout, not the question.
pub async fn next(state: &AppState, hello: WorkerHello) -> Result<Option<WorkerJob>, ApiError> {
    heartbeat(state, &hello).await?;
    let server = parse_server(hello.server.as_deref())?;
    let (requeued, expired) = q::sweep_stale(
        &state.db,
        RUNNING_STALE_SECS,
        QUEUED_EXPIRE_SECS,
        MAX_ATTEMPTS,
    )
    .await?;
    if requeued + expired > 0 {
        tracing::info!(requeued, expired, "trevor: swept stale jobs");
    }
    let deadline = Instant::now() + Duration::from_secs(LONG_POLL_SECS);
    loop {
        if let Some(job) = q::claim_next(&state.db, server, &hello.worker_id).await? {
            return Ok(Some(WorkerJob {
                job_id: job.id,
                question: job.question,
                horizon: (!job.horizon.is_empty()).then_some(job.horizon),
                server: job.server,
                flags: Vec::new(),
            }));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// `worker/result`. The answer is stored under the corpus version the worker
/// reports in `result.corpusVersion`; an answer from a version that is not the
/// active one still reaches the job's poller but is never served from cache.
pub async fn result(state: &AppState, res: WorkerResult) -> Result<(), ApiError> {
    check_worker_id(&res.worker_id)?;
    if res.ok {
        let result = res
            .result
            .ok_or_else(|| ApiError::BadRequest("ok result without `result`".into()))?;
        let version = result
            .get("corpusVersion")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| ApiError::BadRequest("result.corpusVersion is missing".into()))?
            .to_owned();
        q::complete_job(&state.db, res.job_id, &res.worker_id, &version, &result)
            .await?
            .ok_or_else(|| ApiError::Conflict("job is not running under this worker".into()))?;
    } else {
        let error = res.error.unwrap_or_else(|| "the worker failed".to_owned());
        let error = match res.status {
            Some(code) => format!("{code}: {error}"),
            None => error,
        };
        if !q::fail_job(&state.db, res.job_id, &res.worker_id, &error).await? {
            return Err(ApiError::Conflict(
                "job is not running under this worker".into(),
            ));
        }
    }
    Ok(())
}

/// `versions`.
pub async fn publish(state: &AppState, req: PublishVersion) -> Result<PublishOutcome, ApiError> {
    let server = parse_server(req.server.as_deref())?;
    if req.version.is_empty() || req.version.len() > 100 {
        return Err(ApiError::BadRequest("version is empty or too long".into()));
    }
    let panels: Vec<(String, Value)> = req.panels.into_iter().collect();
    q::publish_version(
        &state.db,
        &req.version,
        server,
        req.built_at,
        req.gate_report.as_ref(),
        req.manifest.as_ref(),
        &panels,
        req.activate,
    )
    .await?;
    // Panels are keyed by story, not version: clear them so the next read
    // fetches the newly active version's. Answers carry the version in their
    // key and need nothing.
    state
        .cache
        .invalidate_by_prefix(&format!("trevor:panels:{server}:"))
        .await;
    Ok(PublishOutcome {
        version: req.version,
        server: server.to_owned(),
        panels: panels.len(),
        active: req.activate,
    })
}
