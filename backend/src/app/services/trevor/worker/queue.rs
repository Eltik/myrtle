//! The queue as a worker sees it: heartbeat, claim (Notify wake-up or the
//! old long-poll), stream deltas, and the stale sweep on its own tick.

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::trevor::runtime::StreamEvent;
use crate::app::services::trevor::{
    LONG_POLL_SECS, MAX_ATTEMPTS, Mode, ONLINE_SECS, RUNNING_STALE_SECS, Rung, RungFlags,
    SWEEP_EVERY_SECS, parse_server, settle_rung, worker_horizon,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::{Heartbeat, WorkerDelta, WorkerHello, WorkerJob};

pub(super) fn check_worker_id(id: &str) -> Result<(), ApiError> {
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
    let guard = beat
        .guard
        .as_ref()
        .map(|g| {
            let mut g = g.clone();
            g.reason = g.reason.map(|r| r.chars().take(300).collect());
            serde_json::to_value(g)
        })
        .transpose()
        .map_err(|e| ApiError::Internal(e.into()))?;
    q::upsert_worker(
        &state.db,
        &beat.worker_id,
        &rungs,
        corpus,
        busy,
        guard.as_ref(),
    )
    .await?;
    // A guard trip pings at once; an alert never fails the heartbeat.
    if let Err(e) = crate::app::services::trevor::alerts::after_heartbeat(state).await {
        tracing::warn!(error = %e, "trevor: guard alert check failed");
    }
    Ok(())
}

/// Claim a job and settle its rung against the claiming worker.
async fn claim(
    state: &AppState,
    server: &str,
    worker_id: &str,
) -> Result<Option<WorkerJob>, ApiError> {
    let cfg = state.trevor.config();
    // TREVOR_MODE=off: nothing is claimed; queued jobs wait (or expire).
    if cfg.mode == Mode::Off {
        return Ok(None);
    }
    state.trevor.claim_queries.fetch_add(1, Ordering::Relaxed);
    let (rungs, _) = q::worker_rungs_and_neurons(&state.db, worker_id).await?;
    let worker_rungs: RungFlags = rungs
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let rules = q::ClaimRules {
        admission: cfg.admission,
        fair: cfg.fair_claim,
        by_rung: cfg.claim_by_rung,
        cf: worker_rungs.cf,
        cpu: worker_rungs.cpu,
        online_secs: ONLINE_SECS,
    };
    let Some(job) = q::claim_next(&state.db, server, worker_id, rules).await? else {
        return Ok(None);
    };
    let assigned = Rung::parse_job(&job.rung);
    let neurons_today = q::neurons_today(&state.db).await?;
    let rung = settle_rung(assigned, worker_rungs, neurons_today, &cfg);
    let profile = rung.profile_with(cfg.cpu_profile);
    if rung != assigned || job.profile != profile.as_str() {
        q::set_job_rung(&state.db, job.id, rung.as_str(), profile.as_str()).await?;
    }
    state.trevor.mark_running(job.id, worker_id);
    Ok(Some(WorkerJob {
        job_id: job.id,
        question: job.question,
        server: job.server,
        // A spoiler-mode job's no-limit bucket reaches the worker as null.
        horizon_pos: worker_horizon(job.horizon_pos),
        rung,
        profile,
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
            online_secs: crate::app::services::trevor::ONLINE_SECS,
            max_run_secs: cfg.job_max_run_secs,
            deferred_ttl_secs: cfg.deferred_ttl_secs,
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
/// and only prunes idle stream channels. Also starts the alert evaluator
/// (contract v1.1 section 2.2), which does nothing under `TREVOR_ALERTS=0`.
pub fn spawn_sweep(state: AppState) {
    crate::app::services::trevor::alerts::spawn_evaluator(state.clone());
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
