//! The reader's reads besides the ask: their questions, the popular list,
//! one job, a story's panels, the public status, and feedback.

use chrono::Utc;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::services::trevor::{
    AnswerSource, CfBudget, FeedbackRequest, GuardState, JobStatus, JobView, MyQuestion,
    NO_LIMIT_ERA, ONLINE_SECS, Popular, PopularQuestion, QueueDepth, Rung, RungFlags, SpoilerMode,
    StoryPanels, TrevorAnswer, TrevorStatus, WorkerCounts, cf_budget,
};
use crate::app::state::AppState;
use crate::database::queries::trevor as q;

use super::{MY_QUESTIONS_MAX, POPULAR_MAX, POPULAR_MIN_ASKERS, job_view, resolve_horizon, view};

/// `GET /trevor/me/jobs`: the reader's recent questions, newest first.
pub async fn my_questions(
    state: &AppState,
    user_id: Uuid,
    before: Option<chrono::DateTime<Utc>>,
    limit: Option<i64>,
) -> Result<Vec<MyQuestion>, ApiError> {
    let limit = limit.unwrap_or(20).clamp(1, MY_QUESTIONS_MAX);
    let rows = q::my_asks(&state.db, user_id, before, limit).await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            let answer = match (r.answer_id, &r.a_answer) {
                (Some(id), Some(body)) => Some(TrevorAnswer {
                    id,
                    version: r.a_version.clone().unwrap_or_default(),
                    server: r.a_server.clone().unwrap_or_default(),
                    era_pos: r.a_era_pos.unwrap_or(0),
                    rung: Rung::parse_job(r.a_rung.as_deref().unwrap_or("passages")),
                    source: AnswerSource::parse(r.a_source.as_deref().unwrap_or("passages")),
                    answer: body.clone(),
                }),
                _ => None,
            };
            let (status, rung) = if r.hit {
                (JobStatus::Done, Rung::Cache)
            } else {
                (
                    // A job deleted since (or a hit whose answer went) reads
                    // as failed rather than as forever queued.
                    r.status
                        .as_deref()
                        .map_or(JobStatus::Failed, JobStatus::parse),
                    Rung::parse_job(r.rung.as_deref().unwrap_or("passages")),
                )
            };
            MyQuestion {
                job_id: r.job_id,
                question: r.question,
                status,
                rung,
                source: answer.as_ref().map(|a| a.source),
                created_at: r.created_at,
                finished_at: r.finished_at,
                answer,
                spoilers: (r.era_pos == NO_LIMIT_ERA).then_some(SpoilerMode::All),
            }
        })
        .collect())
}

/// `GET /trevor/popular`: the most-asked cached questions inside the
/// reader's era, asked by at least [`POPULAR_MIN_ASKERS`] accounts. A
/// signed-out reader is at era 0 (nothing released): an empty list.
pub async fn popular(
    state: &AppState,
    user_id: Option<Uuid>,
    server: &str,
    limit: Option<i64>,
) -> Result<Popular, ApiError> {
    let Some(user_id) = user_id else {
        return Ok(Popular {
            spoiler_safe_up_to: None,
            items: Vec::new(),
        });
    };
    let limit = limit.unwrap_or(8).clamp(1, POPULAR_MAX);
    let (version, horizon) = resolve_horizon(state, user_id, server, None).await?;
    let Some(version) = version else {
        return Ok(Popular {
            spoiler_safe_up_to: horizon.spoiler_safe,
            items: Vec::new(),
        });
    };
    let items = q::popular(
        &state.db,
        &version,
        server,
        horizon.era_pos,
        POPULAR_MIN_ASKERS,
        limit,
    )
    .await?
    .into_iter()
    .map(|(askers, row, question)| PopularQuestion {
        question,
        askers,
        answer: view(&row),
    })
    .collect();
    Ok(Popular {
        spoiler_safe_up_to: horizon.spoiler_safe,
        items,
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
    let (cf_workers, cpu_workers, tripped) = q::worker_counts(&state.db, ONLINE_SECS).await?;
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
        workers: WorkerCounts {
            cf: cf_workers,
            cpu: cpu_workers,
        },
        guard: if tripped > 0 {
            GuardState::Tripped
        } else {
            GuardState::Ok
        },
        mode: cfg.mode,
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
