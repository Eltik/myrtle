use sqlx::PgPool;

use crate::database::models::score::{LeaderboardEntry, UserScore};

/// Get leaderboard with pagination
pub async fn get_leaderboard(
    pool: &PgPool,
    server: Option<&str>,
    sort_by: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<LeaderboardEntry>, sqlx::Error> {
    let sort_col = match sort_by {
        "operator_score" => "operator_score",
        "stage_score" => "stage_score",
        "roguelike_score" => "roguelike_score",
        "sandbox_score" => "sandbox_score",
        "medal_score" => "medal_score",
        "base_score" => "base_score",
        "skin_score" => "skin_score",
        "composite_score" => "composite_score",
        _ => "total_score",
    };

    let query = if server.is_some() {
        format!(
            "SELECT * FROM v_leaderboard WHERE server = $1 ORDER BY {sort_col} DESC NULLS LAST LIMIT $2 OFFSET $3"
        )
    } else {
        format!(
            "SELECT * FROM v_leaderboard ORDER BY {sort_col} DESC NULLS LAST LIMIT $1 OFFSET $2"
        )
    };

    if let Some(srv) = server {
        sqlx::query_as::<_, LeaderboardEntry>(&query)
            .bind(srv)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    } else {
        sqlx::query_as::<_, LeaderboardEntry>(&query)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await
    }
}

/// Count leaderboard entries
pub async fn count_leaderboard(pool: &PgPool, server: Option<&str>) -> Result<i64, sqlx::Error> {
    if let Some(srv) = server {
        sqlx::query_scalar("SELECT COUNT(*) FROM v_leaderboard WHERE server = $1")
            .bind(srv)
            .fetch_one(pool)
            .await
    } else {
        sqlx::query_scalar("SELECT COUNT(*) FROM v_leaderboard")
            .fetch_one(pool)
            .await
    }
}

/// Upsert a user's score
pub async fn update_score(pool: &PgPool, score: &UserScore) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO user_scores (user_id, total_score, operator_score, stage_score,
            roguelike_score, sandbox_score, medal_score, base_score, skin_score,
            grade, composite_score, breakdown)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)
        ON CONFLICT (user_id) DO UPDATE SET
            total_score = EXCLUDED.total_score, operator_score = EXCLUDED.operator_score,
            stage_score = EXCLUDED.stage_score, roguelike_score = EXCLUDED.roguelike_score,
            sandbox_score = EXCLUDED.sandbox_score, medal_score = EXCLUDED.medal_score,
            base_score = EXCLUDED.base_score, skin_score = EXCLUDED.skin_score,
            grade = EXCLUDED.grade, composite_score = EXCLUDED.composite_score,
            breakdown = EXCLUDED.breakdown, calculated_at = NOW()
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
    .bind(score.composite_score)
    .bind(&score.breakdown)
    .execute(pool)
    .await?;
    Ok(())
}
