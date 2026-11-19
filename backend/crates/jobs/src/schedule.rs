use std::time::Duration;

use chrono::{DateTime, Utc};
use myrtle_db::{Conn, Db, DbError};
use myrtle_types::Server;
use serde_json::Value;

use crate::model::{JobKind, JobSpec};
use crate::queue::{QueueError, enqueue_in};

const UPSERT: &str = "
INSERT INTO job_schedules (kind, scope, every_secs, offset_secs, enabled, next_run_at)
VALUES ($1, $2, $3, $4, $5, now() + make_interval(secs => $4))
ON CONFLICT (kind, scope) DO UPDATE
   SET every_secs = EXCLUDED.every_secs,
       offset_secs = EXCLUDED.offset_secs,
       enabled = EXCLUDED.enabled";

const DUE: &str = "
SELECT kind, scope, every_secs, offset_secs, now()
  FROM job_schedules
 WHERE enabled AND next_run_at <= now()
   FOR UPDATE SKIP LOCKED";

const ADVANCE: &str = "
UPDATE job_schedules
   SET next_run_at = $3, last_enqueued_at = now()
 WHERE kind = $1 AND scope = $2";

const MEMORY: &str = "SELECT memory FROM job_schedules WHERE kind = $1 AND scope = $2";

const REMEMBER: &str = "UPDATE job_schedules SET memory = $3 WHERE kind = $1 AND scope = $2";

#[derive(Debug, Clone, Copy)]
pub struct Schedule {
    pub kind: JobKind,
    pub server: Option<Server>,
    pub every: Duration,
    pub enabled: bool,
}

impl Schedule {
    #[must_use]
    pub const fn new(kind: JobKind, every: Duration) -> Self {
        Self {
            kind,
            server: None,
            every,
            enabled: true,
        }
    }
}

pub const DEFAULTS: [Schedule; 11] = [
    Schedule::new(JobKind::TrendingRecompute, Duration::from_mins(15)),
    Schedule::new(JobKind::MaintenancePoll, Duration::from_mins(5)),
    Schedule::new(JobKind::MusicStatDedupeSweep, Duration::from_hours(1)),
    Schedule::new(JobKind::GachaDetailRefresh, Duration::from_hours(6)),
    Schedule::new(JobKind::EventShopRefresh, Duration::from_hours(6)),
    Schedule::new(JobKind::SirenSync, Duration::from_hours(6)),
    Schedule::new(JobKind::LeaderboardSnapshot, Duration::from_hours(24)),
    Schedule::new(JobKind::OperatorOwnershipStats, Duration::from_hours(24)),
    Schedule::new(JobKind::MedalOwnershipStats, Duration::from_hours(24)),
    Schedule::new(JobKind::RegradeAll, Duration::from_hours(24)),
    Schedule {
        enabled: false,
        ..Schedule::new(JobKind::DpsUpstreamWatch, Duration::from_hours(1))
    },
];

pub async fn sync(
    conn: &mut Conn,
    schedules: &[Schedule],
    stagger: Duration,
) -> Result<(), QueueError> {
    for s in schedules {
        sqlx::query(UPSERT)
            .bind(s.kind.as_str())
            .bind(scope(s.server))
            .bind(s.every.as_secs().clamp(1, i32::MAX as u64) as i32)
            .bind(offset_for(s.kind, stagger).as_secs() as i32)
            .bind(s.enabled)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

pub async fn enqueue_due(db: &Db) -> Result<u64, QueueError> {
    let mut tx = db.begin().await?;
    let due: Vec<(String, String, i32, i32, DateTime<Utc>)> =
        sqlx::query_as(DUE).fetch_all(&mut *tx).await?;
    let mut enqueued = 0;
    for (kind, scope, every, offset, now) in due {
        let Some(job) = JobKind::parse(&kind) else {
            continue; // a newer build's schedule
        };
        let spec = JobSpec {
            server: Server::parse(&scope),
            dedupe_key: Some(format!("schedule:{scope}")),
            ..JobSpec::new(job)
        };
        enqueue_in(&mut tx, &spec).await?;
        let next = next_run(
            Duration::from_secs(every as u64),
            Duration::from_secs(offset as u64),
            now,
        );
        sqlx::query(ADVANCE)
            .bind(&kind)
            .bind(&scope)
            .bind(next)
            .execute(&mut *tx)
            .await?;
        enqueued += 1;
    }
    tx.commit().await?;
    Ok(enqueued)
}

pub async fn memory(
    conn: &mut Conn,
    kind: JobKind,
    server: Option<Server>,
) -> Result<Value, QueueError> {
    Ok(sqlx::query_scalar(MEMORY)
        .bind(kind.as_str())
        .bind(scope(server))
        .fetch_one(conn)
        .await?)
}

pub async fn remember(
    conn: &mut Conn,
    kind: JobKind,
    server: Option<Server>,
    memory: &Value,
) -> Result<(), QueueError> {
    let kept = sqlx::query(REMEMBER)
        .bind(kind.as_str())
        .bind(scope(server))
        .bind(memory)
        .execute(conn)
        .await?;
    if kept.rows_affected() == 0 {
        return Err(DbError::NotFound.into());
    }
    Ok(())
}

#[must_use]
pub fn next_run(every: Duration, offset: Duration, now: DateTime<Utc>) -> DateTime<Utc> {
    let every = every.as_secs().max(1) as i64;
    let offset = offset.as_secs() as i64 % every;
    let n = (now.timestamp() - offset).div_euclid(every) + 1;
    DateTime::from_timestamp(n * every + offset, 0).unwrap_or(now)
}

#[must_use]
pub fn offset_for(kind: JobKind, window: Duration) -> Duration {
    let window = window.as_secs();
    if window == 0 {
        return Duration::ZERO;
    }
    let hash = kind
        .as_str()
        .bytes()
        .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
        });
    Duration::from_secs(hash % window)
}

fn scope(server: Option<Server>) -> &'static str {
    server.map_or("", Server::as_str)
}
