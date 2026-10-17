//! Asks (contract v1.1 sections 3c, 5): the ask log behind "My questions",
//! the popular list and the warm-up candidates; and reader feedback.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use super::AnswerRow;

/// One ask to record.
pub struct NewAsk<'a> {
    pub user_id: Uuid,
    pub server: &'a str,
    pub version: Option<&'a str>,
    pub question: &'a str,
    pub normq: &'a str,
    pub qkey: &'a str,
    pub era_pos: i32,
    pub job_id: Option<Uuid>,
    pub deferred_job_id: Option<Uuid>,
    pub answer_id: Option<Uuid>,
}

pub async fn insert_ask(pool: &PgPool, a: &NewAsk<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO trevor_asks (user_id, server, version, question, normq, qkey, era_pos, job_id, \
                                  deferred_job_id, answer_id) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
    )
    .bind(a.user_id)
    .bind(a.server)
    .bind(a.version)
    .bind(a.question)
    .bind(a.normq)
    .bind(a.qkey)
    .bind(a.era_pos)
    .bind(a.job_id)
    .bind(a.deferred_job_id)
    .bind(a.answer_id)
    .execute(pool)
    .await?;
    Ok(())
}

/// One row of "My questions": the ask, its primary job (the deferred one
/// when there is one) and that job's (or the cache hit's) answer.
#[derive(Debug, Clone, FromRow)]
pub struct MyAskRow {
    pub question: String,
    pub created_at: DateTime<Utc>,
    pub job_id: Option<Uuid>,
    pub status: Option<String>,
    pub rung: Option<String>,
    pub finished_at: Option<DateTime<Utc>>,
    pub hit: bool,
    pub answer_id: Option<Uuid>,
    pub a_version: Option<String>,
    pub a_server: Option<String>,
    pub a_era_pos: Option<i32>,
    pub a_rung: Option<String>,
    pub a_source: Option<String>,
    pub a_answer: Option<Value>,
}

pub async fn my_asks(
    pool: &PgPool,
    user_id: Uuid,
    before: Option<DateTime<Utc>>,
    limit: i64,
) -> Result<Vec<MyAskRow>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT k.question, k.created_at, j.id AS job_id, j.status, j.rung, j.finished_at,
               (j.id IS NULL AND k.answer_id IS NOT NULL) AS hit,
               a.id AS answer_id, a.version AS a_version, a.server AS a_server,
               a.era_pos AS a_era_pos, a.rung AS a_rung, a.source AS a_source, a.answer AS a_answer
        FROM trevor_asks k
        LEFT JOIN trevor_jobs j ON j.id = COALESCE(k.deferred_job_id, k.job_id)
        LEFT JOIN trevor_answers a ON a.id = CASE
            WHEN j.id IS NULL THEN k.answer_id
            WHEN j.status = 'done' THEN j.answer_id
        END
        WHERE k.user_id = $1 AND ($2::timestamptz IS NULL OR k.created_at < $2)
        ORDER BY k.created_at DESC
        LIMIT $3
        ",
    )
    .bind(user_id)
    .bind(before)
    .bind(limit)
    .fetch_all(pool)
    .await
}

/// The most-asked cached questions of a version that a reader at `era_pos`
/// may see: per question, the cached answer of the latest era at or before
/// `era_pos`, listed when at least `min_askers` distinct accounts asked it
/// on the server (in any version, so a publish does not reset the list),
/// most askers (then hits) first.
pub async fn popular(
    pool: &PgPool,
    version: &str,
    server: &str,
    era_pos: i32,
    min_askers: i64,
    limit: i64,
) -> Result<Vec<(i64, AnswerRow, String)>, sqlx::Error> {
    #[derive(FromRow)]
    struct Row {
        askers: i64,
        question: String,
        #[sqlx(flatten)]
        answer: AnswerRow,
    }
    let rows: Vec<Row> = sqlx::query_as(
        r"
        WITH askers AS (
            SELECT qkey, count(DISTINCT user_id) AS askers
            FROM trevor_asks WHERE server = $2
            GROUP BY qkey HAVING count(DISTINCT user_id) >= $4
        ), best AS (
            SELECT DISTINCT ON (a.qkey) a.*
            FROM trevor_answers a JOIN askers k ON k.qkey = a.qkey
            WHERE a.version = $1 AND a.server = $2 AND a.cached AND a.era_pos <= $3
            ORDER BY a.qkey, a.era_pos DESC
        )
        SELECT k.askers, b.question, b.id, b.version, b.server, b.era_pos, b.rung, b.source,
               b.answer, b.created_at
        FROM best b JOIN askers k ON k.qkey = b.qkey
        ORDER BY k.askers DESC, b.hits DESC, b.qkey
        LIMIT $5
        ",
    )
    .bind(version)
    .bind(server)
    .bind(era_pos)
    .bind(min_askers)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| (r.askers, r.answer, r.question))
        .collect())
}

/// The warm-up list (contract v1.1 section 3c): the questions most asked in
/// the last `days` days, per (normQ, era), that `version` holds no cached
/// answer for (so neither carried over nor answered since): (latest asked
/// text, normQ, eraPos, asks).
pub async fn warm_candidates(
    pool: &PgPool,
    version: &str,
    server: &str,
    days: i32,
    limit: i64,
) -> Result<Vec<(String, String, i32, i64)>, sqlx::Error> {
    sqlx::query_as(
        r"
        SELECT (array_agg(k.question ORDER BY k.created_at DESC))[1] AS question,
               (array_agg(k.normq ORDER BY k.created_at DESC))[1] AS normq,
               k.era_pos, count(*) AS asks
        FROM trevor_asks k
        WHERE k.server = $2 AND k.created_at > now() - make_interval(days => $3)
          AND NOT EXISTS (SELECT 1 FROM trevor_answers a
                          WHERE a.version = $1 AND a.server = $2 AND a.qkey = k.qkey
                            AND a.era_pos = k.era_pos AND a.cached)
        GROUP BY k.qkey, k.era_pos
        ORDER BY count(*) DESC, k.qkey, k.era_pos
        LIMIT $4
        ",
    )
    .bind(version)
    .bind(server)
    .bind(days)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn upsert_feedback(
    pool: &PgPool,
    answer_id: Uuid,
    user_id: Uuid,
    vote: i16,
    note: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO trevor_feedback (answer_id, user_id, vote, note)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (answer_id, user_id) DO UPDATE SET
            vote = EXCLUDED.vote, note = EXCLUDED.note, updated_at = now()
        ",
    )
    .bind(answer_id)
    .bind(user_id)
    .bind(vote)
    .bind(note)
    .execute(pool)
    .await?;
    Ok(())
}
