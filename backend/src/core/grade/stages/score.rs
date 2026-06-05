use sqlx::PgPool;
use uuid::Uuid;

use crate::core::gamedata::types::GameData;
use crate::database::queries::stages::{get_known_stage_ids_for_server, get_user_stage_clears};

use super::event::{event_is_gradeable, score_event_pool};
use super::permanent::score_permanent_pool;

const PERMANENT_POOL_WEIGHT: f64 = 0.70;
const EVENT_POOL_WEIGHT: f64 = 0.30;

#[derive(Debug, Clone)]
pub struct StageGradeDetail {
    pub total: f64,
    pub permanent_pool: f64,
    pub event_pool: f64,
    pub permanent_total: usize,
    pub permanent_cleared: usize,
    pub permanent_three_starred: usize,
    pub event_total: usize,
    pub event_cleared: usize,
    pub event_three_starred: usize,
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
    let clears = &data.clears;
    let last_synced_ts = data.last_synced_ts;
    let now = chrono::Utc::now().timestamp();
    let allowed = Some(&known_stage_ids);

    let universe = &game_data.stage_universe;
    let permanent_pool = score_permanent_pool(universe, clears, allowed);
    let event_pool = score_event_pool(universe, clears, now, last_synced_ts, allowed);

    let total = ((PERMANENT_POOL_WEIGHT * permanent_pool) + (EVENT_POOL_WEIGHT * event_pool))
        .clamp(0.0, 1.0);

    let in_known = |stage_id: &str| known_stage_ids.contains(stage_id);

    let permanent_total = universe
        .permanent
        .iter()
        .filter(|e| in_known(&e.stage_id))
        .count();
    let permanent_cleared = universe
        .permanent
        .iter()
        .filter(|e| in_known(&e.stage_id))
        .filter(|e| {
            clears
                .get(&e.stage_id)
                .is_some_and(super::types::StageClear::is_cleared)
        })
        .count();
    let permanent_three_starred = universe
        .permanent
        .iter()
        .filter(|e| in_known(&e.stage_id))
        .filter(|e| {
            clears
                .get(&e.stage_id)
                .is_some_and(|c| c.is_cleared() && c.state >= 3)
        })
        .count();

    let event_in_window = |e: &crate::core::gamedata::types::stage_universe::EventEntry| -> bool {
        event_is_gradeable(e, now, last_synced_ts, allowed)
    };

    let event_total = universe.event.iter().filter(|e| event_in_window(e)).count();
    let event_cleared = universe
        .event
        .iter()
        .filter(|e| event_in_window(e))
        .filter(|e| {
            clears
                .get(&e.stage_id)
                .is_some_and(super::types::StageClear::is_cleared)
        })
        .count();
    let event_three_starred = universe
        .event
        .iter()
        .filter(|e| event_in_window(e))
        .filter(|e| {
            clears
                .get(&e.stage_id)
                .is_some_and(|c| c.is_cleared() && c.state >= 3)
        })
        .count();

    Ok(StageGradeDetail {
        total,
        permanent_pool,
        event_pool,
        permanent_total,
        permanent_cleared,
        permanent_three_starred,
        event_total,
        event_cleared,
        event_three_starred,
    })
}
