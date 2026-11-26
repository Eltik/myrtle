use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::DateTime;
use myrtle_db::Conn;
use myrtle_db::repos::story::{self as repo, GameStoryMark, StoredMarkRow};
use myrtle_engine_story::StoryRef;
use myrtle_engine_story::verdict::{GameStoryReadSet, parse_game_story_read};
use myrtle_types::{Server, UserId};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::{AppContext, AppError, Subject};

pub const MAX_DOCUMENT_BYTES: usize = 512 * 1024;
pub const STORY_INDEX_BUDGET: Duration = Duration::from_secs(30);
pub const GAME_READ_STORE_BUDGET: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ts_rs::TS)]
#[ts(export)]
#[serde(transparent)]
pub struct StoryProgressDocument(
    #[ts(
        type = "{ v: 1 | 2; read: Record<string, number>; unread?: Record<string, number>; pos: Record<string, { halt: number; total: number; ts: number; choices: Record<number, string>; reach?: number }>; last?: string }"
    )]
    pub Value,
);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ts_rs::TS)]
#[ts(export)]
#[serde(rename_all = "camelCase")]
pub struct StoryProgressResponse {
    pub progress: Option<StoryProgressDocument>,
    #[ts(type = "number | null")]
    pub updated_at: Option<i64>,
    pub game_read: Vec<String>,
    pub game_unread: Vec<String>,
    pub game_archived: u32,
    #[ts(type = "number | null")]
    pub game_synced_at: Option<i64>,
}

fn signed_in(subject: &Subject) -> Result<UserId, AppError> {
    subject.user_id().ok_or(AppError::Unauthorized)
}

fn non_negative(v: &Value) -> bool {
    v.as_f64().is_some_and(|n| n.is_finite() && n >= 0.0)
}

fn bad(message: impl Into<String>) -> AppError {
    AppError::BadRequest(message.into())
}

pub fn validate(doc: &StoryProgressDocument) -> Result<(), AppError> {
    let Some(root) = doc.0.as_object() else {
        return Err(bad("progress must be an object"));
    };
    if !matches!(root.get("v").and_then(Value::as_i64), Some(1 | 2)) {
        return Err(bad("progress.v must be 1 or 2"));
    }
    let Some(read) = root.get("read").and_then(Value::as_object) else {
        return Err(bad("progress.read must be an object"));
    };
    if read.keys().any(String::is_empty) {
        return Err(bad("progress.read has an empty story id"));
    }
    if let Some(unread) = root.get("unread") {
        let Some(unread) = unread.as_object() else {
            return Err(bad("progress.unread must be an object"));
        };
        if unread.keys().any(String::is_empty) {
            return Err(bad("progress.unread has an empty story id"));
        }
        if !unread.values().all(non_negative) {
            return Err(bad(
                "progress.unread values must be finite numbers at or above zero",
            ));
        }
    }
    let Some(pos) = root.get("pos").and_then(Value::as_object) else {
        return Err(bad("progress.pos must be an object"));
    };
    for (id, entry) in pos {
        if id.is_empty() {
            return Err(bad("progress.pos has an empty story id"));
        }
        let Some(entry) = entry.as_object() else {
            return Err(bad(format!("progress.pos.{id} must be an object")));
        };
        for key in ["halt", "total", "ts"] {
            if !entry.get(key).is_some_and(non_negative) {
                return Err(bad(format!(
                    "progress.pos.{id}.{key} must be a finite number at or above zero"
                )));
            }
        }
        if entry.get("reach").is_some_and(|r| !non_negative(r)) {
            return Err(bad(format!(
                "progress.pos.{id}.reach must be a finite number at or above zero"
            )));
        }
    }
    if root.get("last").is_some_and(|last| !last.is_string()) {
        return Err(bad("progress.last must be a string"));
    }
    let bytes = serde_json::to_vec(&doc.0)
        .map_err(|e| bad(format!("progress is not serializable: {e}")))?
        .len(); // the compact encoding is what the column stores
    if bytes > MAX_DOCUMENT_BYTES {
        return Err(bad(format!(
            "progress is {bytes} bytes, over the {MAX_DOCUMENT_BYTES} byte limit"
        )));
    }
    Ok(())
}

async fn respond(
    conn: &mut Conn,
    user: UserId,
    document: Option<repo::StoryDocumentRow>,
) -> Result<StoryProgressResponse, AppError> {
    let rows = repo::mark_verdicts(conn, user).await?;
    let game_synced_at = rows.iter().map(|r| r.synced_at.timestamp()).max();
    let archived = rows.iter().filter(|r| r.archived && r.read_in_game).count();
    let (read, unread): (Vec<_>, Vec<_>) = rows.into_iter().partition(|r| r.read_in_game);
    Ok(StoryProgressResponse {
        updated_at: document.as_ref().map(|d| d.updated_at.timestamp()),
        progress: document.map(|d| StoryProgressDocument(d.progress)),
        game_read: read.into_iter().map(|r| r.story_id).collect(),
        game_unread: unread.into_iter().map(|r| r.story_id).collect(),
        game_archived: u32::try_from(archived).unwrap_or(u32::MAX),
        game_synced_at,
    })
}

pub async fn get(ctx: &AppContext, subject: &Subject) -> Result<StoryProgressResponse, AppError> {
    let user = signed_in(subject)?;
    let mut conn = ctx.db.conn().await?;
    let document = repo::document(&mut conn, user).await?;
    respond(&mut conn, user, document).await
}

pub async fn put(
    ctx: &AppContext,
    subject: &Subject,
    doc: StoryProgressDocument,
) -> Result<StoryProgressResponse, AppError> {
    let user = signed_in(subject)?;
    validate(&doc)?;
    let mut conn = ctx.db.conn().await?;
    let stored = repo::put_document(&mut conn, user, &doc.0).await?;
    respond(&mut conn, user, Some(stored)).await
}

fn data_server(ctx: &AppContext, server: Server) -> Server {
    if ctx.snapshots.get(server).is_some() {
        server
    } else {
        ctx.snapshots.default_server()
    }
}

pub async fn import(
    ctx: &AppContext,
    subject: &Subject,
) -> Result<StoryProgressResponse, AppError> {
    let user = signed_in(subject)?;
    let server = subject.game_server()?;
    let mut conn = ctx.db.conn().await?;
    let rows = repo::stored_marks(&mut conn, user).await?;
    let stages = repo::stage_blob(&mut conn, user).await?;
    if !rows.is_empty() || stages.is_some() {
        drop(conn);
        let snapshot = super::snapshot(ctx, data_server(ctx, server)).await?;
        let set = ctx
            .cpu
            .offload("story_reverdict", move || {
                let library = &snapshot.story()?.library;
                let raw = stored_payload(&rows, stages, &library.lookup);
                Ok::<_, AppError>(parse_game_story_read(&raw, &library.by_txt, &library.gates))
            })
            .await??;
        store_game_read(ctx, user, &set)
            .await
            .map_err(AppError::internal)?;
        tracing::info!(
            user = %user.0,
            rows = set.read.len(),
            read = set.read_count(),
            archived = set.archived(),
            stage_hits = set.stage_hits,
            archive_only_unread = set.archive_only_unread,
            "game story verdict re-derived from stored data"
        );
        conn = ctx.db.conn().await?;
    }
    let document = repo::document(&mut conn, user).await?;
    respond(&mut conn, user, document).await
}

#[must_use]
pub fn stored_payload(
    rows: &[StoredMarkRow],
    stages: Option<Value>,
    lookup: &HashMap<String, StoryRef>,
) -> Value {
    let mut flags = Map::new();
    let mut stories = Vec::new();
    for row in rows {
        if row.played
            && let Some(r) = lookup.get(&row.story_id)
        {
            flags.insert(r.story_txt.clone(), Value::from(1));
        }
        if row.archived {
            let mut story = Map::new();
            story.insert("id".into(), Value::from(row.story_id.clone()));
            story.insert("rc".into(), Value::from(row.reread_count));
            if let Some(at) = row.unlocked_at {
                story.insert("uts".into(), Value::from(at.timestamp()));
            }
            stories.push(Value::Object(story));
        }
    }
    json!({ "user": {
        "status": { "flags": flags },
        "storyreview": { "groups": { "stored": { "stories": stories } } },
        "dungeon": { "stages": stages.unwrap_or_else(|| Value::Object(Map::new())) }
    } })
}

pub async fn store_game_read(
    ctx: &AppContext,
    user: UserId,
    set: &GameStoryReadSet,
) -> Result<usize, StoreFailure> {
    if !set.present {
        return Ok(0);
    }
    let marks: Vec<GameStoryMark> = set
        .read
        .iter()
        .map(|r| GameStoryMark {
            story_id: r.story_id.clone(),
            reread_count: r.reread_count,
            unlocked_at: r.unlocked_at.and_then(|t| DateTime::from_timestamp(t, 0)),
            played: r.played,
            archived: r.archived,
            cleared: r.cleared,
            read_in_game: r.read,
        })
        .collect();
    let started = Instant::now();
    let failed = |stage: &'static str, e: myrtle_db::DbError| StoreFailure {
        stage,
        elapsed_ms: started.elapsed().as_millis(),
        message: e.to_string(),
    };
    let mut conn = ctx.db.conn().await.map_err(|e| failed("begin", e))?;
    repo::replace_game_marks(&mut conn, user, &marks)
        .await
        .map_err(|e| failed("delete/insert/commit", e))
}

#[derive(Debug, Clone)]
pub struct StoreFailure {
    pub stage: &'static str,
    pub elapsed_ms: u128,
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

pub async fn game_read_set(
    ctx: &AppContext,
    uid: &str,
    server: Server,
    raw: Arc<Value>,
) -> GameStoryReadSet {
    let built = tokio::time::timeout(
        STORY_INDEX_BUDGET,
        super::snapshot(ctx, data_server(ctx, server)),
    )
    .await;
    let snapshot = match built {
        Ok(Ok(snapshot)) => Some(snapshot),
        Ok(Err(e)) => {
            tracing::warn!(uid, error = %e, "story index unavailable; the flags half is skipped");
            None
        }
        Err(_) => {
            tracing::warn!(
                uid,
                budget_secs = STORY_INDEX_BUDGET.as_secs(),
                "story index did not build inside the budget; the flags half is skipped"
            );
            None
        }
    };
    let parsed = ctx
        .cpu
        .offload("sync_story_census", move || {
            let library = snapshot
                .as_ref()
                .and_then(|s| s.story().ok())
                .map(|s| &s.library);
            let (no_txt, no_gates) = (HashMap::new(), HashMap::new());
            parse_game_story_read(
                &raw,
                library.map_or(&no_txt, |l| &l.by_txt),
                library.map_or(&no_gates, |l| &l.gates),
            )
        })
        .await;
    parsed.unwrap_or_else(|e| {
        tracing::warn!(uid, error = %e, "story census failed; nothing is imported");
        GameStoryReadSet::default()
    })
}

pub async fn import_game_read(
    ctx: &AppContext,
    uid: &str,
    user: UserId,
    set: &GameStoryReadSet,
) -> Value {
    let started = Instant::now();
    let elapsed_ms = || u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    match tokio::time::timeout(GAME_READ_STORE_BUDGET, store_game_read(ctx, user, set)).await {
        Ok(Ok(rows)) => {
            if set.present {
                tracing::info!(
                    uid,
                    rows,
                    read = set.read_count(),
                    archived = set.archived(),
                    elapsed_ms = elapsed_ms(),
                    "game story read marks imported"
                );
            }
            json!({
                "ok": true,
                "rows": rows,
                "read": set.read_count(),
                "archived": set.archived(),
                "elapsedMs": elapsed_ms(),
            })
        }
        Ok(Err(e)) => {
            tracing::error!(
                uid,
                stage = e.stage,
                stage_ms = u64::try_from(e.elapsed_ms).unwrap_or(u64::MAX),
                elapsed_ms = elapsed_ms(),
                error = %e.message,
                "game story read import FAILED; the roster sync stands"
            );
            json!({ "ok": false, "error": e.to_string() })
        }
        Err(_) => {
            tracing::error!(
                uid,
                budget_secs = GAME_READ_STORE_BUDGET.as_secs(),
                rows_expected = set.read.len(),
                "game story read import TIMED OUT; the roster sync stands"
            );
            json!({
                "ok": false,
                "error": format!(
                    "the game read store did not answer inside {} s",
                    GAME_READ_STORE_BUDGET.as_secs()
                ),
            })
        }
    }
}
