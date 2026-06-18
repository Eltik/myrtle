use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::services::tier_list::find_and_authorize;
use crate::app::services::tier_list::get_by_slug;
use crate::app::state::AppState;
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::TierListVersion;
use crate::database::queries::tier_lists::create_version;
use crate::database::queries::tier_lists::get_versions;
use crate::database::queries::tier_lists::latest_version;

pub async fn list(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Result<Json<Vec<TierListVersion>>, ApiError> {
    let tier_list = super::load_tier_list(&state, &slug).await?;
    let versions = get_versions(&state.db, tier_list.id).await?;
    Ok(Json(versions))
}

#[derive(Deserialize)]
pub struct PublishRequest {
    pub changelog: Option<String>,
}

pub async fn publish(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<PublishRequest>,
) -> Result<Json<TierListVersion>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Publish).await?;

    // Build snapshot from current state
    let detail = get_by_slug(&state, &slug).await?;
    let snapshot = serde_json::to_value(&detail.tiers).map_err(|e| ApiError::Internal(e.into()))?;

    let next_version = latest_version(&state.db, list.id).await?.unwrap_or(0) + 1;

    let version = create_version(
        &state.db,
        list.id,
        next_version,
        &snapshot,
        body.changelog.as_deref(),
        user_id,
    )
    .await?;
    Ok(Json(version))
}
