use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Uuid;

use crate::database::models::score::{
    LeaderboardEntry, LeaderboardMover, PlayerStanding, ServerShare, UserScore,
};

/// Fetch the full score row for one user by `uid` (external 10-digit id).
/// Returns `None` if the user has never been scored.
pub async fn get_score_by_uid(pool: &PgPool, uid: &str) -> Result<Option<UserScore>, sqlx::Error> {
    sqlx::query_as::<_, UserScore>(
        r#"
        SELECT sc.*
        FROM user_scores sc
        JOIN users u ON u.id = sc.user_id
        WHERE u.uid = $1
        "#,
    )
    .bind(uid)
    .fetch_optional(pool)
    .await
}

/// Fetch the full score row for one user by internal user_id (Uuid).
#[allow(dead_code)]
pub async fn get_score_by_user_id(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Option<UserScore>, sqlx::Error> {
    sqlx::query_as::<_, UserScore>("SELECT * FROM user_scores WHERE user_id = $1")
        .bind(user_id)
        .fetch_optional(pool)
        .await
}

fn sort_column(sort_by: &str) -> &'static str {
    match sort_by {
        "operator_score" => "operator_score",
        "stage_score" => "stage_score",
        "roguelike_score" => "roguelike_score",
        "sandbox_score" => "sandbox_score",
        "medal_score" => "medal_score",
        "base_score" => "base_score",
        "skin_score" => "skin_score",
        _ => "total_score",
    }
}

/// Get leaderboard with pagination.
///
/// When `movement_interval` is provided, each row is enriched with `rank_delta`
/// computed against the most recent snapshot taken before that interval.
/// When `movement_only` is also true, rows whose rank hasn't changed (or which
/// have no baseline yet) are excluded — count and pagination both reflect the
/// filtered population, so the caller can paginate normally.
pub async fn get_leaderboard(
    pool: &PgPool,
    server: Option<&str>,
    sort_by: &str,
    movement_interval: Option<&str>,
    movement_only: bool,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let sort_col = sort_column(sort_by);

    let Some(interval) = movement_interval else {
        let query = if server.is_some() {
            format!(
                "SELECT * FROM v_leaderboard WHERE server = $1 ORDER BY {sort_col} DESC NULLS LAST LIMIT $2 OFFSET $3"
            )
        } else {
            format!(
                "SELECT * FROM v_leaderboard ORDER BY {sort_col} DESC NULLS LAST LIMIT $1 OFFSET $2"
            )
        };
        let mut q = sqlx::query_as::<_, LeaderboardEntry>(&query);
        if let Some(srv) = server {
            q = q.bind(srv);
        }
        return q.bind(limit).bind(offset).fetch_all(pool).await;
    };

    // $1 = interval, [$2 = server,] then limit, offset
    let (server_clause, limit_idx) = match server {
        Some(_) => ("AND v.server = $2", 3),
        None => ("", 2),
    };
    let offset_idx = limit_idx + 1;
    let movement_filter = if movement_only {
        "AND b.rank_global IS NOT NULL AND b.rank_global <> v.rank_global"
    } else {
        ""
    };

    let sql = format!(
        r#"
        WITH baseline AS (
            SELECT DISTINCT ON (e.user_id)
                e.user_id, e.rank_global
            FROM leaderboard_snapshot_entries e
            JOIN leaderboard_snapshots s ON s.id = e.snapshot_id
            WHERE s.taken_at <= NOW() - $1::interval
            ORDER BY e.user_id, s.taken_at DESC
        )
        SELECT v.*,
               (b.rank_global - v.rank_global)::bigint AS rank_delta
        FROM v_leaderboard v
        LEFT JOIN baseline b ON b.user_id = v.id
        WHERE v.rank_global IS NOT NULL
          {server_clause}
          {movement_filter}
        ORDER BY v.{sort_col} DESC NULLS LAST
        LIMIT ${limit_idx} OFFSET ${offset_idx}
        "#
    );

    let mut q = sqlx::query_as::<_, LeaderboardEntry>(&sql).bind(interval);
    if let Some(srv) = server {
        q = q.bind(srv);
    }
    q.bind(limit).bind(offset).fetch_all(pool).await
}

/// Count leaderboard entries. When `movement_interval` is provided and
/// `movement_only` is true, the count is restricted to users with non-zero
/// movement since the baseline — so it matches the paginated rows.
pub async fn count_leaderboard(
    pool: &PgPool,
    server: Option<&str>,
    movement_interval: Option<&str>,
    movement_only: bool,
) -> Result<i64, sqlx::Error> {
    if !movement_only || movement_interval.is_none() {
        return if let Some(srv) = server {
            sqlx::query_scalar("SELECT COUNT(*) FROM v_leaderboard WHERE server = $1")
                .bind(srv)
                .fetch_one(pool)
                .await
        } else {
            sqlx::query_scalar("SELECT COUNT(*) FROM v_leaderboard")
                .fetch_one(pool)
                .await
        };
    }

    let interval = movement_interval.expect("checked above");
    let server_clause = if server.is_some() {
        "AND v.server = $2"
    } else {
        ""
    };

    let sql = format!(
        r#"
        WITH baseline AS (
            SELECT DISTINCT ON (e.user_id)
                e.user_id, e.rank_global
            FROM leaderboard_snapshot_entries e
            JOIN leaderboard_snapshots s ON s.id = e.snapshot_id
            WHERE s.taken_at <= NOW() - $1::interval
            ORDER BY e.user_id, s.taken_at DESC
        )
        SELECT COUNT(*)
        FROM v_leaderboard v
        JOIN baseline b ON b.user_id = v.id
        WHERE v.rank_global IS NOT NULL
          AND b.rank_global <> v.rank_global
          {server_clause}
        "#
    );

    let mut q = sqlx::query_scalar::<_, i64>(&sql).bind(interval);
    if let Some(srv) = server {
        q = q.bind(srv);
    }
    q.fetch_one(pool).await
}

/// Upsert a user's score
pub async fn update_score(pool: &PgPool, score: &UserScore) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_scores (user_id, total_score, operator_score, stage_score,
            roguelike_score, sandbox_score, medal_score, base_score, skin_score,
            grade)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
        ON CONFLICT (user_id) DO UPDATE SET
            total_score = EXCLUDED.total_score, operator_score = EXCLUDED.operator_score,
            stage_score = EXCLUDED.stage_score, roguelike_score = EXCLUDED.roguelike_score,
            sandbox_score = EXCLUDED.sandbox_score, medal_score = EXCLUDED.medal_score,
            base_score = EXCLUDED.base_score, skin_score = EXCLUDED.skin_score,
            grade = EXCLUDED.grade,
            calculated_at = NOW()
    "#,
    )
    .bind(score.user_id)
    .bind(score.total_score)
    .bind(score.operator_score)
    .bind(score.stage_score)
    .bind(score.roguelike_score)
    .bind(score.sandbox_score)
    .bind(score.medal_score)
    .bind(score.base_score)
    .bind(score.skin_score)
    .bind(&score.grade)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_leaderboard_snapshot_at(
    pool: &PgPool,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    sqlx::query_scalar("SELECT MAX(taken_at) FROM leaderboard_snapshots")
        .fetch_one(pool)
        .await
}

/// Snapshot the current leaderboard. Returns the new snapshot id.
pub async fn take_leaderboard_snapshot(pool: &PgPool) -> Result<i64, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let snapshot_id: i64 =
        sqlx::query_scalar("INSERT INTO leaderboard_snapshots DEFAULT VALUES RETURNING id")
            .fetch_one(&mut *tx)
            .await?;

    sqlx::query(
        r#"
        INSERT INTO leaderboard_snapshot_entries
            (snapshot_id, user_id, server_id, rank_global, rank_server, total_score)
        SELECT $1, v.id, srv.id, v.rank_global, v.rank_server, v.total_score
        FROM v_leaderboard v
        JOIN servers srv ON srv.code = v.server
        WHERE v.rank_global IS NOT NULL
        "#,
    )
    .bind(snapshot_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(snapshot_id)
}

/// Top movers since `interval` ago. `direction` is "up" or "down".
/// `rank_floor` excludes baselines worse than this rank (cuts noise from
/// players who jumped from "unranked-ish" to mid-pack).
pub async fn get_top_movers(
    pool: &PgPool,
    interval: &str,  // e.g. "7 days", "1 day"
    direction: &str, // "up" or "down"
    server: Option<&str>,
    rank_floor: i64,
    limit: i64,
) -> Result<Vec<LeaderboardMover>, sqlx::Error> {
    let order = if direction == "down" { "ASC" } else { "DESC" };
    let server_clause = if server.is_some() {
        "AND v.server = $4"
    } else {
        ""
    };

    let sql = format!(
        r#"
        WITH baseline AS (
            SELECT DISTINCT ON (e.user_id)
                e.user_id, e.rank_global, e.total_score
            FROM leaderboard_snapshot_entries e
            JOIN leaderboard_snapshots s ON s.id = e.snapshot_id
            WHERE s.taken_at <= NOW() - $1::interval
            ORDER BY e.user_id, s.taken_at DESC
        )
        SELECT
            v.uid, v.nickname, v.nick_number, v.avatar_id, v.server,
            v.rank_global                       AS current_rank,
            b.rank_global::bigint               AS previous_rank,
            (b.rank_global - v.rank_global)::bigint AS rank_delta,
            v.total_score                       AS current_score,
            v.total_score - b.total_score       AS score_delta
        FROM v_leaderboard v
        JOIN baseline b ON b.user_id = v.id
        WHERE v.rank_global IS NOT NULL
        AND b.rank_global <= $2
        {server_clause}
        ORDER BY rank_delta {order}
        LIMIT $3
        "#
    );

    let mut q = sqlx::query_as::<_, LeaderboardMover>(&sql)
        .bind(interval)
        .bind(rank_floor)
        .bind(limit);
    if let Some(s) = server {
        q = q.bind(s);
    }
    q.fetch_all(pool).await
}

pub async fn get_server_distribution(
    pool: &PgPool,
    top_n: i64,
) -> Result<Vec<ServerShare>, sqlx::Error> {
    sqlx::query_as::<_, ServerShare>(
        r#"
        SELECT server, COUNT(*) AS players
        FROM (
            SELECT server FROM v_leaderboard
            WHERE rank_global IS NOT NULL
            ORDER BY total_score DESC NULLS LAST
            LIMIT $1
        ) t
        GROUP BY server
        ORDER BY players DESC
        "#,
    )
    .bind(top_n)
    .fetch_all(pool)
    .await
}

pub async fn get_last_updated(
    pool: &PgPool,
    server: Option<&str>,
) -> Result<Option<DateTime<Utc>>, sqlx::Error> {
    if let Some(srv) = server {
        sqlx::query_scalar(
            r#"
            SELECT MAX(sc.calculated_at)
            FROM user_scores sc
            JOIN users u ON u.id = sc.user_id
            JOIN servers s ON s.id = u.server_id
            WHERE s.code = $1
            "#,
        )
        .bind(srv)
        .fetch_one(pool)
        .await
    } else {
        sqlx::query_scalar("SELECT MAX(calculated_at) FROM user_scores")
            .fetch_one(pool)
            .await
    }
}

pub async fn get_player_standing(
    pool: &PgPool,
    uid: &str,
    server: &str,
    window: i64, // neighbors above + below
) -> Result<Option<PlayerStanding>, sqlx::Error> {
    let player = sqlx::query_as::<_, LeaderboardEntry>(
        "SELECT * FROM v_leaderboard WHERE uid = $1 AND server = $2",
    )
    .bind(uid)
    .bind(server)
    .fetch_optional(pool)
    .await?;

    let Some(player) = player else {
        return Ok(None);
    };
    let Some(rank) = player.rank_global else {
        return Ok(None);
    };

    let (neighbors, total, prev_rank) = tokio::try_join!(
        sqlx::query_as::<_, LeaderboardEntry>(
            r#"
            SELECT * FROM v_leaderboard
            WHERE rank_global BETWEEN $1 AND $2 AND uid <> $3
            ORDER BY rank_global ASC
            "#,
        )
        .bind((rank - window).max(1))
        .bind(rank + window)
        .bind(uid)
        .fetch_all(pool),
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM v_leaderboard WHERE rank_global IS NOT NULL",
        )
        .fetch_one(pool),
        sqlx::query_scalar::<_, Option<i32>>(
            r#"
            SELECT e.rank_global
            FROM leaderboard_snapshot_entries e
            JOIN leaderboard_snapshots s ON s.id = e.snapshot_id
            WHERE e.user_id = $1 AND s.taken_at <= NOW() - INTERVAL '7 days'
            ORDER BY s.taken_at DESC
            LIMIT 1
            "#,
        )
        .bind(player.id)
        .fetch_optional(pool),
    )?;

    let percentile = if total > 0 {
        rank as f64 / total as f64
    } else {
        0.0
    };
    let rank_delta_7d = prev_rank.flatten().map(|p| p as i64 - rank);

    Ok(Some(PlayerStanding {
        player,
        neighbors,
        percentile,
        rank_delta_7d,
    }))
}
