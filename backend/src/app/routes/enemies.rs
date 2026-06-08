use axum::Json;
use axum::extract::{Query, State};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::cache::keys::CacheKey;
use crate::app::error::ApiError;
use crate::app::extractors::auth::MaybeAuthUser;
use crate::app::state::AppState;
use crate::database::queries::enemies::{
    get_community_average_encountered, get_user_encountered_enemies,
};
use crate::database::queries::users::find_by_uid;

#[derive(Deserialize)]
pub struct EncounteredEnemiesParams {
    pub uid: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredEnemy {
    pub enemy_id: String,
    /// Handbook display fields; `None` when the encountered id isn't present in
    /// the loaded `enemy_handbook_table` (e.g. an unreleased/unknown variant).
    pub name: Option<String>,
    pub enemy_index: Option<String>,
    pub sort_id: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounteredEnemiesResponse {
    /// Number of distinct enemies the user has encountered.
    pub encountered_count: usize,
    /// Number of enemies visible in the handbook (excludes `hideInHandbook`),
    /// i.e. the denominator for a completion ratio.
    pub handbook_total: usize,
    /// Encountered enemies, enriched and sorted by handbook `sortId`.
    pub enemies: Vec<EncounteredEnemy>,
}

async fn resolve_user_id(
    state: &AppState,
    auth: &MaybeAuthUser,
    uid_param: Option<&str>,
) -> Result<Uuid, ApiError> {
    if let Some(uid) = uid_param {
        let profile = find_by_uid(&state.db, uid)
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
    } else {
        let auth = auth.0.as_ref().ok_or(ApiError::Unauthorized)?;
        auth.user_uuid()
    }
}

pub async fn get_encountered_enemies(
    State(state): State<AppState>,
    auth: MaybeAuthUser,
    Query(params): Query<EncounteredEnemiesParams>,
) -> Result<Json<EncounteredEnemiesResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &auth, params.uid.as_deref()).await?;
    let ids = get_user_encountered_enemies(&state.db, user_id).await?;

    let game_data = state.game_data.load();
    let handbook = &game_data.enemies.enemy_data;

    let handbook_total = handbook.values().filter(|e| !e.hide_in_handbook).count();

    let mut enemies_out: Vec<EncounteredEnemy> = ids
        .into_iter()
        .map(|enemy_id| {
            let entry = handbook.get(&enemy_id);
            EncounteredEnemy {
                name: entry.map(|e| e.name.clone()),
                enemy_index: entry.map(|e| e.enemy_index.clone()),
                sort_id: entry.map(|e| e.sort_id),
                enemy_id,
            }
        })
        .collect();

    enemies_out.sort_by(|a, b| {
        a.sort_id
            .is_none()
            .cmp(&b.sort_id.is_none())
            .then_with(|| a.sort_id.cmp(&b.sort_id))
            .then_with(|| a.enemy_id.cmp(&b.enemy_id))
    });

    Ok(Json(EncounteredEnemiesResponse {
        encountered_count: enemies_out.len(),
        handbook_total,
        enemies: enemies_out,
    }))
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CommunityEnemyAverageResponse {
    /// Mean number of distinct enemies encountered across every synced user.
    pub average_encountered: f64,
    /// Number of users in the average's denominator (those with synced enemy data).
    pub user_count: i64,
    /// Visible handbook size, matching the per-user response's denominator.
    pub handbook_total: usize,
    pub computed_at: String,
}

/// Community-wide average enemies-encountered figure, used to draw the
/// "community average" marker on a user's enemy-handbook progress bar.
pub async fn get_community_average(
    State(state): State<AppState>,
) -> Result<Json<CommunityEnemyAverageResponse>, ApiError> {
    let key = CacheKey::CommunityEnemyAverage;
    if let Some(cached) = state.cache.get::<CommunityEnemyAverageResponse>(&key).await {
        return Ok(Json(cached));
    }

    let (user_count, average_encountered) = get_community_average_encountered(&state.db).await?;

    let handbook_total = {
        let game_data = state.game_data.load();
        game_data
            .enemies
            .enemy_data
            .values()
            .filter(|e| !e.hide_in_handbook)
            .count()
    };

    let response = CommunityEnemyAverageResponse {
        average_encountered,
        user_count,
        handbook_total,
        computed_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
    };

    state.cache.set(&key, &response).await;
    Ok(Json(response))
}
