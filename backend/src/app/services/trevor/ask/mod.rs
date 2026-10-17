//! The reader side: ask, poll a job, read a story's panels, leave feedback,
//! and the public status.
//!
//! - this module: the ask itself (allowance, cache, admission, the twin join).
//! - [`progress`]: the reader's spoiler horizon from their reading progress.
//! - [`eta`]: rung ETAs and a job's view.
//! - [`reads`]: the reader's lists, a job, panels, status and feedback.

pub mod eta;
pub mod progress;
pub mod reads;

pub use eta::*;
pub use progress::*;
pub use reads::*;

use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::services::trevor::{
    ASK_PER_HOUR, Admission, AdmissionInputs, AnswerSource, AskRequest, AskResponse, Horizon,
    JobStatus, MAX_OPEN_PER_USER, MAX_QUEUED, Mode, ONLINE_SECS, Rung, RungInputs, TrevorAnswer,
    admit_rung, parse_server, qkey, select_rung, turnstile, validate_question,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use eta::rung_flags;

/// Why an ask is refused before it costs anything, or `Ok` to go on.
///
/// Pure so the limits are testable: the hourly allowance first (it counts
/// every ask, cached or not), then the per-user open-job cap, then the global
/// queue cap. The two caps only matter for an ask that will enqueue.
pub fn admit(asked_this_hour: u64, open_of_user: i64, queued: i64) -> Result<(), ApiError> {
    if asked_this_hour > ASK_PER_HOUR {
        return Err(ApiError::RateLimited);
    }
    if open_of_user >= MAX_OPEN_PER_USER {
        return Err(ApiError::Conflict(format!(
            "at most {MAX_OPEN_PER_USER} questions may wait at once; wait for one to finish"
        )));
    }
    if queued >= MAX_QUEUED {
        return Err(ApiError::ServiceUnavailable);
    }
    Ok(())
}

fn hour_index() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() / 3600)
}

/// What a reader sees when `TREVOR_MODE=off` and the answer is not cached.
pub const RESTING: &str = "Trevor is resting";

/// "My questions" lists at most this many per page.
pub const MY_QUESTIONS_MAX: i64 = 50;

/// The popular list: at most this many, and only questions this many
/// distinct accounts asked.
pub const POPULAR_MAX: i64 = 50;

pub const POPULAR_MIN_ASKERS: i64 = 2;

fn view(row: &q::AnswerRow) -> TrevorAnswer {
    TrevorAnswer {
        id: row.id,
        version: row.version.clone(),
        server: row.server.clone(),
        era_pos: row.era_pos,
        rung: Rung::parse_job(&row.rung),
        source: AnswerSource::parse(&row.source),
        answer: row.answer.clone(),
    }
}

/// The SSE route of a job, as the reader requests it.
pub fn stream_url(job_id: Uuid) -> String {
    format!("/api/trevor/jobs/{job_id}/stream")
}

/// Record an ask for "My questions", the popular list and the warm-up.
/// Never fails the ask: a failed insert is logged.
async fn log_ask(state: &AppState, ask: q::NewAsk<'_>) {
    if !state.trevor.config().ask_log {
        return;
    }
    if let Err(e) = q::insert_ask(&state.db, &ask).await {
        tracing::warn!(error = %e, "trevor: recording the ask failed");
    }
}

/// Join an open job (normal or deferred) as a twin, so its stream and poll
/// open to this asker.
fn join(state: &AppState, job: &q::JobRow, user_id: Uuid) {
    if job.user_id != Some(user_id) {
        state.trevor.add_joiner(job.id, user_id);
    }
}

/// Insert a job, or join the identical one that won the race for the
/// unique open-key index.
async fn insert_or_join(
    state: &AppState,
    user_id: Uuid,
    new: &q::NewJob<'_>,
) -> Result<q::JobRow, ApiError> {
    if let Some(job) = q::insert_job(&state.db, new).await? {
        state.trevor.job_enqueued();
        return Ok(job);
    }
    let horizon = new.horizon_pos.unwrap_or(0);
    let open = q::open_job_for_key(&state.db, new.server, new.qkey, horizon, new.deferred)
        .await?
        .ok_or_else(|| {
            ApiError::Conflict("the identical question just closed; ask again".into())
        })?;
    join(state, &open, user_id);
    Ok(open)
}

/// `POST /trevor/ask`.
///
/// Contract v1.1: the cache first (also under `TREVOR_MODE=off`, which
/// otherwise answers 503 "Trevor is resting"); then a twin's open job; then
/// admission control ([`admit_rung`]): a model rung with room takes the job,
/// and when every offered model rung is full the ask gets a quick passages
/// job (claimed first by any worker, no model) plus, under
/// `TREVOR_GLOBAL_QUEUE_MAX`, a deferred job for the full answer. A
/// passages-only answer comes with a "Quick look" when an earlier era cached
/// the same question. `TREVOR_MODE=passages` makes every new job a passages
/// one, with no deferred job. `TREVOR_ADMISSION=0` restores the v1 path.
pub async fn ask(
    state: &AppState,
    user_id: Uuid,
    remote_ip: Option<IpAddr>,
    req: AskRequest,
) -> Result<AskResponse, ApiError> {
    let cfg = state.trevor.config();
    let normalized = validate_question(&req.question, cfg.normq)?;
    let server = parse_server(req.server.as_deref())?;
    let key = qkey(&normalized);
    turnstile::check(state, req.turnstile_token.as_deref(), remote_ip).await?;

    let user = user_id.to_string();
    // Counted before the cache lookup: a cached answer is cheap, but 30 an
    // hour is the product limit, not a cost limit. Redis down admits.
    let asked = state
        .cache
        .incr_window(&CacheKey::TrevorAskQuota {
            user_id: &user,
            hour: hour_index(),
        })
        .await
        .unwrap_or(0);
    admit(asked, 0, 0)?;

    let (version, horizon) =
        resolve_horizon(state, user_id, server, req.horizon_story.as_deref()).await?;
    let question = req.question.trim();
    let ask_row = |job_id, deferred_job_id, answer_id| q::NewAsk {
        user_id,
        server,
        version: version.as_deref(),
        question,
        normq: &normalized,
        qkey: &key,
        era_pos: horizon.era_pos,
        job_id,
        deferred_job_id,
        answer_id,
    };

    if let Some(version) = &version
        && let Some(row) = q::hit_answer(&state.db, version, server, &key, horizon.era_pos).await?
    {
        log_ask(state, ask_row(None, None, Some(row.id))).await;
        return Ok(AskResponse {
            job_id: None,
            status: JobStatus::Done,
            rung: Rung::Cache,
            spoiler_safe_up_to: horizon.spoiler_safe,
            answer: Some(view(&row)),
            eta_secs: None,
            stream_url: None,
            deferred_job_id: None,
            quick_look: None,
        });
    }
    if cfg.mode == Mode::Off {
        return Err(ApiError::ServiceUnavailableMessage(RESTING.into()));
    }

    let new_job = |rung: Rung, deferred: bool| q::NewJob {
        user_id: Some(user_id),
        question,
        qkey: &key,
        horizon_pos: Some(horizon.era_pos),
        server,
        corpus_version: version.as_deref(),
        rung: rung.as_str(),
        profile: rung.profile_with(cfg.cpu_profile).as_str(),
        deferred,
        kind: "ask",
    };

    // Someone already asked this under the same horizon: join their job
    // (and its deferred twin) instead of queueing another.
    let (job, deferred) = if let Some(open) =
        q::open_job_for_key(&state.db, server, &key, horizon.era_pos, false).await?
    {
        join(state, &open, user_id);
        let deferred = if cfg.admission {
            q::open_job_for_key(&state.db, server, &key, horizon.era_pos, true).await?
        } else {
            None
        };
        if let Some(d) = &deferred {
            join(state, d, user_id);
        }
        (open, deferred)
    } else if !cfg.admission {
        let open = q::open_jobs_of_user(&state.db, user_id, false).await?;
        let queued = q::queued_jobs(&state.db, server).await?;
        admit(0, open, queued)?;
        let (rungs, neurons_today, cpu_depth) = q::rung_inputs(&state.db).await?;
        let rung = select_rung(
            &RungInputs {
                rungs: rung_flags(rungs.as_ref()),
                neurons_today,
                cpu_depth,
            },
            &cfg,
        );
        (
            insert_or_join(state, user_id, &new_job(rung, false)).await?,
            None,
        )
    } else {
        let open = q::open_jobs_of_user(&state.db, user_id, true).await?;
        admit(0, open, 0)?;
        // Admission reads the depths and inserts under one advisory lock, so
        // a burst cannot overfill a rung ([`q::admission_lock`]).
        let mut tx = state.db.begin().await?;
        q::admission_lock(&mut *tx).await?;
        // A twin admitted while this ask waited for the lock: join it rather
        // than read it as load (measured: without this, 24 of 50 keys in a
        // 2,000-ask burst also got a needless deferred job).
        if let Some(open) =
            q::open_job_for_key(&mut *tx, server, &key, horizon.era_pos, false).await?
        {
            tx.commit().await?;
            join(state, &open, user_id);
            let deferred =
                q::open_job_for_key(&state.db, server, &key, horizon.era_pos, true).await?;
            if let Some(d) = &deferred {
                join(state, d, user_id);
            }
            return finish_ask(
                state,
                user_id,
                &key,
                server,
                &version,
                horizon,
                question,
                &normalized,
                open,
                deferred,
            )
            .await;
        }
        let adm = q::admission_inputs(&mut *tx, ONLINE_SECS).await?;
        let verdict = if cfg.mode == Mode::Passages {
            Admission::Now(Rung::Passages)
        } else {
            admit_rung(
                &AdmissionInputs {
                    rung: RungInputs {
                        rungs: rung_flags(adm.rungs.as_ref()),
                        neurons_today: adm.neurons_today,
                        cpu_depth: adm.cpu_depth,
                    },
                    cf_depth: adm.cf_depth,
                    cpu_workers: adm.cpu_workers,
                    open_total: adm.open_total,
                },
                &cfg,
            )
        };
        let (first, second) = match verdict {
            Admission::Now(rung) => (new_job(rung, false), None),
            Admission::Overflow { deferred } => {
                // A user holds at most MAX_OPEN_PER_USER deferred jobs too.
                let deferred = match deferred {
                    Some(rung)
                        if q::deferred_jobs_of_user(&mut *tx, user_id).await?
                            < MAX_OPEN_PER_USER =>
                    {
                        Some(new_job(rung, true))
                    }
                    _ => None,
                };
                (new_job(Rung::Passages, false), deferred)
            }
        };
        let first_row = q::insert_job(&mut *tx, &first).await?;
        let second_row = match &second {
            Some(d) => Some(q::insert_job(&mut *tx, d).await?),
            None => None,
        };
        tx.commit().await?;
        if first_row.is_some() || second_row.as_ref().is_some_and(Option::is_some) {
            state.trevor.job_enqueued();
        }
        // An identical ask won the race for the open-key index: join it.
        let job = match first_row {
            Some(j) => j,
            None => insert_or_join(state, user_id, &first).await?,
        };
        let deferred = match (second_row, &second) {
            (Some(Some(j)), _) => Some(j),
            (Some(None), Some(d)) => Some(insert_or_join(state, user_id, d).await?),
            _ => None,
        };
        (job, deferred)
    };
    finish_ask(
        state,
        user_id,
        &key,
        server,
        &version,
        horizon,
        question,
        &normalized,
        job,
        deferred,
    )
    .await
}

/// The response of an ask that got a job: its view, the deferred job, the
/// Quick look of a passages-only answer; and the ask's row.
#[allow(clippy::too_many_arguments)]
async fn finish_ask(
    state: &AppState,
    user_id: Uuid,
    key: &str,
    server: &str,
    version: &Option<String>,
    horizon: Horizon,
    question: &str,
    normalized: &str,
    job: q::JobRow,
    deferred: Option<q::JobRow>,
) -> Result<AskResponse, ApiError> {
    let cfg = state.trevor.config();
    let jv = job_view(state, &job).await?;
    let quick_look = match version {
        Some(v) if cfg.admission && jv.rung == Rung::Passages && jv.answer.is_none() => {
            q::quick_look(&state.db, v, server, key, horizon.era_pos)
                .await?
                .as_ref()
                .map(view)
        }
        _ => None,
    };
    log_ask(
        state,
        q::NewAsk {
            user_id,
            server,
            version: version.as_deref(),
            question,
            normq: normalized,
            qkey: key,
            era_pos: horizon.era_pos,
            job_id: Some(job.id),
            deferred_job_id: deferred.as_ref().map(|d| d.id),
            answer_id: None,
        },
    )
    .await;
    Ok(AskResponse {
        job_id: Some(job.id),
        status: jv.status,
        rung: jv.rung,
        spoiler_safe_up_to: horizon.spoiler_safe,
        answer: jv.answer,
        eta_secs: jv.eta_secs,
        stream_url: Some(stream_url(job.id)),
        deferred_job_id: deferred.map(|d| d.id),
        quick_look,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admit_allows_up_to_the_hourly_allowance() {
        assert!(admit(ASK_PER_HOUR, 0, 0).is_ok());
        assert!(matches!(
            admit(ASK_PER_HOUR + 1, 0, 0),
            Err(ApiError::RateLimited)
        ));
    }

    #[test]
    fn admit_caps_open_jobs_per_user() {
        assert!(admit(1, MAX_OPEN_PER_USER - 1, 0).is_ok());
        assert!(matches!(
            admit(1, MAX_OPEN_PER_USER, 0),
            Err(ApiError::Conflict(_))
        ));
    }

    #[test]
    fn admit_caps_the_global_queue() {
        assert!(admit(1, 0, MAX_QUEUED - 1).is_ok());
        assert!(matches!(
            admit(1, 0, MAX_QUEUED),
            Err(ApiError::ServiceUnavailable)
        ));
    }
}
