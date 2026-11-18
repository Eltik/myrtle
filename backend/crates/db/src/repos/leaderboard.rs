use chrono::{DateTime, Utc};
use myrtle_types::Server;
use uuid::Uuid;

use crate::sql::contains_pattern;
use crate::{Conn, DbError};

#[derive(Debug, Clone)]
pub struct LeaderboardRow {
    pub id: Uuid,
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub level: Option<i16>,
    pub avatar_id: Option<String>,
    pub secretary: Option<String>,
    pub secretary_skin_id: Option<String>,
    pub server: String,
    pub total_score: Option<f64>,
    pub grade: Option<String>,
    pub operator_score: Option<f64>,
    pub stage_score: Option<f64>,
    pub roguelike_score: Option<f64>,
    pub sandbox_score: Option<f64>,
    pub medal_score: Option<f64>,
    pub base_score: Option<f64>,
    pub skin_score: Option<f64>,
    pub rank_global: Option<i64>,
    pub rank_server: Option<i64>,
    /// Positive = climbed since the baseline
    /// `None` without a movement interval or a baseline
    pub rank_delta: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct MoverRow {
    pub uid: String,
    pub nickname: Option<String>,
    pub nick_number: Option<String>,
    pub avatar_id: Option<String>,
    pub server: String,
    pub current_rank: i64,
    pub previous_rank: i64,
    pub rank_delta: i64,
    pub current_score: Option<f64>,
    pub score_delta: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct StandingRows {
    pub player: LeaderboardRow,
    /// Ranked within the window around the player (server doesn't matter)
    pub neighbours: Vec<LeaderboardRow>,
    /// Every ranked player
    pub ranked: i64,
    pub previous_rank: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LeaderboardSort {
    #[default]
    Total,
    Operator,
    Stage,
    Roguelike,
    Sandbox,
    Medal,
    Base,
    Skin,
}

impl LeaderboardSort {
    #[must_use]
    pub const fn column(self) -> &'static str {
        match self {
            Self::Total => "total_score",
            Self::Operator => "operator_score",
            Self::Stage => "stage_score",
            Self::Roguelike => "roguelike_score",
            Self::Sandbox => "sandbox_score",
            Self::Medal => "medal_score",
            Self::Base => "base_score",
            Self::Skin => "skin_score",
        }
    }
}

/// Rank change against the latest snapshot at least `interval` old
#[derive(Debug, Clone, Copy)]
pub struct Movement<'a> {
    /// Postgres interval text (ex. `7 days`)
    pub interval: &'a str,
    /// Drops rows that did not move or have no baseline
    pub moved_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoverDirection {
    Up,
    Down,
}

/// Note: Unscored players sort last on every column
pub async fn page(
    conn: &mut Conn,
    sort: LeaderboardSort,
    server: Option<Server>,
    q: Option<&str>,
    movement: Option<Movement<'_>>,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardRow>, DbError> {
    let server = server.map(Server::as_str);
    let pattern = q.map(contains_pattern);
    let Some(movement) = movement else {
        return Ok(sqlx::query_as!(
            LeaderboardRow,
            r#"SELECT id AS "id!", uid AS "uid!", nickname, nick_number, level, avatar_id,
                      secretary, secretary_skin_id, server AS "server!", total_score, grade,
                      operator_score, stage_score, roguelike_score, sandbox_score,
                      medal_score, base_score, skin_score, rank_global, rank_server,
                      NULL::bigint AS rank_delta
               FROM v_leaderboard
               WHERE ($1::text IS NULL OR server = $1)
                 AND ($2::text IS NULL OR nickname ILIKE $2 ESCAPE '\' OR uid ILIKE $2 ESCAPE '\')
               ORDER BY CASE $3::text
                   WHEN 'operator_score' THEN operator_score
                   WHEN 'stage_score' THEN stage_score
                   WHEN 'roguelike_score' THEN roguelike_score
                   WHEN 'sandbox_score' THEN sandbox_score
                   WHEN 'medal_score' THEN medal_score
                   WHEN 'base_score' THEN base_score
                   WHEN 'skin_score' THEN skin_score
                   ELSE total_score
               END DESC NULLS LAST, id
               LIMIT $4 OFFSET $5"#,
            server,
            pattern,
            sort.column(),
            limit,
            offset,
        )
        .fetch_all(conn)
        .await?);
    };
    Ok(sqlx::query_as!(
        LeaderboardRow,
        r#"WITH baseline AS (
               SELECT e.user_id, e.rank_global
               FROM leaderboard_snapshot_entries e
               WHERE e.snapshot_id = (
                   SELECT id FROM leaderboard_snapshots
                   WHERE taken_at <= NOW() - $1::text::interval
                   ORDER BY taken_at DESC, id DESC
                   LIMIT 1
               )
           )
           SELECT v.id AS "id!", v.uid AS "uid!", v.nickname, v.nick_number, v.level,
                  v.avatar_id, v.secretary, v.secretary_skin_id, v.server AS "server!",
                  v.total_score, v.grade, v.operator_score, v.stage_score,
                  v.roguelike_score, v.sandbox_score, v.medal_score, v.base_score,
                  v.skin_score, v.rank_global, v.rank_server,
                  (b.rank_global - v.rank_global)::bigint AS rank_delta
           FROM v_leaderboard v
           LEFT JOIN baseline b ON b.user_id = v.id
           WHERE ($2::text IS NULL OR v.server = $2)
             AND ($3::text IS NULL OR v.nickname ILIKE $3 ESCAPE '\' OR v.uid ILIKE $3 ESCAPE '\')
             AND (NOT $4::bool OR (b.rank_global IS NOT NULL AND b.rank_global <> v.rank_global))
           ORDER BY CASE $5::text
               WHEN 'operator_score' THEN v.operator_score
               WHEN 'stage_score' THEN v.stage_score
               WHEN 'roguelike_score' THEN v.roguelike_score
               WHEN 'sandbox_score' THEN v.sandbox_score
               WHEN 'medal_score' THEN v.medal_score
               WHEN 'base_score' THEN v.base_score
               WHEN 'skin_score' THEN v.skin_score
               ELSE v.total_score
           END DESC NULLS LAST, v.id
           LIMIT $6 OFFSET $7"#,
        movement.interval,
        server,
        pattern,
        movement.moved_only,
        sort.column(),
        limit,
        offset,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn count(
    conn: &mut Conn,
    server: Option<Server>,
    q: Option<&str>,
    movement: Option<Movement<'_>>,
) -> Result<i64, DbError> {
    let server = server.map(Server::as_str);
    let pattern = q.map(contains_pattern);
    let Some(movement) = movement.filter(|m| m.moved_only) else {
        return Ok(sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!"
               FROM v_leaderboard
               WHERE ($1::text IS NULL OR server = $1)
                 AND ($2::text IS NULL OR nickname ILIKE $2 ESCAPE '\' OR uid ILIKE $2 ESCAPE '\')"#,
            server,
            pattern,
        )
        .fetch_one(conn)
        .await?);
    };
    Ok(sqlx::query_scalar!(
        r#"WITH baseline AS (
               SELECT e.user_id, e.rank_global
               FROM leaderboard_snapshot_entries e
               WHERE e.snapshot_id = (
                   SELECT id FROM leaderboard_snapshots
                   WHERE taken_at <= NOW() - $1::text::interval
                   ORDER BY taken_at DESC, id DESC
                   LIMIT 1
               )
           )
           SELECT COUNT(*) AS "count!"
           FROM v_leaderboard v
           JOIN baseline b ON b.user_id = v.id
           WHERE b.rank_global <> v.rank_global
             AND ($2::text IS NULL OR v.server = $2)
             AND ($3::text IS NULL OR v.nickname ILIKE $3 ESCAPE '\' OR v.uid ILIKE $3 ESCAPE '\')"#,
        movement.interval,
        server,
        pattern,
    )
    .fetch_one(conn)
    .await?)
}

pub async fn top_movers(
    conn: &mut Conn,
    interval: &str,
    direction: MoverDirection,
    server: Option<Server>,
    limit: i64,
) -> Result<Vec<MoverRow>, DbError> {
    let sign: i64 = match direction {
        MoverDirection::Up => 1,
        MoverDirection::Down => -1,
    };
    Ok(sqlx::query_as!(
        MoverRow,
        r#"WITH baseline AS (
               SELECT e.user_id, e.rank_global, e.total_score
               FROM leaderboard_snapshot_entries e
               WHERE e.snapshot_id = (
                   SELECT id FROM leaderboard_snapshots
                   WHERE taken_at <= NOW() - $1::text::interval
                   ORDER BY taken_at DESC, id DESC
                   LIMIT 1
               )
           )
           SELECT v.uid AS "uid!", v.nickname, v.nick_number, v.avatar_id,
                  v.server AS "server!",
                  v.rank_global AS "current_rank!",
                  b.rank_global::bigint AS "previous_rank!",
                  (b.rank_global - v.rank_global)::bigint AS "rank_delta!",
                  v.total_score AS current_score,
                  v.total_score - b.total_score AS score_delta
           FROM v_leaderboard v
           JOIN baseline b ON b.user_id = v.id
           WHERE b.rank_global <> v.rank_global
             AND ($3::text IS NULL OR v.server = $3)
           ORDER BY (b.rank_global - v.rank_global) * $4 DESC, v.id
           LIMIT $2"#,
        interval,
        limit,
        server.map(Server::as_str),
        sign,
    )
    .fetch_all(conn)
    .await?)
}

pub async fn standing(
    conn: &mut Conn,
    uid: &str,
    server: Server,
    window: i64,
    interval: &str,
) -> Result<Option<StandingRows>, DbError> {
    let rows = sqlx::query!(
        r#"WITH board AS MATERIALIZED (
               SELECT id, uid, nickname, nick_number, level, avatar_id, secretary,
                      secretary_skin_id, server, total_score, grade, operator_score,
                      stage_score, roguelike_score, sandbox_score, medal_score, base_score,
                      skin_score, rank_global, rank_server, count(*) OVER () AS ranked
               FROM v_leaderboard
           ),
           me AS (
               SELECT b.id, b.rank_global,
                      (SELECT e.rank_global
                       FROM leaderboard_snapshot_entries e
                       WHERE e.user_id = b.id AND e.snapshot_id = (
                           SELECT id FROM leaderboard_snapshots
                           WHERE taken_at <= NOW() - $4::text::interval
                           ORDER BY taken_at DESC, id DESC
                           LIMIT 1
                       )) AS previous_rank
               FROM board b WHERE b.uid = $1 AND b.server = $2
           )
           SELECT b.id AS "id!", b.uid AS "uid!", b.nickname, b.nick_number, b.level,
                  b.avatar_id, b.secretary, b.secretary_skin_id, b.server AS "server!",
                  b.total_score, b.grade, b.operator_score, b.stage_score, b.roguelike_score,
                  b.sandbox_score, b.medal_score, b.base_score, b.skin_score, b.rank_global,
                  b.rank_server, b.ranked AS "ranked!", b.id = me.id AS "is_player!",
                  me.previous_rank
           FROM board b CROSS JOIN me
           WHERE b.rank_global BETWEEN GREATEST(me.rank_global - $3, 1) AND me.rank_global + $3
           ORDER BY b.rank_global"#,
        uid,
        server.as_str(),
        window.max(0),
        interval,
    )
    .fetch_all(conn)
    .await?;

    let (mut player, mut ranked, mut previous_rank) = (None, 0, None);
    let mut neighbours = Vec::with_capacity(rows.len());
    for r in rows {
        ranked = r.ranked;
        previous_rank = r.previous_rank;
        let row = LeaderboardRow {
            id: r.id,
            uid: r.uid,
            nickname: r.nickname,
            nick_number: r.nick_number,
            level: r.level,
            avatar_id: r.avatar_id,
            secretary: r.secretary,
            secretary_skin_id: r.secretary_skin_id,
            server: r.server,
            total_score: r.total_score,
            grade: r.grade,
            operator_score: r.operator_score,
            stage_score: r.stage_score,
            roguelike_score: r.roguelike_score,
            sandbox_score: r.sandbox_score,
            medal_score: r.medal_score,
            base_score: r.base_score,
            skin_score: r.skin_score,
            rank_global: r.rank_global,
            rank_server: r.rank_server,
            rank_delta: None,
        };
        if r.is_player {
            player = Some(row);
        } else {
            neighbours.push(row);
        }
    }
    Ok(player.map(|player| StandingRows {
        player,
        neighbours,
        ranked,
        previous_rank,
    }))
}

pub async fn latest_snapshot_at(conn: &mut Conn) -> Result<Option<DateTime<Utc>>, DbError> {
    Ok(
        sqlx::query_scalar!(r#"SELECT MAX(taken_at) AS "latest?" FROM leaderboard_snapshots"#)
            .fetch_one(conn)
            .await?,
    )
}

pub async fn take_snapshot(conn: &mut Conn) -> Result<i64, DbError> {
    let mut tx = sqlx::Connection::begin(&mut *conn).await?;
    let snapshot_id =
        sqlx::query_scalar!("INSERT INTO leaderboard_snapshots DEFAULT VALUES RETURNING id")
            .fetch_one(&mut *tx)
            .await?;
    sqlx::query!(
        "INSERT INTO leaderboard_snapshot_entries
             (snapshot_id, user_id, server_id, rank_global, rank_server, total_score)
         SELECT $1, v.id, srv.id, v.rank_global, v.rank_server, v.total_score
         FROM v_leaderboard v
         JOIN servers srv ON srv.code = v.server",
        snapshot_id,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(snapshot_id)
}
