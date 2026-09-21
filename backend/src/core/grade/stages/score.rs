use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::database::queries::stages::{get_known_stage_ids_for_server, get_user_stage_clears};

use super::pool::{PlayerPools, PoolCounts, weighted_score};

const PERMANENT_POOL_WEIGHT: f64 = 0.70;
const EVENT_POOL_WEIGHT: f64 = 0.30;

#[derive(Debug, Clone)]
pub struct StageGradeDetail {
    /// The stage grade: 0.70 of the permanent pool plus 0.30 of the event pool.
    pub total: f64,
    pub permanent_score: f64,
    pub event_score: f64,
    pub permanent: PoolCounts,
    pub event: PoolCounts,
}

pub async fn grade_stages(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<f64, sqlx::Error> {
    grade_stages_detail(pool, user_id, game_data)
        .await
        .map(|d| d.total)
}

pub async fn grade_stages_detail(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<StageGradeDetail, sqlx::Error> {
    // Fetch the user's clears and the per-server "stages the gamedata table has
    // that this server actually ships" set in parallel. The bundled gamedata
    // can be ahead of a user's server (e.g. EN players against CN-era tables);
    // anything no user on the same server has ever seen is treated as not yet
    // available on that server and excluded from grading.
    let (data, known_stage_ids) = tokio::try_join!(
        get_user_stage_clears(pool, user_id),
        get_known_stage_ids_for_server(pool, user_id),
    )?;
    let now = chrono::Utc::now().timestamp();
    let pools = PlayerPools::new(
        &game_data.stage_universe,
        &data.clears,
        Some(&known_stage_ids),
        now,
        data.last_synced_ts,
    );

    let permanent_score = weighted_score(pools.permanent());
    let event_score = weighted_score(pools.event());
    let total = ((PERMANENT_POOL_WEIGHT * permanent_score) + (EVENT_POOL_WEIGHT * event_score))
        .clamp(0.0, 1.0);

    Ok(StageGradeDetail {
        total,
        permanent_score,
        event_score,
        permanent: PoolCounts::of(pools.permanent()),
        event: PoolCounts::of(pools.event()),
    })
}
