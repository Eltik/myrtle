use axum::Json;
use axum::extract::{Path, State};

use crate::app::{error::ApiError, services, state::AppState};

pub async fn get_static(
    State(state): State<AppState>,
    Path(resource): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let value = services::static_data::get_resource(&state, &resource).await?;
    Ok(Json(value))
}
