//! The answer cache: a hit, the quick look, and an answer by id.

use sqlx::PgPool;
use uuid::Uuid;

use super::{ANSWER_COLUMNS, AnswerRow};

/// A cached answer by its full key, counting the hit.
pub async fn hit_answer(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "UPDATE trevor_answers SET hits = hits + 1 \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos = $4 AND cached \
         RETURNING {ANSWER_COLUMNS}"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_optional(pool)
    .await
}

/// "Quick look": the same question's cached answer from the latest era
/// before `era_pos` (so spoiler-safe for this reader). No hit is counted.
pub async fn quick_look(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "SELECT {ANSWER_COLUMNS} FROM trevor_answers \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos < $4 AND cached \
         ORDER BY era_pos DESC LIMIT 1"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_optional(pool)
    .await
}

pub async fn has_cached(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trevor_answers \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos = $4 AND cached)",
    )
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_one(pool)
    .await
}

pub async fn answer_by_id(pool: &PgPool, id: Uuid) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "SELECT {ANSWER_COLUMNS} FROM trevor_answers WHERE id = $1"
    ))
    .bind(id)
    .fetch_optional(pool)
    .await
}
