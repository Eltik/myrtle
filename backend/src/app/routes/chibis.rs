use axum::extract::{Path, State};
use axum::http::HeaderMap;
use axum::response::Response;

use crate::app::routes::static_data::json_response;
use crate::app::services::static_data::get_chibi;
use crate::app::{error::ApiError, state::AppState};
use crate::core::hypergryph::constants::Server;

/// `GET /chibis/{operatorId}` - one operator's chibi catalog entry (default
/// server). Replaces fetching the whole `/static/chibis` catalog to find one.
pub async fn chibi_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(operator_id): Path<String>,
) -> Result<Response, ApiError> {
    let body = get_chibi(&state, state.default_server, &operator_id).await?;
    Ok(json_response(body, &headers))
}

/// `GET /{server}/chibis/{operatorId}` - per-server variant.
pub async fn chibi_detail_srv(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server, operator_id)): Path<(Server, String)>,
) -> Result<Response, ApiError> {
    let body = get_chibi(&state, server, &operator_id).await?;
    Ok(json_response(body, &headers))
}
