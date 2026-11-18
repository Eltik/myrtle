use chrono::{DateTime, Utc};
use myrtle_types::UserId;
use serde_json::Value;
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct StoryDocumentRow {
    pub progress: Value,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct GameStoryMark {
    pub story_id: String,
    pub reread_count: i32,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub played: bool,
    pub archived: bool,
    pub cleared: bool,
    pub read_in_game: bool,
}

#[derive(Debug, Clone)]
pub struct StoredMarkRow {
    pub story_id: String,
    pub reread_count: i32,
    pub unlocked_at: Option<DateTime<Utc>>,
    pub played: bool,
    pub archived: bool,
}

#[derive(Debug, Clone)]
pub struct MarkVerdictRow {
    pub story_id: String,
    pub archived: bool,
    pub read_in_game: bool,
    pub synced_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CommunityAccountRow {
    pub id: Uuid,
    pub stages: Option<String>,
    pub progress: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReadStoryRow {
    pub user_id: Uuid,
    pub story_id: String,
}

pub async fn document(conn: &mut Conn, user: UserId) -> Result<Option<StoryDocumentRow>, DbError> {
    Ok(sqlx::query_as!(
        StoryDocumentRow,
        "SELECT progress, updated_at FROM user_story_progress WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn put_document(
    conn: &mut Conn,
    user: UserId,
    progress: &Value,
) -> Result<StoryDocumentRow, DbError> {
    Ok(sqlx::query_as!(
        StoryDocumentRow,
        "INSERT INTO user_story_progress (user_id, progress, updated_at)
         VALUES ($1, $2, now())
         ON CONFLICT (user_id) DO UPDATE SET progress = EXCLUDED.progress, updated_at = now()
         RETURNING progress, updated_at",
        user.0,
        progress,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn replace_game_marks(
    conn: &mut Conn,
    user: UserId,
    marks: &[GameStoryMark],
) -> Result<usize, DbError> {
    let ids: Vec<&str> = marks.iter().map(|m| m.story_id.as_str()).collect();
    let counts: Vec<i32> = marks.iter().map(|m| m.reread_count).collect();
    let times: Vec<Option<DateTime<Utc>>> = marks.iter().map(|m| m.unlocked_at).collect();
    let played: Vec<bool> = marks.iter().map(|m| m.played).collect();
    let archived: Vec<bool> = marks.iter().map(|m| m.archived).collect();
    let cleared: Vec<bool> = marks.iter().map(|m| m.cleared).collect();
    let read: Vec<bool> = marks.iter().map(|m| m.read_in_game).collect();

    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    sqlx::query!(
        "DELETE FROM user_game_story_read WHERE user_id = $1",
        user.0
    )
    .execute(&mut *tx)
    .await?;
    if !ids.is_empty() {
        sqlx::query!(
            "INSERT INTO user_game_story_read
                 (user_id, story_id, reread_count, unlocked_at, played, archived, cleared,
                  read_in_game, synced_at)
             SELECT $1, s.story_id, s.reread_count, s.unlocked_at, s.played, s.archived,
                    s.cleared, s.read_in_game, now()
             FROM UNNEST($2::text[], $3::int[], $4::timestamptz[], $5::bool[], $6::bool[],
                         $7::bool[], $8::bool[])
                  AS s(story_id, reread_count, unlocked_at, played, archived, cleared,
                       read_in_game)",
            user.0,
            &ids as _,
            &counts,
            &times as _,
            &played,
            &archived,
            &cleared,
            &read,
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(ids.len())
}

pub async fn stored_marks(conn: &mut Conn, user: UserId) -> Result<Vec<StoredMarkRow>, DbError> {
    Ok(sqlx::query_as!(
        StoredMarkRow,
        "SELECT story_id, reread_count, unlocked_at, played, archived
         FROM user_game_story_read WHERE user_id = $1",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn mark_verdicts(conn: &mut Conn, user: UserId) -> Result<Vec<MarkVerdictRow>, DbError> {
    Ok(sqlx::query_as!(
        MarkVerdictRow,
        "SELECT story_id, archived, read_in_game, synced_at FROM user_game_story_read
         WHERE user_id = $1
         ORDER BY story_id",
        user.0,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn stage_blob(conn: &mut Conn, user: UserId) -> Result<Option<Value>, DbError> {
    Ok(sqlx::query_scalar!(
        "SELECT stages FROM user_stage_progress WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn accounts_with_game_reads(conn: &mut Conn) -> Result<Vec<Uuid>, DbError> {
    Ok(sqlx::query_scalar!(
        r#"SELECT DISTINCT user_id AS "user_id!" FROM user_game_story_read
         WHERE read_in_game AND user_id IS NOT NULL"#,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn community_accounts(
    conn: &mut Conn,
    after: Option<Uuid>,
    limit: i64,
) -> Result<Vec<CommunityAccountRow>, DbError> {
    Ok(sqlx::query_as!(
        CommunityAccountRow,
        r#"SELECT u.id AS "id!", sp.stages::text AS stages, gp.progress::text AS progress
         FROM users u
         JOIN user_settings s ON s.user_id = u.id
         LEFT JOIN user_stage_progress sp ON sp.user_id = u.id
         LEFT JOIN user_story_progress gp ON gp.user_id = u.id
         WHERE s.public_profile AND s.share_stats AND ($1::uuid IS NULL OR u.id > $1)
         ORDER BY u.id
         LIMIT $2"#,
        after,
        limit,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn game_reads_of(conn: &mut Conn, users: &[Uuid]) -> Result<Vec<ReadStoryRow>, DbError> {
    Ok(sqlx::query_as!(
        ReadStoryRow,
        "SELECT user_id, story_id FROM user_game_story_read
         WHERE read_in_game AND user_id = ANY($1)",
        users,
    )
    .fetch_all(conn)
    .await?)
}
