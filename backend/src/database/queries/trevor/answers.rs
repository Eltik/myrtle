//! The answer cache: a hit, the quick look, and an answer by id.

use sqlx::PgPool;
use uuid::Uuid;

use super::{ANSWER_COLUMNS, AnswerRow};

/// A cached answer by its full key, counting the hit. `with_bank` false
/// (`TREVOR_BANK_IMPORT=0`) skips an imported bank row.
pub async fn hit_answer(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
    with_bank: bool,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "UPDATE trevor_answers SET hits = hits + 1 \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos = $4 AND cached \
           AND (origin IS NULL OR $5) \
         RETURNING {ANSWER_COLUMNS}"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .bind(with_bank)
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

/// Whether an ask under this key would be served from the cache (no hit
/// counted): an answer under the exact key, or for a spoiler-safe era an
/// imported bank row at or below it (the ask's bank fallback).
pub async fn has_cached(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trevor_answers \
         WHERE version = $1 AND server = $2 AND qkey = $3 AND cached \
           AND (era_pos = $4 OR (origin IS NOT NULL AND era_pos <= $4 AND $4 < 2147483647)))",
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
