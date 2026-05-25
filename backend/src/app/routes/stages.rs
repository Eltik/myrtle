use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::queries::{stages, users};

#[derive(Deserialize)]
pub struct StageClearsParams {
    pub uid: Option<String>,
}

#[derive(Serialize)]
pub struct StageClearDto {
    pub state: i16,
    #[serde(rename = "completeTimes")]
    pub complete_times: i32,
    #[serde(rename = "practiceTimes")]
    pub practice_times: i32,
}

async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    match uid_param {
        Some(uid) => {
            let profile = users::find_by_uid(&state.db, uid)
                .await?
                .ok_or(ApiError::NotFound)?;

            let is_own = auth
                .0
                .as_ref()
                .and_then(|a| a.user_id.parse::<Uuid>().ok())
                .is_some_and(|id| id == profile.id);

            if !is_own && profile.public_profile != Some(true) {
                return Err(ApiError::Forbidden);
            }

            Ok(profile.id)
        }
        None => {
            let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
            auth.user_id.parse().map_err(|_| ApiError::Unauthorized)
        }
    }
}

pub async fn get_stage_clears(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<StageClearsParams>,
) -> Result<Json<std::collections::HashMap<String, StageClearDto>>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let data = stages::get_user_stage_clears(&state.db, user_id).await?;
    let body = data
        .clears
        .into_iter()
        .map(|(id, c)| {
            (
                id,
                StageClearDto {
                    state: c.state,
                    complete_times: c.complete_times,
                    practice_times: c.practice_times,
                },
            )
        })
        .collect();
    Ok(Json(body))
}
