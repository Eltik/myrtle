use chrono::{DateTime, Utc};
use myrtle_types::{Server, UserId};
use uuid::Uuid;

use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct ScoreRow {
    pub user_id: Uuid,
    pub total_score: f64,
    pub operator_score: f64,
    pub stage_score: f64,
    pub roguelike_score: f64,
    pub sandbox_score: f64,
    pub medal_score: f64,
    pub base_score: f64,
    pub base_utilization: Option<f64>,
    pub base_infrastructure: Option<f64>,
    pub skin_score: f64,
    pub grade: Option<String>,
    pub calculated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct ScoreWrite<'a> {
    pub total_score: f64,
    pub operator_score: f64,
    pub stage_score: f64,
    pub roguelike_score: f64,
    pub sandbox_score: f64,
    pub medal_score: f64,
    pub base_score: f64,
    pub base_utilization: Option<f64>,
    pub base_infrastructure: Option<f64>,
    pub skin_score: f64,
    pub grade: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub struct StoredSections {
    pub operator_score: f64,
    pub base_score: f64,
    pub roguelike_score: f64,
    pub medal_score: f64,
    pub stage_score: f64,
}

#[derive(Debug, Clone)]
pub struct ScoreHistoryRow {
    pub taken_at: DateTime<Utc>,
    pub total_score: Option<f64>,
    pub rank_global: i32,
    pub rank_server: i32,
}

#[derive(Debug, Clone)]
pub struct ServerShareRow {
    pub server: String,
    pub players: i64,
}

pub async fn get_by_uid(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<Option<ScoreRow>, DbError> {
    Ok(sqlx::query_as!(
        ScoreRow,
        "SELECT user_id, total_score, operator_score, stage_score, roguelike_score,
                sandbox_score, medal_score, base_score, base_utilization,
                base_infrastructure, skin_score, grade, calculated_at
         FROM user_scores
         WHERE user_id = (SELECT id FROM users
                          WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2)
                          ORDER BY created_at, id LIMIT 1)",
        uid,
        server.map(Server::index),
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn get(conn: &mut Conn, user: UserId) -> Result<Option<ScoreRow>, DbError> {
    Ok(sqlx::query_as!(
        ScoreRow,
        "SELECT user_id, total_score, operator_score, stage_score, roguelike_score,
                sandbox_score, medal_score, base_score, base_utilization,
                base_infrastructure, skin_score, grade, calculated_at
         FROM user_scores WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn upsert(conn: &mut Conn, user: UserId, score: &ScoreWrite<'_>) -> Result<(), DbError> {
    sqlx::query!(
        "INSERT INTO user_scores (user_id, total_score, operator_score, stage_score,
             roguelike_score, sandbox_score, medal_score, base_score,
             base_utilization, base_infrastructure, skin_score, grade)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
         ON CONFLICT (user_id) DO UPDATE SET
             total_score = EXCLUDED.total_score, operator_score = EXCLUDED.operator_score,
             stage_score = EXCLUDED.stage_score, roguelike_score = EXCLUDED.roguelike_score,
             sandbox_score = EXCLUDED.sandbox_score, medal_score = EXCLUDED.medal_score,
             base_score = EXCLUDED.base_score,
             base_utilization = EXCLUDED.base_utilization,
             base_infrastructure = EXCLUDED.base_infrastructure,
             skin_score = EXCLUDED.skin_score,
             grade = EXCLUDED.grade,
             calculated_at = NOW()",
        user.0,
        score.total_score,
        score.operator_score,
        score.stage_score,
        score.roguelike_score,
        score.sandbox_score,
        score.medal_score,
        score.base_score,
        score.base_utilization,
        score.base_infrastructure,
        score.skin_score,
        score.grade,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn stored_sections(
    conn: &mut Conn,
    user: UserId,
) -> Result<Option<StoredSections>, DbError> {
    Ok(sqlx::query_as!(
        StoredSections,
        "SELECT operator_score, base_score, roguelike_score, medal_score, stage_score
         FROM user_scores WHERE user_id = $1",
        user.0,
    )
    .fetch_optional(conn)
    .await?)
}

pub async fn write_sandbox(
    conn: &mut Conn,
    user: UserId,
    sandbox_score: f64,
    total_score: f64,
    grade: &str,
) -> Result<(), DbError> {
    sqlx::query!(
        "UPDATE user_scores SET sandbox_score = $2, total_score = $3, grade = $4,
             calculated_at = NOW()
         WHERE user_id = $1",
        user.0,
        sandbox_score,
        total_score,
        grade,
    )
    .execute(conn)
    .await?;
    Ok(())
}

pub async fn history(
    conn: &mut Conn,
    uid: &str,
    server: Option<Server>,
) -> Result<Vec<ScoreHistoryRow>, DbError> {
    Ok(sqlx::query_as!(
        ScoreHistoryRow,
        "SELECT s.taken_at, e.total_score, e.rank_global, e.rank_server
         FROM leaderboard_snapshot_entries e
         JOIN leaderboard_snapshots s ON s.id = e.snapshot_id
         WHERE e.user_id = (SELECT id FROM users
                            WHERE uid = $1 AND ($2::smallint IS NULL OR server_id = $2)
                            ORDER BY created_at, id LIMIT 1)
         ORDER BY s.taken_at",
        uid,
        server.map(Server::index),
    )
    .fetch_all(conn)
    .await?)
}

pub async fn last_calculated_at(
    conn: &mut Conn,
    server: Option<Server>,
) -> Result<Option<DateTime<Utc>>, DbError> {
    Ok(match server {
        Some(server) => {
            sqlx::query_scalar!(
                r#"SELECT MAX(sc.calculated_at) AS "latest?"
                 FROM user_scores sc
                 JOIN users u ON u.id = sc.user_id
                 WHERE u.server_id = $1"#,
                server.index(),
            )
            .fetch_one(conn)
            .await?
        }
        None => {
            sqlx::query_scalar!(r#"SELECT MAX(calculated_at) AS "latest?" FROM user_scores"#)
                .fetch_one(conn)
                .await?
        }
    })
}

pub async fn server_shares(conn: &mut Conn, top_n: i64) -> Result<Vec<ServerShareRow>, DbError> {
    Ok(sqlx::query_as!(
        ServerShareRow,
        r#"SELECT server AS "server!", COUNT(*) AS "players!"
           FROM (
               SELECT server FROM v_leaderboard
               ORDER BY total_score DESC NULLS LAST
               LIMIT $1
           ) t
           GROUP BY server
           ORDER BY COUNT(*) DESC"#,
        top_n,
    )
    .fetch_all(conn)
    .await?)
}
