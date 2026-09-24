//! The database half of the account's game read marks, and the re-derivation.
//!
//! What this module OWNS is `user_game_story_read`: writing the verdict set to
//! it, reading it back, and rebuilding the refresh's payload shape out of it so
//! [`super::verdict::parse_game_story_read`] can be run again with no call to
//! the game server. Nothing here decides what "read" means; that is
//! [`super::verdict`].
//!
//! Scale: 1,365 rows for the measured account against a 1,887-story EN
//! library, written as ONE `UNNEST` insert inside one transaction on one
//! connection. The stage timings on [`StoreOutcome`] are the deliverable, not
//! decoration: see the doc comment there.

use serde_json::Value;
use uuid::Uuid;

use super::verdict::{GameStoryReadSet, parse_game_story_read};
use super::{StoryProgressResponse, get};
use crate::app::{error::ApiError, state::AppState};

/// Re-derive the game verdict from what the database ALREADY holds, with no
/// call to the game server.
///
/// A refresh is the only thing that reads the game's payload, but every input
/// the verdict needs survives it: `user_game_story_read` keeps which stories
/// the flags named (`played`) and which the Archive listed (`archived`, with
/// `rc` and `uts`), and `user_stage_progress.stages` keeps every stage record
/// with `state`, `startTimes` and `completeTimes`. Those are rebuilt into the
/// payload shape the refresh parses and run through the SAME rule, so a rule
/// change reaches an account the moment it presses "Sync now" rather than on
/// its next refresh. An account with neither table row is answered as a plain
/// read. The `cowFirstTs` the refresh copies onto a stage record is what lets
/// the special story stages count here; a store written before that copy
/// loses those (six on the test account) until the next refresh.
pub async fn reverdict(
    state: &AppState,
    user_id: Uuid,
    server: crate::core::hypergryph::constants::Server,
) -> Result<StoryProgressResponse, ApiError> {
    let rows: Vec<StoredRead> = sqlx::query_as(
        "SELECT story_id, reread_count, unlocked_at, played, archived FROM user_game_story_read WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let stages: Option<(Value,)> =
        sqlx::query_as("SELECT stages FROM user_stage_progress WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(&state.db)
            .await?;
    if rows.is_empty() && stages.is_none() {
        return get(state, user_id).await;
    }
    let index = crate::app::services::story::cached_index(state, server).await?;
    let set = crate::app::cpu::offload("story_reverdict", move || {
        let raw = stored_payload(&rows, stages.map(|s| s.0), &index.lookup);
        parse_game_story_read(&raw, &index.by_txt, &index.gates)
    })
    .await?;
    store_game_read(state, user_id, &set)
        .await
        .map_err(|e| ApiError::Internal(anyhow::anyhow!("{e}")))?;
    tracing::info!(
        %user_id,
        rows = set.read.len(),
        read = set.read_count(),
        archived = set.archived(),
        stage_hits = set.stage_hits,
        archive_only_unread = set.archive_only_unread,
        "game story verdict re-derived from stored data"
    );
    get(state, user_id).await
}

/// One stored row, as {@link `stored_payload`} rebuilds it.
#[derive(Debug, sqlx::FromRow)]
pub struct StoredRead {
    pub story_id: String,
    pub reread_count: i32,
    pub unlocked_at: Option<chrono::DateTime<chrono::Utc>>,
    pub played: bool,
    pub archived: bool,
}

/// The refresh's payload shape, rebuilt from stored rows: `played` rows become
/// `status.flags` keyed by the story's script path (through the index's
/// `lookup`, so a row whose story the index no longer knows drops out of the
/// flags half exactly as an unknown path would), `archived` rows become one
/// Archive group carrying `rc` and `uts`, and the stage records go under
/// `dungeon.stages` as stored.
pub fn stored_payload(
    rows: &[StoredRead],
    stages: Option<Value>,
    lookup: &std::collections::HashMap<String, crate::app::services::story::StoryRef>,
) -> Value {
    let mut flags = serde_json::Map::new();
    let mut stories = Vec::new();
    for row in rows {
        if row.played
            && let Some(r) = lookup.get(&row.story_id)
        {
            flags.insert(r.story_txt.clone(), Value::from(1));
        }
        if row.archived {
            let mut story = serde_json::Map::new();
            story.insert("id".into(), Value::from(row.story_id.clone()));
            story.insert("rc".into(), Value::from(row.reread_count));
            if let Some(at) = row.unlocked_at {
                story.insert("uts".into(), Value::from(at.timestamp()));
            }
            stories.push(Value::Object(story));
        }
    }
    serde_json::json!({ "user": {
        "status": { "flags": flags },
        "storyreview": { "groups": { "stored": { "stories": stories } } },
        "dungeon": { "stages": stages.unwrap_or_else(|| Value::Object(serde_json::Map::new())) }
    } })
}

/// Replace the account's imported game read marks with this set.
///
/// The set is the UNION of both sources and every row in it is a story the
/// game says was opened, so every row is stored. That is 1,365 rows for the
/// measured account against a 1,887-story EN library; the first pass stored 2.
///
/// A set that carries neither source is a no-op, not a delete. One payload
/// without them must not erase what a payload with them imported.
///
/// One connection, one transaction, no nested acquire: the DELETE and the
/// INSERT run on the same `tx` and nothing inside this function reaches back
/// to the pool. The caller bounds it with a timeout.
pub async fn store_game_read(
    state: &AppState,
    user_id: Uuid,
    set: &GameStoryReadSet,
) -> Result<StoreOutcome, StoreFailure> {
    if !set.present {
        return Ok(StoreOutcome::default());
    }
    let started = std::time::Instant::now();
    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| stage_failed("begin", started, &e))?;
    let outcome = store_game_read_on(&mut tx, user_id, set).await?;
    let at_commit = std::time::Instant::now();
    tx.commit()
        .await
        .map_err(|e| stage_failed("commit", started, &e))?;
    Ok(StoreOutcome {
        commit_ms: at_commit.elapsed().as_millis(),
        ..outcome
    })
}

/// What the store did, stage by stage.
///
/// The stage times are the deliverable, not decoration: the 2026-09-24 refresh
/// answered 504 with ZERO rows stored, and the only way to tell a slow INSERT
/// from a store that never ran is to have it say which stage it reached.
#[derive(Debug, Default, Clone, Copy)]
pub struct StoreOutcome {
    /// Rows written by the INSERT.
    pub rows: usize,
    /// How long the DELETE took.
    pub delete_ms: u128,
    /// How long the UNNEST INSERT took.
    pub insert_ms: u128,
    /// How long the COMMIT took.
    pub commit_ms: u128,
}

/// A store that did not happen, in the words both the log line and the refresh
/// response need.
///
/// `ApiError::Internal` Displays as "internal error", which is exactly the
/// message that cost the 2026-09-24 pass an afternoon, so the stage travels as
/// a field rather than inside an opaque wrapper.
#[derive(Debug, Clone)]
pub struct StoreFailure {
    /// Which statement did not answer: `begin`, `delete`, `insert`, `commit`.
    pub stage: &'static str,
    /// How long that stage had been running when it failed.
    pub elapsed_ms: u128,
    /// The database's own words.
    pub message: String,
}

impl std::fmt::Display for StoreFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "the game read store failed at stage {} after {} ms: {}",
            self.stage, self.elapsed_ms, self.message
        )
    }
}

impl std::error::Error for StoreFailure {}

fn stage_failed(stage: &'static str, started: std::time::Instant, e: &sqlx::Error) -> StoreFailure {
    StoreFailure {
        stage,
        elapsed_ms: started.elapsed().as_millis(),
        message: e.to_string(),
    }
}

/// The DELETE and the INSERT, on a connection the CALLER owns.
///
/// Split out from [`store_game_read`] so the store can be driven inside a
/// transaction that is rolled back, which is how the reproduction test in
/// `tests/sync_stall_test.rs` runs the real statements against the real
/// database without writing to the user's own table.
///
/// One connection, no nested acquire: nothing here reaches back to the pool.
pub async fn store_game_read_on(
    conn: &mut sqlx::PgConnection,
    user_id: Uuid,
    set: &GameStoryReadSet,
) -> Result<StoreOutcome, StoreFailure> {
    if !set.present {
        return Ok(StoreOutcome::default());
    }
    let ids: Vec<&str> = set.read.iter().map(|r| r.story_id.as_str()).collect();
    let counts: Vec<i32> = set.read.iter().map(|r| r.reread_count).collect();
    let times: Vec<Option<chrono::DateTime<chrono::Utc>>> = set
        .read
        .iter()
        .map(|r| {
            r.unlocked_at
                .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
        })
        .collect();
    let played: Vec<bool> = set.read.iter().map(|r| r.played).collect();
    let archived: Vec<bool> = set.read.iter().map(|r| r.archived).collect();
    let cleared: Vec<bool> = set.read.iter().map(|r| r.cleared).collect();
    let read: Vec<bool> = set.read.iter().map(|r| r.read).collect();

    let started = std::time::Instant::now();
    sqlx::query("DELETE FROM user_game_story_read WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *conn)
        .await
        .map_err(|e| stage_failed("delete", started, &e))?;
    let delete_ms = started.elapsed().as_millis();

    let at_insert = std::time::Instant::now();
    if !ids.is_empty() {
        sqlx::query(
            r"
            INSERT INTO user_game_story_read
                (user_id, story_id, reread_count, unlocked_at, played, archived, cleared, read_in_game, synced_at)
            SELECT $1, s.story_id, s.reread_count, s.unlocked_at, s.played, s.archived, s.cleared, s.read_in_game, now()
            FROM UNNEST($2::text[], $3::int[], $4::timestamptz[], $5::bool[], $6::bool[], $7::bool[], $8::bool[])
                 AS s(story_id, reread_count, unlocked_at, played, archived, cleared, read_in_game)
            ",
        )
        .bind(user_id)
        .bind(&ids)
        .bind(&counts)
        .bind(&times)
        .bind(&played)
        .bind(&archived)
        .bind(&cleared)
        .bind(&read)
        .execute(&mut *conn)
        .await
        .map_err(|e| stage_failed("insert", at_insert, &e))?;
    }
    Ok(StoreOutcome {
        rows: ids.len(),
        delete_ms,
        insert_ms: at_insert.elapsed().as_millis(),
        commit_ms: 0,
    })
}

/// The imported game read marks for one account, split by the verdict: the
/// ids marked read, the ids the Archive unlocked but nothing played, how many
/// of the read ones the Archive lists, and when they were imported.
///
/// No `reread_count` filter: the count is a re-read counter and is 0 for
/// almost every story that has been read.
pub(super) async fn game_read_for(
    state: &AppState,
    user_id: Uuid,
) -> Result<(Vec<String>, Vec<String>, usize, Option<i64>), ApiError> {
    let rows: Vec<(String, bool, bool, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r"
        SELECT story_id, archived, read_in_game, synced_at FROM user_game_story_read
        WHERE user_id = $1
        ORDER BY story_id
        ",
    )
    .bind(user_id)
    .fetch_all(&state.db)
    .await?;
    let synced_at = rows.iter().map(|(_, _, _, at)| at.timestamp()).max();
    let archived = rows.iter().filter(|(_, a, r, _)| *a && *r).count();
    let (read, unread): (Vec<_>, Vec<_>) = rows.into_iter().partition(|(_, _, r, _)| *r);
    Ok((
        read.into_iter().map(|(id, _, _, _)| id).collect(),
        unread.into_iter().map(|(id, _, _, _)| id).collect(),
        archived,
        synced_at,
    ))
}
