//! The reader side: ask, poll a job, read a story's panels, leave feedback,
//! and the public status.
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use super::{
    ASK_PER_HOUR, AnswerSource, AskRequest, AskResponse, CfBudget, Era, FeedbackRequest, Horizon,
    JobStatus, JobView, MAX_OPEN_PER_USER, MAX_QUEUED, ONLINE_SECS, QueueDepth, Rung, RungFlags,
    RungInputs, StoryPanels, TrevorAnswer, TrevorStatus, bucket_horizon, cf_budget, horizon_pos,
    parse_server, qkey, select_rung, turnstile, validate_question,
};
use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::services::story_progress;
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

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

/// The stories the account has read: the game's read flags plus its own
/// marks, minus what it marked unread.
pub async fn read_stories(state: &AppState, user_id: Uuid) -> Result<Vec<String>, ApiError> {
    let progress = story_progress::get(state, user_id).await?;
    let mut read: HashSet<String> = progress.game_read.into_iter().collect();
    if let Some(doc) = &progress.progress {
        if let Some(marks) = doc.0.get("read").and_then(Value::as_object) {
            read.extend(marks.keys().cloned());
        }
        if let Some(unread) = doc.0.get("unread").and_then(Value::as_object) {
            for id in unread.keys() {
                read.remove(id);
            }
        }
    }
    Ok(read.into_iter().collect())
}

/// The active version and its eras; an unparsable `eras` reads as none.
async fn active_eras(
    state: &AppState,
    server: &str,
) -> Result<Option<(String, Vec<Era>)>, ApiError> {
    Ok(q::active_version_eras(&state.db, server)
        .await?
        .map(|(version, eras)| {
            let eras = eras
                .and_then(|v| serde_json::from_value::<Vec<Era>>(v).ok())
                .unwrap_or_default();
            (version, eras)
        }))
}

/// The ask's horizon (contract section 1): the furthest gatePos the account
/// has read in the active version, lowered by `horizonStory`, bucketed to its
/// era. No active version means nothing is released: eraPos 0.
pub async fn resolve_horizon(
    state: &AppState,
    user_id: Uuid,
    server: &str,
    horizon_story: Option<&str>,
) -> Result<(Option<String>, Horizon), ApiError> {
    let buckets = state.trevor.config().era_buckets;
    let Some((version, eras)) = active_eras(state, server).await? else {
        return Ok((None, bucket_horizon(&[], 0, buckets)));
    };
    let read = read_stories(state, user_id).await?;
    let profile_pos = q::max_gate_pos(&state.db, &version, server, &read)
        .await?
        .unwrap_or(0);
    let story_pos = match horizon_story.map(str::trim).filter(|s| !s.is_empty()) {
        None => None,
        Some(story) => {
            if story.len() > 200 {
                return Err(ApiError::BadRequest("horizonStory is too long".into()));
            }
            match q::story_gate_pos(&state.db, &version, server, story).await? {
                Some(pos) => Some(pos.unwrap_or(0)),
                None => {
                    return Err(ApiError::BadRequest(format!(
                        "horizonStory {story} is not in the active Trevor version"
                    )));
                }
            }
        }
    };
    let pos = horizon_pos(profile_pos, story_pos);
    Ok((Some(version), bucket_horizon(&eras, pos, buckets)))
}

fn rung_flags(v: Option<&Value>) -> RungFlags {
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
                (*n >= super::ETA_MIN_SAMPLES)
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

/// `POST /trevor/ask`.
pub async fn ask(
    state: &AppState,
    user_id: Uuid,
    remote_ip: Option<IpAddr>,
    req: AskRequest,
) -> Result<AskResponse, ApiError> {
    let normalized = validate_question(&req.question)?;
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

    if let Some(version) = &version
        && let Some(row) = q::hit_answer(&state.db, version, server, &key, horizon.era_pos).await?
    {
        return Ok(AskResponse {
            job_id: None,
            status: JobStatus::Done,
            rung: Rung::Cache,
            spoiler_safe_up_to: horizon.spoiler_safe,
            answer: Some(view(&row)),
            eta_secs: None,
            stream_url: None,
        });
    }

    // Someone already asked this under the same horizon: join their job
    // instead of queueing a twin.
    let job =
        if let Some(open) = q::open_job_for_key(&state.db, server, &key, horizon.era_pos).await? {
            if open.user_id != Some(user_id) {
                state.trevor.add_joiner(open.id, user_id);
            }
            open
        } else {
            let open = q::open_jobs_of_user(&state.db, user_id).await?;
            let queued = q::queued_jobs(&state.db, server).await?;
            admit(0, open, queued)?;
            let (rungs, neurons_today, cpu_depth) = q::rung_inputs(&state.db).await?;
            let rung = select_rung(
                &RungInputs {
                    rungs: rung_flags(rungs.as_ref()),
                    neurons_today,
                    cpu_depth,
                },
                &state.trevor.config(),
            );
            let inserted = q::insert_job(
                &state.db,
                &q::NewJob {
                    user_id: Some(user_id),
                    question: req.question.trim(),
                    qkey: &key,
                    horizon_pos: Some(horizon.era_pos),
                    server,
                    corpus_version: version.as_deref(),
                    rung: rung.as_str(),
                    profile: rung.profile().as_str(),
                },
            )
            .await?;
            if let Some(job) = inserted {
                state.trevor.job_enqueued();
                job
            } else {
                // An identical ask enqueued between the lookup and the
                // insert (the unique open-key index refused this one): join it.
                let open = q::open_job_for_key(&state.db, server, &key, horizon.era_pos)
                    .await?
                    .ok_or_else(|| {
                        ApiError::Conflict("the identical question just closed; ask again".into())
                    })?;
                if open.user_id != Some(user_id) {
                    state.trevor.add_joiner(open.id, user_id);
                }
                open
            }
        };
    let jv = job_view(state, &job).await?;
    Ok(AskResponse {
        job_id: Some(job.id),
        status: jv.status,
        rung: jv.rung,
        spoiler_safe_up_to: horizon.spoiler_safe,
        answer: jv.answer,
        eta_secs: jv.eta_secs,
        stream_url: Some(stream_url(job.id)),
    })
}

/// May this user read this job's stream? Its asker, or someone who joined
/// it as a twin in this process.
pub fn may_stream(state: &AppState, job: &q::JobRow, user_id: Uuid) -> bool {
    job.user_id == Some(user_id) || state.trevor.is_joiner(job.id, user_id)
}

/// `GET /trevor/jobs/{id}`. Any signed-in user may poll a job id they hold:
/// ids are unguessable, and askers who joined a twin job poll its id.
///
/// Owner (or twin joiner) only, like the stream; `TREVOR_JOB_OWNER_ONLY=0`
/// lets any signed-in user poll any id again.
pub async fn job(state: &AppState, user_id: Uuid, job_id: Uuid) -> Result<JobView, ApiError> {
    let job = q::job_by_id(&state.db, job_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    if state.trevor.config().job_owner_only && !may_stream(state, &job, user_id) {
        return Err(ApiError::Forbidden);
    }
    job_view(state, &job).await
}

/// `GET /trevor/panels/{storyId}`: 404 until a version with this story is
/// published and active. Read from Postgres on every call; the edge caches
/// it (see the route's `Cache-Control`).
pub async fn panels(
    state: &AppState,
    server: &'static str,
    story_id: &str,
) -> Result<StoryPanels, ApiError> {
    if story_id.is_empty() || story_id.len() > 200 {
        return Err(ApiError::BadRequest("storyId is empty or too long".into()));
    }
    let (version, panels) = q::active_panels(&state.db, server, story_id)
        .await?
        .ok_or_else(|| ApiError::NotFoundMessage(format!("no Trevor panels for {story_id}")))?;
    Ok(StoryPanels {
        story_id: story_id.to_owned(),
        server: server.to_owned(),
        version,
        panels,
    })
}

/// `GET /trevor/status`.
pub async fn status(state: &AppState, server: &str) -> Result<TrevorStatus, ApiError> {
    let (online, cf, cpu) = q::workers_online(&state.db, ONLINE_SECS).await?;
    let (cf_depth, cpu_depth) = q::model_queue_depth(&state.db).await?;
    let neurons_today = q::neurons_today(&state.db).await?;
    let cfg = state.trevor.config();
    Ok(TrevorStatus {
        online: online > 0,
        rungs: RungFlags { cf, cpu },
        active_version: q::active_version(&state.db, server).await?,
        queue_depth: QueueDepth {
            cf: cf_depth,
            cpu: cpu_depth,
        },
        cf_budget: if cfg.cf_daily_neurons <= 0 {
            CfBudget::Spent
        } else {
            cf_budget(neurons_today, &cfg)
        },
    })
}

/// `POST /trevor/answers/{id}/feedback`.
pub async fn feedback(
    state: &AppState,
    user_id: Uuid,
    answer_id: Uuid,
    req: FeedbackRequest,
) -> Result<(), ApiError> {
    if !matches!(req.vote, -1..=1) {
        return Err(ApiError::BadRequest("vote must be -1, 0 or 1".into()));
    }
    let note = req.note.as_deref().map(str::trim).filter(|n| !n.is_empty());
    if note.is_some_and(|n| n.chars().count() > 2000) {
        return Err(ApiError::BadRequest(
            "note is longer than 2000 characters".into(),
        ));
    }
    if q::answer_by_id(&state.db, answer_id).await?.is_none() {
        return Err(ApiError::NotFound);
    }
    q::upsert_feedback(&state.db, answer_id, user_id, req.vote, note).await?;
    Ok(())
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
