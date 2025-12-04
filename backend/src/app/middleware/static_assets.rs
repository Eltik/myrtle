use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

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

pub async fn serve_asset(
    State(base_dir): State<PathBuf>,
    AxumPath(asset_path): AxumPath<String>,
    headers: HeaderMap,
) -> Result<Response, StatusCode> {
    // Validate path (prevents traversal attacks)
    let file_path =
        validate_and_normalize_path(&asset_path, &base_dir).ok_or(StatusCode::BAD_REQUEST)?;

    // Check extension whitelist
    if !is_extension_allowed(&file_path) {
        return Err(StatusCode::FORBIDDEN);
    }

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
    if let Some(if_none_match) = headers.get(header::IF_NONE_MATCH) {
        if if_none_match.to_str().ok() == Some(&etag) {
            return Ok(Response::builder()
                .status(StatusCode::NOT_MODIFIED)
                .header(header::CACHE_CONTROL, "public, max-age=86400")
                .body(Body::empty())
                .unwrap());
        }
    }

    // Guess MIME type
    let content_type = mime_guess::from_path(&file_path)
        .first_or_octet_stream()
        .to_string();

    // Stream file (memory efficient for large files)
    let file = File::open(&file_path)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, metadata.len())
        .header(header::CACHE_CONTROL, "public, max-age=86400")
        .header(header::ETAG, etag)
        .body(body)
        .unwrap())
}

fn is_extension_allowed(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ALLOWED_EXTENSIONS.contains(ext.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Validates path to prevent traversal attacks
/// Returns None if path is malicious
fn validate_and_normalize_path(asset_path: &str, base_dir: &Path) -> Option<PathBuf> {
    // Decode URL encoding
    let decoded = urlencoding::decode(asset_path).ok()?;

    // Remove null bytes
    let sanitized = decoded.replace('\0', "");

    // Build path and canonicalize to resolve any ../ tricks
    let requested = base_dir.join(&sanitized);

    // Canonicalize resolves symlinks and ../ - this is the key security check
    let canonical = requested.canonicalize().ok()?;
    let base_canonical = base_dir.canonicalize().ok()?;

    // Ensure the resolved path is still within base_dir
    if canonical.starts_with(&base_canonical) {
        Some(canonical)
    } else {
        None // Path traversal attempt
    }
}
