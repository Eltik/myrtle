//! Corpus versions: the active one, its eras and gate positions, the
//! publish (with the answer carry-over), and a version's panels.

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;

/// The active corpus version of a server, if one was published.
pub async fn active_version(pool: &PgPool, server: &str) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT version FROM trevor_versions WHERE server = $1 AND active")
        .bind(server)
        .fetch_optional(pool)
        .await
}

/// The active version of a server and its manifest's `eras` (not the whole
/// manifest: `storyOrder` alone is ~40 KB).
pub async fn active_version_eras(
    pool: &PgPool,
    server: &str,
) -> Result<Option<(String, Option<Value>)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT version, manifest -> 'eras' FROM trevor_versions WHERE server = $1 AND active",
    )
    .bind(server)
    .fetch_optional(pool)
    .await
}

/// The highest gatePos among `story_ids` in one version's panels; `None`
/// when none of them has one.
pub async fn max_gate_pos(
    pool: &PgPool,
    version: &str,
    server: &str,
    story_ids: &[String],
) -> Result<Option<i32>, sqlx::Error> {
    if story_ids.is_empty() {
        return Ok(None);
    }
    sqlx::query_scalar(
        "SELECT max(gate_pos) FROM trevor_panels \
         WHERE version = $1 AND server = $2 AND story_id = ANY($3)",
    )
    .bind(version)
    .bind(server)
    .bind(story_ids)
    .fetch_one(pool)
    .await
}

/// One story's gatePos in a version: `None` when the story has no panel,
/// `Some(None)` when its panel carries no position.
pub async fn story_gate_pos(
    pool: &PgPool,
    version: &str,
    server: &str,
    story_id: &str,
) -> Result<Option<Option<i32>>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT gate_pos FROM trevor_panels WHERE version = $1 AND server = $2 AND story_id = $3",
    )
    .bind(version)
    .bind(server)
    .bind(story_id)
    .fetch_optional(pool)
    .await
}

/// Whether a cached answer exists under the full key (no hit counted).
/// Whether a version of a server was published.
pub async fn version_known(
    pool: &PgPool,
    version: &str,
    server: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM trevor_versions WHERE version = $1 AND server = $2)",
    )
    .bind(version)
    .bind(server)
    .fetch_one(pool)
    .await
}

/// One panel as published: the story, its gatePos (the panel's `position`)
/// and the JSON.
pub struct PanelRow {
    pub story_id: String,
    pub gate_pos: Option<i32>,
    pub panels: Value,
}

/// Insert (or replace) a version's row and its panels; optionally make it the
/// active one of its server. A call without a manifest, gate report or build
/// time keeps the stored one, so `activate: false` chunks need not resend
/// them. Activation fills any panel without a position from the manifest's
/// `storyOrder`. One transaction, so readers never see a version active
/// without its panels.
#[allow(clippy::too_many_arguments)]
pub async fn publish_version(
    pool: &PgPool,
    version: &str,
    server: &str,
    built_at: Option<DateTime<Utc>>,
    gate_report: Option<&Value>,
    manifest: Option<&Value>,
    panels: &[PanelRow],
    activate: bool,
    chunk_shas: &[(String, String)],
    carry_over: bool,
) -> Result<u64, sqlx::Error> {
    publish_version_with_bank(
        pool,
        version,
        server,
        built_at,
        gate_report,
        manifest,
        panels,
        activate,
        chunk_shas,
        carry_over,
        &[],
    )
    .await
    .map(|p| p.carried)
}

/// What [`publish_version_with_bank`] did.
#[derive(Debug, Clone)]
pub struct Published {
    pub carried: u64,
    /// The bank import's counts; `Err` with the database's message when it
    /// failed and was rolled back (the publish went on without it).
    pub bank: Result<super::BankImported, String>,
}

/// [`publish_version`] plus the bank import of an activating call
/// ([`super::import_bank`]), run in a savepoint before the carry-over, so a
/// failed import rolls back alone and never fails the publish.
#[allow(clippy::too_many_arguments)]
pub async fn publish_version_with_bank(
    pool: &PgPool,
    version: &str,
    server: &str,
    built_at: Option<DateTime<Utc>>,
    gate_report: Option<&Value>,
    manifest: Option<&Value>,
    panels: &[PanelRow],
    activate: bool,
    chunk_shas: &[(String, String)],
    carry_over: bool,
    bank: &[super::BankRow],
) -> Result<Published, sqlx::Error> {
    let mut tx = pool.begin().await?;
    sqlx::query(
        r"
        INSERT INTO trevor_versions (version, server, built_at, gate_report, manifest)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (version, server) DO UPDATE SET
            built_at = COALESCE(EXCLUDED.built_at, trevor_versions.built_at),
            gate_report = COALESCE(EXCLUDED.gate_report, trevor_versions.gate_report),
            manifest = COALESCE(EXCLUDED.manifest, trevor_versions.manifest),
            published_at = now()
        ",
    )
    .bind(version)
    .bind(server)
    .bind(built_at)
    .bind(gate_report)
    .bind(manifest)
    .execute(&mut *tx)
    .await?;
    if !panels.is_empty() {
        let ids: Vec<&str> = panels.iter().map(|p| p.story_id.as_str()).collect();
        let positions: Vec<Option<i32>> = panels.iter().map(|p| p.gate_pos).collect();
        let bodies: Vec<&Value> = panels.iter().map(|p| &p.panels).collect();
        sqlx::query(
            r"
            INSERT INTO trevor_panels (version, server, story_id, gate_pos, panels)
            SELECT $1, $2, t.story_id, t.gate_pos, t.panels
            FROM UNNEST($3::text[], $4::int[], $5::jsonb[]) AS t(story_id, gate_pos, panels)
            ON CONFLICT (version, server, story_id) DO UPDATE SET
                gate_pos = EXCLUDED.gate_pos, panels = EXCLUDED.panels
            ",
        )
        .bind(version)
        .bind(server)
        .bind(ids)
        .bind(positions)
        .bind(bodies)
        .execute(&mut *tx)
        .await?;
    }
    if !chunk_shas.is_empty() {
        let ids: Vec<&str> = chunk_shas.iter().map(|(id, _)| id.as_str()).collect();
        let shas: Vec<&str> = chunk_shas.iter().map(|(_, sha)| sha.as_str()).collect();
        sqlx::query(
            r"
            INSERT INTO trevor_chunk_shas (version, server, chunk_id, sha)
            SELECT $1, $2, t.chunk_id, t.sha FROM UNNEST($3::text[], $4::text[]) AS t(chunk_id, sha)
            ON CONFLICT (version, server, chunk_id) DO UPDATE SET sha = EXCLUDED.sha
            ",
        )
        .bind(version)
        .bind(server)
        .bind(ids)
        .bind(shas)
        .execute(&mut *tx)
        .await?;
    }
    let mut bank_done = Ok(super::BankImported::default());
    if activate && !bank.is_empty() {
        let mut sp = sqlx::Connection::begin(&mut *tx).await?;
        match super::import_bank(&mut sp, version, server, bank).await {
            Ok(n) => {
                sp.commit().await?;
                bank_done = Ok(n);
            }
            Err(e) => {
                sp.rollback().await?;
                bank_done = Err(e.to_string());
            }
        }
    }
    let mut carried = 0;
    if activate && carry_over {
        carried = carry_answers(&mut tx, version, server).await?;
    }
    if activate {
        sqlx::query(
            r"
            UPDATE trevor_panels p SET gate_pos = o.pos::int
            FROM trevor_versions v,
                 jsonb_array_elements_text(v.manifest -> 'storyOrder') WITH ORDINALITY AS o(story_id, pos)
            WHERE v.version = $1 AND v.server = $2
              AND p.version = v.version AND p.server = v.server
              AND p.story_id = o.story_id AND p.gate_pos IS NULL
            ",
        )
        .bind(version)
        .bind(server)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE trevor_versions SET active = false WHERE server = $1 AND active")
            .bind(server)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE trevor_versions SET active = true WHERE version = $1 AND server = $2")
            .bind(version)
            .bind(server)
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;
    Ok(Published {
        carried,
        bank: bank_done,
    })
}

/// The cache carry-over (contract v1.1 section 3b), run while activating
/// `version`, before the active flag moves: copy each cached answer of the
/// server's active version whose every cited chunk is in `version`'s chunk
/// sha map with the same sha, and whose era position is 0, the internal
/// no-limit era, or a position of `version`'s eras. An answer with no
/// `cited_shas` (a citation without a sha, or none) never moves, nor does an
/// imported bank row (`origin` set; the publish imports the bank again). The copy
/// keeps its hits and names its source in `carried_from`; an answer `version`
/// already holds under the key wins.
async fn carry_answers(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    version: &str,
    server: &str,
) -> Result<u64, sqlx::Error> {
    let done = sqlx::query(
        r"
        INSERT INTO trevor_answers (version, server, qkey, era_pos, gate_pos, rung, source,
                                    question, answer, cached, cited_shas, carried_from, hits)
        SELECT $1, a.server, a.qkey, a.era_pos, a.gate_pos, a.rung, a.source,
               a.question, a.answer, true, a.cited_shas, a.version, a.hits
        FROM trevor_answers a
        JOIN trevor_versions pv ON pv.version = a.version AND pv.server = a.server AND pv.active
        WHERE a.server = $2 AND a.version <> $1 AND a.cached AND a.origin IS NULL
          AND a.cited_shas IS NOT NULL AND a.cited_shas <> '{}'::jsonb
          AND (a.era_pos = 0 OR a.era_pos = 2147483647 OR a.era_pos IN (
                SELECT (e ->> 'position')::int
                FROM trevor_versions nv, jsonb_array_elements(nv.manifest -> 'eras') e
                WHERE nv.version = $1 AND nv.server = $2))
          AND NOT EXISTS (
                SELECT 1 FROM jsonb_each_text(a.cited_shas) c
                WHERE NOT EXISTS (
                    SELECT 1 FROM trevor_chunk_shas s
                    WHERE s.version = $1 AND s.server = $2
                      AND s.chunk_id = c.key AND s.sha = c.value))
        ON CONFLICT (version, server, qkey, era_pos) WHERE cached DO NOTHING
        ",
    )
    .bind(version)
    .bind(server)
    .execute(&mut **tx)
    .await?;
    Ok(done.rows_affected())
}

/// One story's panels under the active version of a server.
pub async fn active_panels(
    pool: &PgPool,
    server: &str,
    story_id: &str,
) -> Result<Option<(String, Value)>, sqlx::Error> {
    sqlx::query_as::<_, (String, Value)>(
        r"
        SELECT p.version, p.panels FROM trevor_panels p
        JOIN trevor_versions v ON v.version = p.version AND v.server = p.server AND v.active
        WHERE p.server = $1 AND p.story_id = $2
        ",
    )
    .bind(server)
    .bind(story_id)
    .fetch_optional(pool)
    .await
}
