use axum::Json;
use axum::extract::{Query, State};
use serde::Deserialize;

use crate::app::cpu;
use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::routes::resolve_uid;
use crate::app::services::improvements::{ImprovementsResponse, get_improvements};
use crate::app::state::AppState;

#[derive(Deserialize)]
pub struct ImprovementsParams {
    pub uid: Option<String>,
}

pub async fn get_user_improvements(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<ImprovementsParams>,
) -> Result<Json<ImprovementsResponse>, ApiError> {
    let uid = resolve_uid(&state, &auth, params.uid.as_deref()).await?;
    // Several full optimizer passes per call, so it is admission-controlled
    // like the planner routes.
    let _admission = cpu::admit("user_improvements")?;
    let body = get_improvements(&state, &uid).await?;
    Ok(Json(body))
}
