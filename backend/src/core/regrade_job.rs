//! Periodic regrade job.
//!
//! Recomputes every user's grade on a fixed cadence (default 24h) because
//! stage scoring depends on aggregate user completion, so individual grades
//! drift as the playerbase progresses.
//!
//! The last-run timestamp is persisted to a small JSON file so restarts
//! don't trigger a fresh regrade pass.

use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::grade::calculate::calculate_user_grade;
use crate::database::models::score::UserScore;
use crate::database::queries::score::update_score;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use uuid::Uuid;

const DEFAULT_INTERVAL_SECS: u64 = 86_400; // 24h
const DEFAULT_CONCURRENCY: usize = 4;
const DEFAULT_PAGE_SIZE: i64 = 500;
const DEFAULT_STATE_FILE: &str = "regrade_state.json";
const RETRY_AFTER_ERROR: Duration = Duration::from_hours(1);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct State {
    last_run_at: Option<DateTime<Utc>>,
}

fn state_path() -> PathBuf {
    PathBuf::from(std::env::var("REGRADE_STATE_FILE").unwrap_or_else(|_| DEFAULT_STATE_FILE.into()))
}

fn read_state(path: &PathBuf) -> State {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            tracing::warn!(error = %e, path = %path.display(), "regrade state file unreadable, treating as fresh");
            State::default()
        }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "could not open regrade state file, treating as fresh");
            State::default()
        }
    }
}

/// Atomic write via temp file + rename. Cheap; called at most once per cycle.
fn write_state(path: &PathBuf, state: &State) -> std::io::Result<()> {
    let body = serde_json::to_vec_pretty(state)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

fn delay_until_due(state: &State, interval: Duration) -> Duration {
    match state.last_run_at {
        Some(last) => {
            let elapsed = Utc::now().signed_duration_since(last);
            let elapsed = elapsed.to_std().unwrap_or(Duration::ZERO);
            interval.checked_sub(elapsed).unwrap_or(Duration::ZERO)
        }
        None => Duration::ZERO,
    }
}

struct Cfg {
    interval: Duration,
    concurrency: usize,
    page_size: i64,
    state_path: PathBuf,
}

impl Cfg {
    fn from_env() -> Self {
        let interval_secs = std::env::var("REGRADE_INTERVAL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_INTERVAL_SECS);
        let concurrency = std::env::var("REGRADE_CONCURRENCY")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|n: &usize| *n >= 1)
            .unwrap_or(DEFAULT_CONCURRENCY);
        let page_size = std::env::var("REGRADE_PAGE_SIZE")
            .ok()
            .and_then(|v| v.parse().ok())
            .filter(|n: &i64| *n >= 1)
            .unwrap_or(DEFAULT_PAGE_SIZE);
        Self {
            interval: Duration::from_secs(interval_secs),
            concurrency,
            page_size,
            state_path: state_path(),
        }
    }
}

async fn fetch_page(
    pool: &PgPool,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<(Uuid, String)>, sqlx::Error> {
    match after {
        None => {
            sqlx::query_as("SELECT id, uid FROM users ORDER BY id LIMIT $1")
                .bind(limit)
                .fetch_all(pool)
                .await
        }
        Some(cursor) => {
            sqlx::query_as("SELECT id, uid FROM users WHERE id > $1 ORDER BY id LIMIT $2")
                .bind(cursor)
                .bind(limit)
                .fetch_all(pool)
                .await
        }
    }
}

async fn regrade_one(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<String, sqlx::Error> {
    let grade = calculate_user_grade(pool, user_id, game_data).await?;
    let overall = grade.overall.clone();
    let score = UserScore {
        user_id,
        total_score: grade.total_score,
        operator_score: grade.operator_grade,
        stage_score: grade.stage_grade,
        roguelike_score: grade.roguelike_grade,
        sandbox_score: grade.sandbox_grade,
        medal_score: grade.medal_grade,
        base_score: grade.base_grade,
        skin_score: 0.0,
        grade: Some(grade.overall),
        calculated_at: Utc::now(),
    };
    update_score(pool, &score).await?;
    Ok(overall)
}

/// Runs one full pass over the users table. Returns (successes, failures).
async fn run_pass(state: &AppState, cfg: &Cfg) -> (u64, u64) {
    let game_data = state.game_data.load_full();
    let pool = state.db.clone();
    let sem = Arc::new(Semaphore::new(cfg.concurrency));
    let successes = Arc::new(AtomicU64::new(0));
    let failures = Arc::new(AtomicU64::new(0));
    let started = Instant::now();
    let mut handles: Vec<tokio::task::JoinHandle<()>> = Vec::new();
    let mut cursor: Option<Uuid> = None;
    let mut dispatched: u64 = 0;

    loop {
        let page = match fetch_page(&pool, cursor, cfg.page_size).await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!(error = %e, "regrade: failed to fetch user page; ending pass early");
                break;
            }
        };
        if page.is_empty() {
            break;
        }
        cursor = page.last().map(|(id, _)| *id);

        for (user_id, uid) in page {
            // Wait for an available slot before spawning. Bounds memory + DB usage.
            let Ok(permit) = sem.clone().acquire_owned().await else {
                break;
            };
            let pool = pool.clone();
            let game_data = game_data.clone();
            let successes = successes.clone();
            let failures = failures.clone();

            handles.push(tokio::spawn(async move {
                let _permit = permit;
                match regrade_one(&pool, user_id, &game_data).await {
                    Ok(grade) => {
                        successes.fetch_add(1, Ordering::Relaxed);
                        tracing::debug!(uid = %uid, user_id = %user_id, grade = %grade, "regraded");
                    }
                    Err(e) => {
                        failures.fetch_add(1, Ordering::Relaxed);
                        tracing::warn!(uid = %uid, user_id = %user_id, error = %e, "regrade failed");
                    }
                }
            }));
            dispatched += 1;
        }
    }

    for h in handles {
        let _ = h.await;
    }

    let s = successes.load(Ordering::Relaxed);
    let f = failures.load(Ordering::Relaxed);
    let elapsed = started.elapsed();
    tracing::info!(
        dispatched,
        successes = s,
        failures = f,
        elapsed_s = format!("{:.2}", elapsed.as_secs_f64()),
        "regrade pass complete"
    );
    (s, f)
}

async fn run_loop(state: AppState) {
    let cfg = Cfg::from_env();
    tracing::info!(
        interval_secs = cfg.interval.as_secs(),
        concurrency = cfg.concurrency,
        page_size = cfg.page_size,
        state_file = %cfg.state_path.display(),
        "regrade job started"
    );

    loop {
        let mut persisted = read_state(&cfg.state_path);
        let wait = delay_until_due(&persisted, cfg.interval);
        if wait > Duration::ZERO {
            tracing::debug!(
                wait_secs = wait.as_secs(),
                "regrade: sleeping until next pass"
            );
            tokio::time::sleep(wait).await;
        }

        let (_s, _f) = run_pass(&state, &cfg).await;

        persisted.last_run_at = Some(Utc::now());
        if let Err(e) = write_state(&cfg.state_path, &persisted) {
            tracing::warn!(
                error = %e,
                path = %cfg.state_path.display(),
                "could not persist regrade timestamp; will retry in 1h"
            );
            tokio::time::sleep(RETRY_AFTER_ERROR).await;
        }
    }
}

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        run_loop(state).await;
    });
}
