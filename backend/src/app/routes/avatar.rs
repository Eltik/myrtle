use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};

use crate::app::state::AppState;

/// Serves avatar images by looking up the correct directory from asset mappings.
/// This handles the case where avatars are spread across multiple directories
/// (ui_char_avatar_0 through ui_char_avatar_13).
///
/// Supports local filesystem, S3, or hybrid mode based on ASSET_MODE configuration.
pub async fn serve_avatar(
    State(state): State<AppState>,
    AxumPath(avatar_id): AxumPath<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // URL-decode the avatar_id to handle special characters like %23 -> #
    let decoded_avatar_id = urlencoding::decode(&avatar_id)
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .into_owned();

    // Get the correct avatar path from asset mappings
    let avatar_path = state
        .game_data
        .asset_mappings
        .get_avatar_path(&decoded_avatar_id);

    // Use the asset source to serve the file (supports local, S3, or hybrid)
    state
        .asset_source
        .serve_asset(&avatar_path, &headers, Some("image/png"))
        .await
}
