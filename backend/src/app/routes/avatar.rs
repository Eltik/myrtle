use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use std::path::PathBuf;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

use crate::app::state::AppState;

/// Serves avatar images by looking up the correct directory from asset mappings.
/// This handles the case where avatars are spread across multiple directories
/// (ui_char_avatar_0 through ui_char_avatar_13).
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

    // Get assets directory
    let assets_dir = std::env::var("ASSETS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("assets"));

    // Build full file path (avatar_path starts with /upk/...)
    let relative_path = avatar_path.trim_start_matches('/');
    let file_path = assets_dir.join(relative_path);

    // Get file metadata
    let metadata = tokio::fs::metadata(&file_path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    // Reject directories
    if metadata.is_dir() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Generate ETag from size + modified time
    let modified = metadata
        .modified()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let etag = format!(
        "\"{}-{}\"",
        metadata.len(),
        modified
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // Check If-None-Match (ETag)
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
        && if_none_match.to_str().ok() == Some(&etag)
    {
        return Ok(Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .body(Body::empty())
            .unwrap());
    }

    // Stream file
    let file = File::open(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .header(header::CONTENT_LENGTH, metadata.len())
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .header(header::ETAG, etag)
        .body(body)
        .unwrap())
}
