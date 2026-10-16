//! The worker side, behind the scoped worker key (contract section 3): claim
//! a job (long-poll), heartbeat, stream deltas, post the result, publish a
//! corpus version; and the stale sweep that runs on its own tick.
use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use subtle::ConstantTimeEq;
use utoipa::ToSchema;
use uuid::Uuid;

use super::runtime::StreamEvent;
use super::{
    AnswerSource, CITATION_TEXT_CHARS, Era, LONG_POLL_SECS, MAX_ATTEMPTS, Profile,
    RUNNING_STALE_SECS, Rung, RungFlags, SWEEP_EVERY_SECS, WorkerAnswer, neurons, parse_server,
    settle_rung, trim_citation_text,
};
use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

/// The header the worker presents its key in.
pub const WORKER_KEY_HEADER: &str = "x-trevor-worker-key";

/// Proof the caller holds `TREVOR_WORKER_KEY`: the extractor every worker
/// route (and only those) takes.
///
/// An empty configured key turns the routes off with a 503. The global
/// service key is refused here, and nothing else accepts the worker key.
/// `TREVOR_SCOPED_WORKER_KEY=0` restores the old gate (service key only).
#[derive(Debug, Clone, Copy)]
pub struct WorkerKey;

/// The old gate: the service principal only, never a user token.
pub fn require_service(auth: &AuthUser) -> Result<(), ApiError> {
    if auth.user_id == "service" {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

/// The scoped check, pure: `presented` against `configured` in constant time.
pub fn check_worker_key(configured: &str, presented: Option<&[u8]>) -> Result<(), ApiError> {
    if configured.is_empty() {
        return Err(ApiError::ServiceUnavailableMessage(
            "the Trevor worker routes are off: TREVOR_WORKER_KEY is not set".into(),
        ));
    }
    match presented {
        Some(key) if !key.is_empty() && bool::from(key.ct_eq(configured.as_bytes())) => Ok(()),
        _ => Err(ApiError::Unauthorized),
    }
}

/// The request carries the valid scoped worker key (constant time): the rate
/// limiter lets it through, as it lets the service key. An invalid key, an
/// empty configured key or `TREVOR_SCOPED_WORKER_KEY=0` reads as false.
pub fn presents_worker_key(headers: &axum::http::HeaderMap, state: &AppState) -> bool {
    let cfg = state.trevor.config();
    cfg.scoped_worker_key
        && check_worker_key(
            &cfg.worker_key,
            headers.get(WORKER_KEY_HEADER).map(|v| v.as_bytes()),
        )
        .is_ok()
}

impl FromRequestParts<AppState> for WorkerKey {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let cfg = state.trevor.config();
        if !cfg.scoped_worker_key {
            let auth = AuthUser::from_request_parts(parts, state).await?;
            require_service(&auth)?;
            return Ok(Self);
        }
        check_worker_key(
            &cfg.worker_key,
            parts.headers.get(WORKER_KEY_HEADER).map(|v| v.as_bytes()),
        )?;
        Ok(Self)
    }
}

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

fn check_worker_id(id: &str) -> Result<(), ApiError> {
    if id.is_empty() || id.len() > 100 {
        return Err(ApiError::BadRequest("workerId is empty or too long".into()));
    }
    Ok(())
}

/// `worker/heartbeat`.
pub async fn heartbeat(state: &AppState, beat: &Heartbeat) -> Result<(), ApiError> {
    check_worker_id(&beat.worker_id)?;
    let busy = match beat.busy_job_id.as_deref().filter(|s| !s.is_empty()) {
        None => None,
        Some(s) => Some(
            Uuid::parse_str(s)
                .map_err(|_| ApiError::BadRequest("busyJobId is not a job id".into()))?,
        ),
    };
    let corpus = beat
        .corpus_version
        .as_deref()
        .filter(|v| !v.is_empty() && v.len() <= 100);
    let rungs = serde_json::to_value(beat.rungs).map_err(|e| ApiError::Internal(e.into()))?;
    q::upsert_worker(&state.db, &beat.worker_id, &rungs, corpus, busy).await?;
    Ok(())
}

/// Claim a job and settle its rung against the claiming worker.
async fn claim(
    state: &AppState,
    server: &str,
    worker_id: &str,
) -> Result<Option<WorkerJob>, ApiError> {
    state.trevor.claim_queries.fetch_add(1, Ordering::Relaxed);
    let Some(job) = q::claim_next(&state.db, server, worker_id).await? else {
        return Ok(None);
    };
    let assigned = Rung::parse_job(&job.rung);
    let (rungs, neurons_today) = q::worker_rungs_and_neurons(&state.db, worker_id).await?;
    let worker_rungs: RungFlags = rungs
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let rung = settle_rung(
        assigned,
        worker_rungs,
        neurons_today,
        &state.trevor.config(),
    );
    if rung != assigned {
        q::set_job_rung(&state.db, job.id, rung.as_str(), rung.profile().as_str()).await?;
    }
    state.trevor.mark_running(job.id, worker_id);
    Ok(Some(WorkerJob {
        job_id: job.id,
        question: job.question,
        server: job.server,
        horizon_pos: job.horizon_pos,
        rung,
        profile: rung.profile(),
        flags: Vec::new(),
    }))
}

/// `worker/next`: claim the oldest queued job of the worker's server, holding
/// an empty poll open up to [`LONG_POLL_SECS`] (under the 30 s handler
/// timeout). Idle, it waits on the enqueue [`tokio::sync::Notify`] and
/// queries nothing; the stale sweep runs on its own tick ([`spawn_sweep`]).
pub async fn next(state: &AppState, hello: WorkerHello) -> Result<Option<WorkerJob>, ApiError> {
    check_worker_id(&hello.worker_id)?;
    let server = parse_server(hello.server.as_deref())?;
    if !state.trevor.config().notify_poll {
        return next_polling(state, server, &hello.worker_id).await;
    }
    let deadline = Instant::now() + Duration::from_secs(LONG_POLL_SECS);
    loop {
        // Armed before the claim, so an enqueue between a miss and the wait
        // still wakes this poll.
        let notified = state.trevor.notify.notified();
        tokio::pin!(notified);
        notified.as_mut().enable();
        if let Some(job) = claim(state, server, &hello.worker_id).await? {
            return Ok(Some(job));
        }
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() || tokio::time::timeout(left, notified).await.is_err() {
            return Ok(None);
        }
    }
}

/// The old long-poll (`TREVOR_NOTIFY_POLL=0`): sweep, then one claim query a
/// second until a job or the deadline.
async fn next_polling(
    state: &AppState,
    server: &str,
    worker_id: &str,
) -> Result<Option<WorkerJob>, ApiError> {
    sweep(state).await?;
    let deadline = Instant::now() + Duration::from_secs(LONG_POLL_SECS);
    loop {
        if let Some(job) = claim(state, server, worker_id).await? {
            return Ok(Some(job));
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

/// Requeue stale running jobs (once) and expire old queued ones, waking
/// pollers for the requeued and ending the streams of the failed. Returns
/// how many jobs changed.
pub async fn sweep(state: &AppState) -> Result<usize, ApiError> {
    let cfg = state.trevor.config();
    let swept = q::sweep_stale(
        &state.db,
        RUNNING_STALE_SECS,
        cfg.queue_ttl_secs,
        MAX_ATTEMPTS,
        q::SweepRules {
            ttl_from_queued: cfg.ttl_from_queued,
            busy_protects: cfg.busy_protects,
            online_secs: super::ONLINE_SECS,
        },
    )
    .await?;
    let mut requeued = 0usize;
    for job in &swept {
        if job.status == "queued" {
            requeued += 1;
            state.trevor.clear_running(job.id);
        } else {
            state.trevor.finish(
                job.id,
                StreamEvent::Error(job.error.clone().unwrap_or_else(|| "failed".into())),
            );
        }
    }
    if requeued > 0 {
        state.trevor.job_enqueued();
    }
    if !swept.is_empty() {
        tracing::info!(
            requeued,
            failed = swept.len() - requeued,
            "trevor: swept stale jobs"
        );
    }
    Ok(swept.len())
}

/// The sweep's background tick (every [`SWEEP_EVERY_SECS`]). Under
/// `TREVOR_NOTIFY_POLL=0` it leaves the sweep to `worker/next`, as before,
/// and only prunes idle stream channels.
pub fn spawn_sweep(state: AppState) {
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_secs(SWEEP_EVERY_SECS));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            tick.tick().await;
            if state.trevor.config().notify_poll
                && let Err(e) = sweep(&state).await
            {
                tracing::warn!(error = %e, "trevor: stale sweep failed");
            }
            state.trevor.prune();
        }
    });
}

/// `worker/jobs/{jobId}/delta`: fan streamed text out to the job's SSE
/// readers. 404 unless the job is running under this worker.
pub async fn delta(state: &AppState, job_id: Uuid, d: WorkerDelta) -> Result<(), ApiError> {
    check_worker_id(&d.worker_id)?;
    let runs_here = match state.trevor.running_worker(job_id) {
        Some(w) => w == d.worker_id,
        // Claimed before a restart (or by the old poll): one lookup, then cached.
        None => match q::job_by_id(&state.db, job_id).await? {
            Some(job)
                if job.status == "running"
                    && job.worker_id.as_deref() == Some(d.worker_id.as_str()) =>
            {
                state.trevor.mark_running(job_id, &d.worker_id);
                true
            }
            _ => false,
        },
    };
    if !runs_here {
        return Err(ApiError::NotFoundMessage(
            "the job is unknown or not running under this worker".into(),
        ));
    }
    state.trevor.publish(
        job_id,
        StreamEvent::Delta {
            seq: d.seq,
            text: d.text,
        },
    );
    Ok(())
}

/// `worker/result`, from the raw body so its size is checked first: a body
/// over `TREVOR_RESULT_MAX_BYTES` is a 413 (the route maps
/// [`ResultError::TooLarge`]). The answer is stored under the corpus
/// version the worker reports; an answer from a version that is not the
/// active one still reaches the job's poller but is never served from cache.
pub async fn result(state: &AppState, body: &[u8]) -> Result<(), ResultError> {
    let cap = state.trevor.config().result_max_bytes;
    if cap > 0 && body.len() > cap {
        return Err(ResultError::TooLarge(cap));
    }
    let res: WorkerResult = serde_json::from_slice(body)
        .map_err(|e| ApiError::BadRequest(format!("result body: {e}")))?;
    store_result(state, res).await.map_err(ResultError::Api)
}

/// Why a result was refused.
#[derive(Debug)]
pub enum ResultError {
    TooLarge(usize),
    Api(ApiError),
}

impl From<ApiError> for ResultError {
    fn from(e: ApiError) -> Self {
        Self::Api(e)
    }
}

async fn store_result(state: &AppState, res: WorkerResult) -> Result<(), ApiError> {
    check_worker_id(&res.worker_id)?;
    if !res.ok {
        let error = res.error.unwrap_or_else(|| "the worker failed".to_owned());
        if !q::fail_job(&state.db, res.job_id, &res.worker_id, &error).await? {
            return Err(ApiError::Conflict(
                "job is not running under this worker".into(),
            ));
        }
        state.trevor.finish(res.job_id, StreamEvent::Error(error));
        return Ok(());
    }
    let mut answer = res
        .result
        .ok_or_else(|| ApiError::BadRequest("ok result without `result`".into()))?;
    if answer.rung == Rung::Cache {
        return Err(ApiError::BadRequest(
            "result.rung must be cf, cpu or passages".into(),
        ));
    }
    if answer.corpus_version.is_empty() || answer.corpus_version.len() > 100 {
        return Err(ApiError::BadRequest(
            "result.corpusVersion is missing or too long".into(),
        ));
    }
    for c in &mut answer.citations {
        c.text = trim_citation_text(&c.text, CITATION_TEXT_CHARS);
    }
    let stored = serde_json::to_value(&answer).map_err(|e| ApiError::Internal(e.into()))?;
    let cfg = state.trevor.config();
    let cost = neurons(answer.tokens_in, answer.tokens_out);
    let Some((_, job)) = q::complete_job(
        &state.db,
        res.job_id,
        &res.worker_id,
        &q::CompletedAnswer {
            version: &answer.corpus_version,
            rung: answer.rung.as_str(),
            source: answer.source.as_str(),
            answer: &stored,
            neurons: cost,
            reported_cf: answer.rung == Rung::Cf,
            cacheable: !(cfg.cache_live_only && answer.source == AnswerSource::Passages),
            version_check: cfg.version_check,
        },
    )
    .await?
    else {
        // A late result (the job was requeued or reclaimed): its answer is
        // refused, but Cloudflare already spent the Neurons.
        let job_cf = q::job_by_id(&state.db, res.job_id)
            .await?
            .is_some_and(|j| j.rung == "cf");
        if answer.rung == Rung::Cf || job_cf {
            q::book_neurons(&state.db, cost).await?;
        }
        return Err(ApiError::Conflict(
            "job is not running under this worker".into(),
        ));
    };
    match super::ask::job_view(state, &job).await {
        Ok(view) => state
            .trevor
            .finish(job.id, StreamEvent::Done(Box::new(view))),
        Err(e) => {
            tracing::warn!(error = %e, job = %job.id, "trevor: done view failed");
            state.trevor.finish(
                job.id,
                StreamEvent::Error("the answer could not be read".into()),
            );
        }
    }
    Ok(())
}

/// What the activating call must carry: the manifest with a well-formed
/// `storyOrder` and `eras`, and the gate report.
fn check_activation(req: &PublishVersion) -> Result<(), ApiError> {
    let manifest = req
        .manifest
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("the activating call must resend `manifest`".into()))?;
    if req.gate_report.is_none() {
        return Err(ApiError::BadRequest(
            "the activating call must resend `gateReport`".into(),
        ));
    }
    let order_ok = manifest
        .get("storyOrder")
        .and_then(Value::as_array)
        .is_some_and(|ids| ids.iter().all(Value::is_string));
    if !order_ok {
        return Err(ApiError::BadRequest(
            "manifest.storyOrder must be an array of story ids".into(),
        ));
    }
    serde_json::from_value::<Vec<Era>>(manifest.get("eras").cloned().unwrap_or(Value::Null))
        .map_err(|e| ApiError::BadRequest(format!("manifest.eras: {e}")))?;
    Ok(())
}

/// `versions`.
pub async fn publish(state: &AppState, req: PublishVersion) -> Result<PublishOutcome, ApiError> {
    let server = parse_server(req.server.as_deref())?;
    if req.version.is_empty() || req.version.len() > 100 {
        return Err(ApiError::BadRequest("version is empty or too long".into()));
    }
    if req.activate {
        check_activation(&req)?;
    }
    let panels: Vec<q::PanelRow> = req
        .panels
        .into_iter()
        .map(|(story_id, panels)| q::PanelRow {
            gate_pos: panels
                .get("position")
                .and_then(Value::as_i64)
                .and_then(|p| i32::try_from(p).ok()),
            story_id,
            panels,
        })
        .collect();
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
    Ok(PublishOutcome {
        version: req.version,
        server: server.to_owned(),
        panels: panels.len(),
        active: req.activate,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_worker_key_is_checked_and_an_empty_one_disables_the_routes() {
        assert!(matches!(
            check_worker_key("", Some(b"anything")),
            Err(ApiError::ServiceUnavailableMessage(_))
        ));
        assert!(matches!(
            check_worker_key("k-123", None),
            Err(ApiError::Unauthorized)
        ));
        assert!(matches!(
            check_worker_key("k-123", Some(b"")),
            Err(ApiError::Unauthorized)
        ));
        assert!(matches!(
            check_worker_key("k-123", Some(b"k-124")),
            Err(ApiError::Unauthorized)
        ));
        assert!(check_worker_key("k-123", Some(b"k-123")).is_ok());
    }

    fn publish_req(v: Value) -> PublishVersion {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn activation_requires_the_manifest_order_eras_and_gate_report() {
        let ok = serde_json::json!({
            "version": "v1", "gateReport": {"ok": true},
            "manifest": {"builtFrom": "26-01", "storyOrder": ["a", "b"],
                         "eras": [{"id": "main_0", "name": "Evil Time", "position": 1}]}
        });
        assert!(check_activation(&publish_req(ok.clone())).is_ok());
        let mut no_report = ok.clone();
        no_report.as_object_mut().unwrap().remove("gateReport");
        assert!(check_activation(&publish_req(no_report)).is_err());
        let mut bad_eras = ok.clone();
        bad_eras["manifest"]["eras"] = serde_json::json!([{"id": "x"}]);
        assert!(check_activation(&publish_req(bad_eras)).is_err());
        let mut bad_order = ok;
        bad_order["manifest"]["storyOrder"] = serde_json::json!([1, 2]);
        assert!(check_activation(&publish_req(bad_order)).is_err());
    }

    #[test]
    fn a_result_drops_the_passages_list_and_keeps_tokens() {
        let r: WorkerResult = serde_json::from_value(serde_json::json!({
            "jobId": Uuid::nil(), "workerId": "w", "ok": true,
            "result": {
                "answer": "A.", "rung": "cf", "profile": "full", "source": "live",
                "citations": [{"n": 1, "storyId": "s", "chunkId": "c", "lineStart": 1,
                               "lineEnd": 2, "text": "t"}],
                "route": "who", "form": "short", "corpusVersion": "v1",
                "tokensIn": 2000, "tokensOut": 250,
                "timings": {"routeMs": 1, "answerMs": 2, "totalMs": 3},
                "passages": [{"text": "every passage"}]
            }
        }))
        .unwrap();
        let stored = serde_json::to_value(r.result.unwrap()).unwrap();
        assert!(stored.get("passages").is_none());
        assert_eq!(stored["tokensIn"], 2000);
        assert_eq!(stored["citations"][0]["storyId"], "s");
    }
}
