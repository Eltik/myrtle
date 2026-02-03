//! Public statistics handler
//!
//! Provides the main handler for the `/stats` endpoint with Redis caching.

use axum::{Json, extract::State};
use redis::AsyncCommands;
use std::collections::HashMap;

use crate::app::{error::ApiError, state::AppState};

use super::types::*;

/// Handler for GET /stats
///
/// Returns comprehensive public statistics about users, gacha, game data, and tier lists.
/// Results are cached in Redis for 5 minutes.
pub async fn get_stats(State(state): State<AppState>) -> Result<Json<StatsResponse>, ApiError> {
    let cache_key = "stats:public:v1";

    // Check Redis cache first
    let mut redis = state.redis.clone();
    if let Ok(cached) = redis.get::<_, String>(cache_key).await
        && let Ok(mut response) = serde_json::from_str::<StatsResponse>(&cached)
    {
        response.cached = true;
        return Ok(Json(response));
    }

    // Fetch all stats in parallel for performance
    let (user_stats, gacha_stats, tier_list_stats) = tokio::join!(
        fetch_user_stats(&state),
        fetch_gacha_stats(&state),
        fetch_tier_list_stats(&state),
    );

    // Game data stats from in-memory (instant, no await needed)
    let game_data_stats = compute_game_data_stats(&state);

    let response = StatsResponse {
        users: user_stats?,
        gacha: gacha_stats?,
        game_data: game_data_stats,
        tier_lists: tier_list_stats?,
        computed_at: chrono::Utc::now().to_rfc3339(),
        cached: false,
    };

    // Cache for 5 minutes (300 seconds)
    if let Ok(json) = serde_json::to_string(&response) {
        let _: Result<(), _> = redis.set_ex::<_, _, ()>(cache_key, &json, 300).await;
    }

    Ok(Json(response))
}

/// Fetch user statistics with a single efficient query
async fn fetch_user_stats(state: &AppState) -> Result<UserStats, ApiError> {
    #[derive(sqlx::FromRow)]
    struct UserStatsRow {
        total: i64,
        en_count: i64,
        jp_count: i64,
        cn_count: i64,
        kr_count: i64,
        tw_count: i64,
        recent_7d: i64,
        recent_30d: i64,
        public_profiles: i64,
    }

    let row: UserStatsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total,
            COUNT(*) FILTER (WHERE server = 'en') as en_count,
            COUNT(*) FILTER (WHERE server = 'jp') as jp_count,
            COUNT(*) FILTER (WHERE server = 'cn') as cn_count,
            COUNT(*) FILTER (WHERE server = 'kr') as kr_count,
            COUNT(*) FILTER (WHERE server = 'tw') as tw_count,
            COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '7 days') as recent_7d,
            COUNT(*) FILTER (WHERE created_at > NOW() - INTERVAL '30 days') as recent_30d,
            COUNT(*) FILTER (WHERE settings->>'publicProfile' IS NULL
                             OR (settings->>'publicProfile')::BOOLEAN = true) as public_profiles
        FROM users
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error fetching user stats: {e:?}");
        ApiError::Internal("Failed to fetch user statistics".into())
    })?;

    let mut by_server = HashMap::new();
    by_server.insert("en".to_string(), row.en_count);
    by_server.insert("jp".to_string(), row.jp_count);
    by_server.insert("cn".to_string(), row.cn_count);
    by_server.insert("kr".to_string(), row.kr_count);
    by_server.insert("tw".to_string(), row.tw_count);

    Ok(UserStats {
        total: row.total,
        by_server,
        recent_signups_7d: row.recent_7d,
        recent_signups_30d: row.recent_30d,
        public_profiles: row.public_profiles,
    })
}

/// Fetch gacha statistics (only from consenting users) with a single efficient query
async fn fetch_gacha_stats(state: &AppState) -> Result<GachaStats, ApiError> {
    #[derive(sqlx::FromRow)]
    struct GachaStatsRow {
        total_pulls: i64,
        total_users: i64,
        total_six_stars: i64,
        total_five_stars: i64,
        total_four_stars: i64,
        total_three_stars: i64,
    }

    let row: GachaStatsRow = sqlx::query_as(
        r#"
        SELECT
            COUNT(*) as total_pulls,
            COUNT(DISTINCT gr.user_id) as total_users,
            COUNT(*) FILTER (WHERE rarity = 6) as total_six_stars,
            COUNT(*) FILTER (WHERE rarity = 5) as total_five_stars,
            COUNT(*) FILTER (WHERE rarity = 4) as total_four_stars,
            COUNT(*) FILTER (WHERE rarity = 3) as total_three_stars
        FROM gacha_records gr
        JOIN user_gacha_settings ugs ON gr.user_id = ugs.user_id
        WHERE ugs.share_anonymous_stats = true
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error fetching gacha stats: {e:?}");
        ApiError::Internal("Failed to fetch gacha statistics".into())
    })?;

    let total_pulls_f64 = row.total_pulls as f64;

    let six_star_rate = if row.total_pulls > 0 {
        (row.total_six_stars as f64 / total_pulls_f64) * 100.0
    } else {
        0.0
    };

    let five_star_rate = if row.total_pulls > 0 {
        (row.total_five_stars as f64 / total_pulls_f64) * 100.0
    } else {
        0.0
    };

    let four_star_rate = if row.total_pulls > 0 {
        (row.total_four_stars as f64 / total_pulls_f64) * 100.0
    } else {
        0.0
    };

    let three_star_rate = if row.total_pulls > 0 {
        (row.total_three_stars as f64 / total_pulls_f64) * 100.0
    } else {
        0.0
    };

    Ok(GachaStats {
        total_pulls: row.total_pulls,
        contributing_users: row.total_users,
        pull_rates: PullRateStats {
            six_star_rate,
            five_star_rate,
            four_star_rate,
            three_star_rate,
        },
    })
}

/// Fetch tier list statistics
async fn fetch_tier_list_stats(state: &AppState) -> Result<TierListStats, ApiError> {
    #[derive(sqlx::FromRow)]
    struct TierListStatsRow {
        total: i64,
        active: i64,
        total_versions: i64,
        total_placements: i64,
        community_count: i64,
    }

    let row: TierListStatsRow = sqlx::query_as(
        r#"
        SELECT
            (SELECT COUNT(*) FROM tier_lists) as total,
            (SELECT COUNT(*) FROM tier_lists WHERE is_active = true AND is_deleted = false) as active,
            (SELECT COUNT(*) FROM tier_list_versions) as total_versions,
            (SELECT COUNT(*) FROM tier_placements) as total_placements,
            (SELECT COUNT(*) FROM tier_lists WHERE tier_list_type = 'community' AND is_deleted = false) as community_count
        "#,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Database error fetching tier list stats: {e:?}");
        ApiError::Internal("Failed to fetch tier list statistics".into())
    })?;

    Ok(TierListStats {
        total: row.total,
        active: row.active,
        total_versions: row.total_versions,
        total_placements: row.total_placements,
        community_count: row.community_count,
    })
}

/// Compute game data statistics from in-memory state (instant, no DB)
fn compute_game_data_stats(state: &AppState) -> GameDataStats {
    GameDataStats {
        operators: state.game_data.operators.len() as i64,
        skills: state.game_data.skills.len() as i64,
        modules: state.game_data.modules.equip_dict.len() as i64,
        skins: state.game_data.skins.char_skins.len() as i64,
        items: state.game_data.materials.items.len() as i64,
        stages: state.game_data.stages.len() as i64,
        enemies: state.game_data.enemies.enemy_data.len() as i64,
        gacha_pools: state.game_data.gacha.gacha_pool_client.len() as i64,
    }
}
