//! Bank answers in the answer cache (contract v1.1 section 3a, addition
//! 2026-10-19): the import a publish runs, and the lookup an ask falls back
//! to when no answer holds its exact key.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::{PgPool, Postgres, Transaction};

use super::{ANSWER_COLUMNS, AnswerRow};

/// One cache row of a bank answer: one wording under one era bucket.
#[derive(Debug, Clone)]
pub struct BankRow {
    pub qkey: String,
    pub era_pos: i32,
    pub gate_pos: Option<i32>,
    pub question: String,
    /// The answer as a worker would have sent it (`WorkerAnswer`).
    pub answer: Value,
    /// `{chunkId: sha}` of every cited chunk.
    pub cited_shas: Value,
    /// `bank` or `reanswer`.
    pub origin: &'static str,
    pub answered_at: Option<DateTime<Utc>>,
}

/// What [`import_bank`] wrote.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BankImported {
    pub imported: u64,
    /// Rows whose cited chunk shas are not all the version's.
    pub stale: u64,
    /// Rows a worker's newer answer under the same key kept out.
    pub kept: u64,
}

/// Import a publish's bank rows into `version`, inside the publish
/// transaction and before the carry-over and the active flag move.
///
/// - The version's earlier imported rows go first: the bank a publish sends
///   is the whole bank, so a row it no longer holds leaves the cache.
/// - A row is written only when every cited chunk is in the version's chunk
///   sha map with the same sha (the carry-over's rule), so a changed chunk
///   invalidates it like any cached answer.
/// - The overwrite rule: a row never replaces a worker's answer (live or
///   carried, `origin` NULL) under the same key, in this version or in the
///   active one (whose answers the carry-over is about to copy), unless the
///   row's `answered_at` is later than that answer's `created_at`; a row
///   without `answered_at` never replaces one. When it does replace one, the
///   carry-over then finds the key taken and copies nothing over it.
///
/// `rows` must hold each (qkey, era_pos) once.
pub async fn import_bank(
    tx: &mut Transaction<'_, Postgres>,
    version: &str,
    server: &str,
    rows: &[BankRow],
) -> Result<BankImported, sqlx::Error> {
    sqlx::query(
        "DELETE FROM trevor_answers WHERE version = $1 AND server = $2 AND origin IS NOT NULL",
    )
    .bind(version)
    .bind(server)
    .execute(&mut **tx)
    .await?;
    let shas: Vec<&Value> = rows.iter().map(|r| &r.cited_shas).collect();
    let n = i32::try_from(rows.len()).unwrap_or(i32::MAX);
    let fresh: Vec<i32> = sqlx::query_scalar(
        r"
        SELECT t.i::int FROM UNNEST($3::jsonb[]) WITH ORDINALITY AS t(shas, i)
        WHERE t.shas <> '{}'::jsonb AND NOT EXISTS (
            SELECT 1 FROM jsonb_each_text(t.shas) c
            WHERE NOT EXISTS (
                SELECT 1 FROM trevor_chunk_shas s
                WHERE s.version = $1 AND s.server = $2 AND s.chunk_id = c.key AND s.sha = c.value))
        ",
    )
    .bind(version)
    .bind(server)
    .bind(&shas)
    .fetch_all(&mut **tx)
    .await?;
    let ok: Vec<&BankRow> = fresh
        .iter()
        .filter(|&&i| (1..=n).contains(&i))
        .map(|&i| &rows[(i - 1) as usize])
        .collect();
    let stale = (rows.len() - ok.len()) as u64;
    if ok.is_empty() {
        return Ok(BankImported {
            imported: 0,
            stale,
            kept: 0,
        });
    }
    let done = sqlx::query(
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source,
                                    question, answer, cached, cited_shas, origin, created_at)
        SELECT $1, $2, t.qkey, t.era_pos, t.gate_pos, 'cpu', 'live',
               t.question, t.answer, true, t.shas, t.origin, COALESCE(t.answered_at, now())
        FROM UNNEST($3::text[], $4::int[], $5::int[], $6::text[], $7::jsonb[], $8::jsonb[],
                    $9::text[], $10::timestamptz[])
             AS t(qkey, era_pos, gate_pos, question, answer, shas, origin, answered_at)
        WHERE NOT EXISTS (
            SELECT 1 FROM trevor_answers a
            WHERE a.server = $2 AND a.qkey = t.qkey AND a.era_pos = t.era_pos
              AND a.cached AND a.origin IS NULL
              AND (a.version = $1 OR a.version = (
                    SELECT v.version FROM trevor_versions v WHERE v.server = $2 AND v.active))
              AND (t.answered_at IS NULL OR a.created_at >= t.answered_at))
        ON CONFLICT (version, server, qkey, era_pos) WHERE cached DO UPDATE SET
            gate_pos = EXCLUDED.gate_pos, rung = EXCLUDED.rung, source = EXCLUDED.source,
            question = EXCLUDED.question, answer = EXCLUDED.answer,
            cited_shas = EXCLUDED.cited_shas, origin = EXCLUDED.origin,
            carried_from = NULL, created_at = EXCLUDED.created_at
        ",
    )
    .bind(version)
    .bind(server)
    .bind(ok.iter().map(|r| r.qkey.as_str()).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| r.era_pos).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| r.gate_pos).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| r.question.as_str()).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| &r.answer).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| &r.cited_shas).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| r.origin).collect::<Vec<_>>())
    .bind(ok.iter().map(|r| r.answered_at).collect::<Vec<_>>())
    .execute(&mut **tx)
    .await?;
    let imported = done.rows_affected();
    Ok(BankImported {
        imported,
        stale,
        kept: ok.len() as u64 - imported,
    })
}

/// The bank fallback of a spoiler-safe ask: the question's imported answer
/// of the latest era bucket at or before `era_pos`, counting the hit. A bank
/// answer is cached under the first era its gate admits; every later era
/// admits it too, so it is read here rather than copied into each bucket.
/// The spoiler-mode bucket sits above every era and is read by the exact
/// lookup only.
pub async fn hit_bank(
    pool: &PgPool,
    version: &str,
    server: &str,
    qkey: &str,
    era_pos: i32,
) -> Result<Option<AnswerRow>, sqlx::Error> {
    sqlx::query_as::<_, AnswerRow>(&format!(
        "UPDATE trevor_answers SET hits = hits + 1 WHERE id = ( \
             SELECT id FROM trevor_answers \
             WHERE version = $1 AND server = $2 AND qkey = $3 AND era_pos <= $4 \
               AND cached AND origin IS NOT NULL \
             ORDER BY era_pos DESC LIMIT 1) \
         RETURNING {ANSWER_COLUMNS}"
    ))
    .bind(version)
    .bind(server)
    .bind(qkey)
    .bind(era_pos)
    .fetch_optional(pool)
    .await
}
