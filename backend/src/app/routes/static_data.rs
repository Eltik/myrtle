use axum::Json;
use axum::extract::{Path, State};

use crate::app::services::static_data::get_resource;
use crate::app::{error::ApiError, state::AppState};

pub async fn get_static(
    State(state): State<AppState>,
    Path(resource): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = get_resource(&state, &resource).await?;
    Ok(Json(value))
}
