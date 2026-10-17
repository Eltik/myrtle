//! The reader side: ask, poll a job, read a story's panels, leave feedback,
//! and the public status.
use std::collections::HashSet;
use std::net::IpAddr;
use std::time::{SystemTime, UNIX_EPOCH};

use chrono::Utc;
use serde_json::Value;
use uuid::Uuid;

use super::{
    ASK_PER_HOUR, Admission, AdmissionInputs, AnswerSource, AskRequest, AskResponse, CfBudget, Era,
    FeedbackRequest, GuardState, Horizon, JobStatus, JobView, MAX_OPEN_PER_USER, MAX_QUEUED, Mode,
    MyQuestion, ONLINE_SECS, Popular, PopularQuestion, QueueDepth, Rung, RungFlags, RungInputs,
    StoryPanels, TrevorAnswer, TrevorStatus, WorkerCounts, admit_rung, bucket_horizon, cf_budget,
    horizon_pos, parse_server, qkey, select_rung, turnstile, validate_question,
};
use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::services::story_progress;
use crate::app::state::AppState;
use crate::core::hypergryph::constants::Server;
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

/// The stories a progress response says were read: the game's read flags
/// plus the site's own marks, minus what the reader marked unread.
fn reads_of(progress: story_progress::StoryProgressResponse) -> Vec<String> {
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
    read.into_iter().collect()
}

/// Whether a progress response carries nothing to read from: no
/// `user_game_story_read` row at all (read or unread) and no site read mark.
fn holds_no_reads(progress: &story_progress::StoryProgressResponse) -> bool {
    progress.game_synced_at.is_none()
        && progress.game_read.is_empty()
        && progress.game_unread.is_empty()
        && progress.progress.as_ref().is_none_or(|doc| {
            doc.0
                .get("read")
                .and_then(Value::as_object)
                .is_none_or(serde_json::Map::is_empty)
        })
}

/// The stories the account has read: the game's read flags plus its own
/// marks, minus what it marked unread.
///
/// An account that has stage progress but no game read rows and no site
/// marks (2,628 of the 2,630 local accounts on 2026-10-17: the rows only
/// exist since a sync after 2026-09-24) would ask at gatePos 0. For it, the
/// first ask in this process re-derives and stores the game's verdict from
/// the stored stage progress ([`story_progress::reverdict`], the same rows a
/// sync writes) and reads again. Once per account per process, whatever it
/// yields; a failed re-derivation is logged and the ask goes on with what
/// was stored. `TREVOR_REVERDICT_ON_ASK=0` skips it.
pub async fn read_stories(
    state: &AppState,
    user_id: Uuid,
    server: &str,
) -> Result<Vec<String>, ApiError> {
    let progress = story_progress::get(state, user_id).await?;
    if !state.trevor.config().reverdict_on_ask || !holds_no_reads(&progress) {
        return Ok(reads_of(progress));
    }
    let Some(game_server) = Server::parse(server) else {
        return Ok(reads_of(progress));
    };
    if !state.trevor.first_reverdict(user_id) {
        return Ok(reads_of(progress));
    }
    let has_stages: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM user_stage_progress WHERE user_id = $1)")
            .bind(user_id)
            .fetch_one(&state.db)
            .await?;
    if !has_stages {
        return Ok(reads_of(progress));
    }
    state
        .trevor
        .reverdicts
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let started = std::time::Instant::now();
    match story_progress::reverdict(state, user_id, game_server).await {
        Ok(fresh) => {
            let read = reads_of(fresh);
            tracing::info!(
                %user_id,
                server,
                read = read.len(),
                ms = started.elapsed().as_millis(),
                "trevor: read verdict re-derived on ask"
            );
            Ok(read)
        }
        Err(e) => {
            tracing::warn!(
                %user_id,
                server,
                error = %e,
                ms = started.elapsed().as_millis(),
                "trevor: re-deriving the read verdict on ask failed; asking with the stored reads"
            );
            Ok(reads_of(progress))
        }
    }
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
    let read = read_stories(state, user_id, server).await?;
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

    fn progress(
        doc: Option<Value>,
        game_read: &[&str],
        game_unread: &[&str],
    ) -> story_progress::StoryProgressResponse {
        let synced = !(game_read.is_empty() && game_unread.is_empty());
        story_progress::StoryProgressResponse {
            progress: doc.map(story_progress::StoryProgressDocument),
            updated_at: None,
            game_read: game_read.iter().map(|s| (*s).to_owned()).collect(),
            game_unread: game_unread.iter().map(|s| (*s).to_owned()).collect(),
            game_archived: 0,
            game_synced_at: synced.then_some(1),
        }
    }

    #[test]
    fn only_an_account_with_nothing_stored_is_re_derived() {
        assert!(holds_no_reads(&progress(None, &[], &[])));
        let empty = serde_json::json!({ "v": 2, "read": {}, "pos": {} });
        assert!(holds_no_reads(&progress(Some(empty), &[], &[])));
        // Unread marks alone are not reads: still re-derived.
        let unread = serde_json::json!({ "v": 2, "read": {}, "unread": { "a": 1 }, "pos": {} });
        assert!(holds_no_reads(&progress(Some(unread), &[], &[])));
        let marked = serde_json::json!({ "v": 2, "read": { "a": 1 }, "pos": {} });
        assert!(!holds_no_reads(&progress(Some(marked), &[], &[])));
        assert!(!holds_no_reads(&progress(None, &["a"], &[])));
        // Rows that are all unread are still a stored verdict.
        assert!(!holds_no_reads(&progress(None, &[], &["a"])));
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
