use axum::{
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode},
    response::Response,
};
use std::collections::HashSet;
use std::path::Path;
use std::sync::LazyLock;

use crate::core::s3::AssetSource;

static ALLOWED_EXTENSIONS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // Images
        "jpg", "jpeg", "png", "gif", "webp", "svg", // Audio
        "mp3", "ogg", "wav", // Video
        "mp4", "webm", // Documents
        "txt", "csv", "xml", "json", // Game assets
        "skel", "atlas",
    ])
});

/// Serves static assets from the configured asset source (local, S3, or hybrid).
/// Includes security validation to prevent path traversal attacks.
pub async fn serve_asset(
    State(asset_source): State<AssetSource>,
    AxumPath(asset_path): AxumPath<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // Validate path (prevents traversal attacks)
    let validated_path =
        validate_path(&asset_path, asset_source.local_dir()).ok_or(StatusCode::BAD_REQUEST)?;

    // Check extension whitelist
    if !is_extension_allowed(&validated_path) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Use the asset source to serve the file (supports local, S3, or hybrid)
    asset_source
        .serve_asset(&validated_path, &headers, None)
        .await
}

fn is_extension_allowed(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.contains(ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validates path to prevent traversal attacks
/// Returns the normalized relative path if valid, None if malicious
fn validate_path(asset_path: &str, base_dir: &Path) -> Option<String> {
    // Decode URL encoding
    let decoded = urlencoding::decode(asset_path).ok()?;

    // Remove null bytes
    let sanitized = decoded.replace('\0', "");

    // Check for path traversal attempts
    if sanitized.contains("..") {
        return None;
    }

    // For local mode, verify the path resolves within base_dir
    let requested = base_dir.join(&sanitized);
    if let (Ok(canonical), Ok(base_canonical)) =
        (requested.canonicalize(), base_dir.canonicalize())
        && !canonical.starts_with(&base_canonical) {
            return None; // Path traversal attempt
        }

    // Return the sanitized relative path
    Some(sanitized)
}
