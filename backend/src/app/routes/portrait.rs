use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use crate::app::state::AppState;

/// Serves portrait images by looking up the correct pack directory from asset mappings.
/// This handles the case where portraits are spread across multiple pack directories
/// (pack0 through pack12).
///
/// Supports local filesystem, S3, or hybrid mode based on ASSET_MODE configuration.
pub async fn serve_portrait(
    State(state): State<AppState>,
    AxumPath(char_id): AxumPath<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // URL-decode the char_id to handle special characters like %23 -> #
    let decoded_char_id = urlencoding::decode(&char_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_owned();

    // Get the correct portrait path from asset mappings
    let portrait_path = state
        .game_data
        .asset_mappings
        .get_portrait_path(&decoded_char_id)
        .ok_or(StatusCode::NOT_FOUND)?;

    // Use the asset source to serve the file (supports local, S3, or hybrid)
    state
        .asset_source
        .serve_asset(&portrait_path, &headers, Some("image/png"))
        .await
}
