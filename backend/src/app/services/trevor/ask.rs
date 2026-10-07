//! The reader side: ask, poll a job, read a story's panels, leave feedback.
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;
use uuid::Uuid;

use super::{
    ASK_PER_HOUR, AskRequest, AskResponse, FeedbackRequest, JobStatus, JobView, MAX_OPEN_PER_USER,
    MAX_QUEUED, QueuedJob, StoryPanels, TrevorAnswer, eta_secs, horizon_key, parse_server, qkey,
    validate_question, worker::Heartbeat,
};
use crate::app::cache::{CachedJson, cached_json_detached, keys::CacheKey};
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
        horizon: (!row.horizon.is_empty()).then(|| row.horizon.clone()),
        answer: row.answer.clone(),
    }
}

/// The last heartbeat of a server's workers, if one is fresh.
pub async fn heartbeat(state: &AppState, server: &str) -> Option<Heartbeat> {
    state.cache.get(&CacheKey::TrevorWorker { server }).await
}

/// The user's furthest-read story: among the stories the account has read
/// (its own marks and the game's), the one in the latest-released group, the
/// highest `sort` within it. `None` when nothing is read or the index is not
/// built for that server.
pub async fn furthest_read(
    state: &AppState,
    user_id: Uuid,
    server: &str,
) -> Result<Option<String>, ApiError> {
    let progress = story_progress::get(state, user_id).await?;
    let mut read: std::collections::HashSet<String> = progress.game_read.into_iter().collect();
    if let Some(doc) = &progress.progress
        && let Some(marks) = doc.0.get("read").and_then(Value::as_object)
    {
        read.extend(marks.keys().cloned());
    }
    if let Some(doc) = &progress.progress
        && let Some(unread) = doc.0.get("unread").and_then(Value::as_object)
    {
        for id in unread.keys() {
            read.remove(id);
        }
    }
    if read.is_empty() {
        return Ok(None);
    }
    let Ok(game_server) = crate::app::services::auth::parse_server(server) else {
        return Ok(None);
    };
    let index = crate::app::services::story::get_story_index(state, game_server).await?;
    Ok(index
        .groups
        .iter()
        .flat_map(|g| g.stories.iter().map(move |s| (g.start_time, s.sort, &s.id)))
        .filter(|(_, _, id)| read.contains(*id))
        .max()
        .map(|(_, _, id)| id.clone()))
}

/// `POST /trevor/ask`.
pub async fn ask(
    state: &AppState,
    user_id: Uuid,
    req: AskRequest,
) -> Result<AskResponse, ApiError> {
    let normalized = validate_question(&req.question)?;
    let server = parse_server(req.server.as_deref())?;
    let key = qkey(&normalized);

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

    let horizon = if req.no_horizon {
        None
    } else if let Some(h) = req.horizon.as_deref().filter(|h| !h.trim().is_empty()) {
        Some(h.trim().to_owned())
    } else {
        furthest_read(state, user_id, server)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(error = %e, "trevor: furthest-read lookup failed");
                None
            })
            .or_else(|| req.story_id.clone())
    };
    let horizon = horizon_key(horizon.as_deref()).to_owned();

    if let Some(version) = q::active_version(&state.db, server).await? {
        let cache_key = CacheKey::TrevorAnswer {
            server,
            version: &version,
            qkey: &key,
            horizon: &horizon,
        };
        if let Some(hit) = state.cache.get::<TrevorAnswer>(&cache_key).await {
            return Ok(AskResponse::Answered(hit));
        }
        if let Some(row) = q::hit_answer(&state.db, &version, server, &key, &horizon).await? {
            let answer = view(&row);
            state.cache.set(&cache_key, &answer).await;
            return Ok(AskResponse::Answered(answer));
        }
    }

    let beat = heartbeat(state, server).await;
    let slots = beat.as_ref().map_or(1, |b| i64::from(b.slots));
    // Someone already asked this: join their job instead of queueing a twin.
    let job = if let Some(open) = q::open_job_for_key(&state.db, server, &key, &horizon).await? {
        open
    } else {
        let open = q::open_jobs_of_user(&state.db, user_id).await?;
        let queued = q::queued_jobs(&state.db, server).await?;
        admit(0, open, queued)?;
        q::insert_job(
            &state.db,
            Some(user_id),
            req.question.trim(),
            &key,
            &horizon,
            server,
        )
        .await?
    };
    let position = if JobStatus::parse(&job.status) == JobStatus::Queued {
        q::queue_position(&state.db, &job).await?
    } else {
        0
    };
    Ok(AskResponse::Queued(QueuedJob {
        job_id: job.id,
        position,
        eta: eta_secs(position.max(1), slots),
        worker_online: beat.is_some(),
    }))
}

/// `GET /trevor/jobs/{id}`. Any signed-in user may poll a job id they hold:
/// ids are unguessable, and askers who joined a twin job poll its id.
pub async fn job(state: &AppState, job_id: Uuid) -> Result<JobView, ApiError> {
    let job = q::job_by_id(&state.db, job_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let status = JobStatus::parse(&job.status);
    let beat = heartbeat(state, &job.server).await;
    let slots = beat.as_ref().map_or(1, |b| i64::from(b.slots));
    let position = if status == JobStatus::Queued {
        q::queue_position(&state.db, &job).await?
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
        position,
        eta: match status {
            JobStatus::Queued => eta_secs(position, slots),
            JobStatus::Running => super::ANSWER_SECS,
            JobStatus::Done | JobStatus::Failed => 0,
        },
        worker_online: beat.is_some(),
        answer,
        error: job.error,
    })
}

/// `GET /trevor/story/{storyId}/panels`: 404 until a version with this story
/// is published and active.
pub async fn panels(
    state: &AppState,
    server: &'static str,
    story_id: &str,
) -> Result<CachedJson, ApiError> {
    if story_id.is_empty() || story_id.len() > 200 {
        return Err(ApiError::BadRequest("storyId is empty or too long".into()));
    }
    let key = CacheKey::TrevorPanels { server, story_id };
    let db = state.db.clone();
    let id = story_id.to_owned();
    cached_json_detached(state, &key, move || async move {
        let (version, panels) = q::active_panels(&db, server, &id)
            .await?
            .ok_or_else(|| ApiError::NotFoundMessage(format!("no Trevor panels for {id}")))?;
        let body = StoryPanels {
            story_id: id,
            server: server.to_owned(),
            version,
            panels,
        };
        serde_json::to_string(&body).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
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
}
