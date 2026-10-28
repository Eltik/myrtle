//! `user_lookback_snapshots` reads and the one write, plus the roster reads a
//! snapshot is built from.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;
use uuid::Uuid;

use crate::core::lookback::{KEEP_EVERY_DAYS, Period, SummaryExtras};
use crate::database::models::lookback::{BuildRow, LookbackSnapshot, LookbackSummary, StoredBuild};

/// `secs` (Unix seconds) as a timestamp; one out of chrono's range, which
/// no stored row has, reads as the epoch.
fn at(secs: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(secs, 0).unwrap_or_default()
}

/// Whether `user_id` already has the snapshot of the period opening at
/// `period_start` (Unix seconds): the whole cost of a sync inside a period
/// that has one, one primary-key probe.
pub async fn period_taken(pool: &PgPool, user_id: Uuid, period_start: i64) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM user_lookback_snapshots \
         WHERE user_id = $1 AND period_start = $2)",
    )
    .bind(user_id)
    .bind(at(period_start))
    .fetch_one(pool)
    .await
}

/// Every owned operator with its masteries and unlocked modules, by id.
pub async fn build_rows(pool: &PgPool, user_id: Uuid) -> Result<Vec<BuildRow>, sqlx::Error> {
    sqlx::query_as::<_, BuildRow>(
        "SELECT o.operator_id::text AS operator_id, o.elite, o.level, o.potential, o.skill_level, \
             COALESCE((SELECT array_agg(s.specialize_level ORDER BY s.skill_index) \
                       FROM user_operator_skills s \
                       WHERE s.user_id = o.user_id AND s.operator_id = o.operator_id), '{}') AS masteries, \
             COALESCE((SELECT array_agg(m.module_id::text ORDER BY m.module_id) \
                       FROM user_operator_modules m \
                       WHERE m.user_id = o.user_id AND m.operator_id = o.operator_id AND NOT m.locked), '{}') AS module_ids, \
             COALESCE((SELECT array_agg(m.module_level ORDER BY m.module_id) \
                       FROM user_operator_modules m \
                       WHERE m.user_id = o.user_id AND m.operator_id = o.operator_id AND NOT m.locked), '{}') AS module_levels \
         FROM user_operators o WHERE o.user_id = $1 ORDER BY o.operator_id",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// The non-roster numbers of a summary, and the account's `server_id` (`None`
/// when the account does not exist).
pub async fn summary_extras(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<(i16, SummaryExtras)>, sqlx::Error> {
    // Non-default skins as `v_user_profile.non_default_skin_count` counts them.
    let row: Option<(i16, i64, Option<f64>, Option<String>)> = sqlx::query_as(
        "SELECT u.server_id, \
             (SELECT COUNT(*) FROM user_skins sk WHERE sk.user_id = u.id AND sk.skin_id LIKE '%@%'), \
             sc.total_score, sc.grade::text \
         FROM users u LEFT JOIN user_scores sc ON sc.user_id = u.id WHERE u.id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(server_id, skins, total_score, grade)| {
        (
            server_id,
            SummaryExtras {
                skins: u32::try_from(skins).unwrap_or(0),
                total_score,
                grade,
            },
        )
    }))
}

/// Store the snapshot of `period` unless the account has it already (a sync
/// racing this one), then prune the account's old rows. Returns whether a row
/// was written.
pub async fn insert(
    pool: &PgPool,
    user_id: Uuid,
    period: &Period,
    summary: &LookbackSummary,
    build: &StoredBuild,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let written = sqlx::query(
        "INSERT INTO user_lookback_snapshots \
             (user_id, period_start, activity_id, activity_name, summary, build) \
         VALUES ($1, $2, $3, $4, $5, $6) \
         ON CONFLICT (user_id, period_start) DO NOTHING",
    )
    .bind(user_id)
    .bind(at(period.start))
    .bind(period.activity_id.as_deref())
    .bind(period.activity_name.as_deref())
    .bind(Json(summary))
    .bind(Json(build))
    .execute(&mut *tx)
    .await?
    .rows_affected()
        == 1;
    if written {
        prune(&mut tx, user_id).await?;
    }
    tx.commit().await?;
    Ok(written)
}

/// Drop `user_id`'s rows older than [`KEEP_EVERY_DAYS`] that are not the first
/// of their UTC calendar month. Returns the rows deleted.
async fn prune(conn: &mut sqlx::PgConnection, user_id: Uuid) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query(
        "DELETE FROM user_lookback_snapshots s \
         WHERE s.user_id = $1 AND s.period_start < now() - make_interval(days => $2) \
           AND EXISTS (SELECT 1 FROM user_lookback_snapshots e \
                       WHERE e.user_id = s.user_id AND e.period_start < s.period_start \
                         AND date_trunc('month', e.period_start AT TIME ZONE 'UTC') \
                           = date_trunc('month', s.period_start AT TIME ZONE 'UTC'))",
    )
    .bind(user_id)
    .bind(KEEP_EVERY_DAYS as i32)
    .execute(conn)
    .await?
    .rows_affected())
}

#[derive(sqlx::FromRow)]
struct SnapshotRow {
    period_start: DateTime<Utc>,
    taken_at: DateTime<Utc>,
    activity_id: Option<String>,
    activity_name: Option<String>,
    summary: Json<LookbackSummary>,
}

/// Every snapshot of `user_id`, oldest first, without builds.
pub async fn list(pool: &PgPool, user_id: Uuid) -> Result<Vec<LookbackSnapshot>, sqlx::Error> {
    let rows = sqlx::query_as::<_, SnapshotRow>(
        "SELECT period_start, taken_at, activity_id::text AS activity_id, activity_name, summary \
         FROM user_lookback_snapshots WHERE user_id = $1 ORDER BY period_start",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| LookbackSnapshot {
            id: r.period_start.timestamp(),
            period_start: r.period_start,
            taken_at: r.taken_at,
            activity_id: r.activity_id,
            activity_name: r.activity_name,
            summary: r.summary.0,
        })
        .collect())
}

/// One snapshot's sync time, counts and stored build, by its key.
pub async fn get(
    pool: &PgPool,
    user_id: Uuid,
    period_start: i64,
) -> Result<Option<(DateTime<Utc>, LookbackSummary, StoredBuild)>, sqlx::Error> {
    let row: Option<(DateTime<Utc>, Json<LookbackSummary>, Json<StoredBuild>)> = sqlx::query_as(
        "SELECT taken_at, summary, build FROM user_lookback_snapshots \
         WHERE user_id = $1 AND period_start = $2",
    )
    .bind(user_id)
    .bind(at(period_start))
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(taken_at, summary, build)| (taken_at, summary.0, build.0)))
}

/// The obtain time of every owned operator, Unix seconds.
pub async fn obtained_times(pool: &PgPool, user_id: Uuid) -> Result<Vec<i64>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COALESCE(obtained_at, 0) FROM user_operators WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}
