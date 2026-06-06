use axum::Json;
use axum::extract::{Path, State};
use serde::Deserialize;
use uuid::Uuid;

use crate::app::error::ApiError;
use crate::app::extractors::auth::AuthUser;
use crate::app::routes::ok_status;
use crate::app::services::tier_list::find_and_authorize;
use crate::app::state::AppState;
use crate::app::validation::{
    TIER_DESCRIPTION_MAX, TIER_NAME_MAX, validate_hex_color, validate_length, validate_opt_length,
};
use crate::core::auth::permissions::Permission;
use crate::database::models::tier_list::Tier;
use crate::database::queries::tier_lists::create_tier;
use crate::database::queries::tier_lists::delete_tier;
use crate::database::queries::tier_lists::update_tier;

#[derive(Deserialize)]
pub struct CreateTierRequest {
    pub name: String,
    pub display_order: i16,
    pub color: Option<String>,
    pub description: Option<String>,
}

fn validate_tier_body(body: &CreateTierRequest) -> Result<(), ApiError> {
    validate_length("tier name", &body.name, TIER_NAME_MAX)?;
    validate_opt_length(
        "tier description",
        body.description.as_deref(),
        TIER_DESCRIPTION_MAX,
    )?;
    validate_hex_color(body.color.as_deref())?;
    Ok(())
}

pub async fn create(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(slug): Path<String>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    validate_tier_body(&body)?;

    let user_id: Uuid = auth.user_uuid()?;
    let list = find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let tier = create_tier(
        &state.db,
        list.id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(tier))
}

pub async fn update(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
    Json(body): Json<CreateTierRequest>,
) -> Result<Json<Tier>, ApiError> {
    validate_tier_body(&body)?;

    let user_id: Uuid = auth.user_uuid()?;
    find_and_authorize(&state, &slug, user_id, auth.role, Permission::Edit).await?;

    let tier = update_tier(
        &state.db,
        tier_id,
        &body.name,
        body.display_order,
        body.color.as_deref(),
        body.description.as_deref(),
    )
    .await?;
    Ok(Json(tier))
}

pub async fn delete(
    State(state): State<AppState>,
    auth: AuthUser,
    Path((slug, tier_id)): Path<(String, Uuid)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let user_id: Uuid = auth.user_uuid()?;
    find_and_authorize(&state, &slug, user_id, auth.role, Permission::Admin).await?;

    delete_tier(&state.db, tier_id).await?;
    Ok(ok_status())
}
