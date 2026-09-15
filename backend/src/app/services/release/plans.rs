use uuid::Uuid;

use crate::{
    app::{error::ApiError, state::AppState},
    core::release::{PutReleasePlan, ReleasePlan},
    database::queries::release as q,
};

fn wire(r: q::PlanRow) -> ReleasePlan {
    ReleasePlan {
        initial: r.initial,
        initial_manual: r.initial_manual,
        picks: serde_json::from_value(r.picks).unwrap_or_default(),
        stages: serde_json::from_value(r.stages).unwrap_or_default(),
        updated_at: r.updated_at.timestamp(),
    }
}

pub async fn get_plan(state: &AppState, user_id: Uuid) -> Result<Option<ReleasePlan>, ApiError> {
    Ok(q::get_plan(&state.db, user_id).await?.map(wire))
}

pub async fn put_plan(
    state: &AppState,
    user_id: Uuid,
    body: PutReleasePlan,
) -> Result<ReleasePlan, ApiError> {
    let picks = serde_json::to_value(&body.picks).unwrap_or_default();
    let stages = serde_json::to_value(&body.stages).unwrap_or_default();
    Ok(wire(
        q::put_plan(
            &state.db,
            user_id,
            body.initial.max(0),
            body.initial_manual,
            &picks,
            &stages,
        )
        .await?,
    ))
}
