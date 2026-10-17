//! Rung ETAs from recent jobs, and a job as its poller sees it.

use chrono::Utc;
use serde_json::Value;

use crate::app::error::ApiError;
use crate::app::services::trevor::{JobStatus, JobView, ONLINE_SECS, Rung, RungFlags};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::view;

pub(super) fn rung_flags(v: Option<&Value>) -> RungFlags {
    v.and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default()
}

/// Seconds per answer on each rung: the median of its recent finished jobs,
/// or the contract's fallback while there are too few.
async fn rung_etas(state: &AppState) -> Result<[(Rung, i64); 3], ApiError> {
    let rows = q::rung_etas(&state.db).await?;
    let pick = |rung: Rung| {
        rows.iter()
            .find(|(r, _, _)| r == rung.as_str())
            .and_then(|(_, median, n)| {
                (*n >= crate::app::services::trevor::ETA_MIN_SAMPLES)
                    .then_some(*median)
                    .flatten()
                    .map(|m| m.ceil() as i64)
            })
            .unwrap_or_else(|| rung.fallback_eta_secs())
    };
    Ok([
        (Rung::Cf, pick(Rung::Cf)),
        (Rung::Cpu, pick(Rung::Cpu)),
        (Rung::Passages, pick(Rung::Passages)),
    ])
}

fn eta_of(etas: &[(Rung, i64); 3], rung: Rung) -> i64 {
    etas.iter()
        .find(|(r, _)| *r == rung)
        .map_or_else(|| rung.fallback_eta_secs(), |(_, s)| *s)
}

/// The ETA of a queued or running job; null while no worker is online.
/// Queued: every queued job up to this one, each at its rung's answer time,
/// shared across the online workers. Running: its rung's answer time less
/// what it has run.
async fn job_eta(
    state: &AppState,
    job: &q::JobRow,
    status: JobStatus,
    online: i64,
) -> Result<Option<i64>, ApiError> {
    if online == 0 || status.is_terminal() {
        return Ok(None);
    }
    let etas = rung_etas(state).await?;
    let rung = Rung::parse_job(&job.rung);
    if status == JobStatus::Running {
        let ran = job
            .started_at
            .map_or(0, |at| (Utc::now() - at).num_seconds().max(0));
        return Ok(Some((eta_of(&etas, rung) - ran).max(1)));
    }
    let ahead = q::queued_ahead_by_rung(&state.db, job).await?;
    let total: i64 = ahead
        .iter()
        .map(|(r, n)| n * eta_of(&etas, Rung::parse_job(r)))
        .sum();
    Ok(Some((total.max(1) + online - 1) / online))
}

/// The view of a job as `GET /trevor/jobs/{id}` and the SSE `done` event
/// show it.
pub async fn job_view(state: &AppState, job: &q::JobRow) -> Result<JobView, ApiError> {
    let status = JobStatus::parse(&job.status);
    let (online, _, _) = q::workers_online(&state.db, ONLINE_SECS).await?;
    let position = if status == JobStatus::Queued {
        q::queued_ahead_by_rung(&state.db, job)
            .await?
            .iter()
            .map(|(_, n)| n)
            .sum()
    } else {
        0
    };
    let answer = match job.answer_id {
        Some(id) if status == JobStatus::Done => {
            q::answer_by_id(&state.db, id).await?.as_ref().map(view)
        }
        _ => None,
    };
    Ok(JobView {
        job_id: job.id,
        status,
        rung: Rung::parse_job(&job.rung),
        position,
        eta_secs: job_eta(state, job, status, online).await?,
        worker_online: online > 0,
        answer,
        error: job.error.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rung_flags_read_a_heartbeat_and_default_to_none() {
        assert_eq!(rung_flags(None), RungFlags::default());
        let v = serde_json::json!({ "cf": true, "cpu": false });
        assert_eq!(
            rung_flags(Some(&v)),
            RungFlags {
                cf: true,
                cpu: false
            }
        );
        assert_eq!(
            rung_flags(Some(&serde_json::json!("x"))),
            RungFlags::default()
        );
    }
}
