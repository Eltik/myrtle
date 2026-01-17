//! S3 Storage Module for Myrtle Backend
//!
//! Provides S3-compatible storage integration for serving game assets.
//! Supports MinIO, Cloudflare R2, Backblaze B2, AWS S3, and other compatible services.
//!
//! # Architecture
//! ```text
//! Client Request → Backend → Asset Source (Local or S3) → Response
//! ```
//!
//! The module supports three modes:
//! - Local only: Assets served from local filesystem (default)
//! - S3 only: Assets served from S3 bucket
//! - Hybrid: Try local first, fall back to S3

mod client;
mod config;

pub use client::S3AssetClient;
pub use config::S3Config;

use axum::body::Body;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::Response;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

/// Asset source mode
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AssetMode {
    /// Use local filesystem only
    #[default]
    LocalOnly,
    /// Use S3 storage only
    S3Only,
    /// Try local first, fall back to S3
    LocalWithS3Fallback,
}

impl AssetMode {
    /// Parse from environment variable string
    pub fn from_env_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "s3" | "s3only" | "s3_only" => Self::S3Only,
            "hybrid" | "fallback" | "local_s3" => Self::LocalWithS3Fallback,
            _ => Self::LocalOnly,
        }
    }
}

/// Unified asset source that can fetch from local filesystem or S3
#[derive(Clone)]
pub struct AssetSource {
    /// Local assets directory
    local_dir: PathBuf,
    /// Optional S3 client
    s3_client: Option<Arc<S3AssetClient>>,
    /// Asset serving mode
    mode: AssetMode,
}

impl AssetSource {
    /// Create a new asset source with local directory only
    pub fn local_only(local_dir: PathBuf) -> Self {
        Self {
            local_dir,
            s3_client: None,
            mode: AssetMode::LocalOnly,
        }
    }

    /// Create a new asset source with S3 client
    pub fn with_s3(local_dir: PathBuf, s3_client: S3AssetClient, mode: AssetMode) -> Self {
        Self {
            local_dir,
            s3_client: Some(Arc::new(s3_client)),
            mode,
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Result<Self, String> {
        let local_dir = std::env::var("ASSETS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("assets"));

        let mode = std::env::var("ASSET_MODE")
            .map(|s| AssetMode::from_env_str(&s))
            .unwrap_or(AssetMode::LocalOnly);

        // If mode requires S3, try to create client
        if mode != AssetMode::LocalOnly {
            match S3Config::from_env() {
                Ok(config) => match S3AssetClient::new(config) {
                    Ok(client) => Ok(Self::with_s3(local_dir, client, mode)),
                    Err(e) => {
                        log::warn!("Failed to create S3 client, falling back to local: {}", e);
                        Ok(Self::local_only(local_dir))
                    }
                },
                Err(e) => {
                    log::warn!("S3 config not available, using local only: {}", e);
                    Ok(Self::local_only(local_dir))
                }
            }
        } else {
            Ok(Self::local_only(local_dir))
        }
    }

    /// Get the local directory path
    pub fn local_dir(&self) -> &Path {
        &self.local_dir
    }

    /// Check if S3 is available
    pub fn has_s3(&self) -> bool {
        self.s3_client.is_some()
    }

    /// Get the asset mode
    pub fn mode(&self) -> AssetMode {
        self.mode
    }

    /// Serve an asset file, returning an HTTP response
    /// The asset_path should be relative (e.g., "upk/spritepack/ui_char_avatar_0/char_002_amiya.png")
    pub async fn serve_asset(
        &self,
        asset_path: &str,
        headers: &HeaderMap,
        content_type: Option<&str>,
    ) -> Result<Response, StatusCode> {
        match self.mode {
            AssetMode::LocalOnly => self.serve_local(asset_path, headers, content_type).await,
            AssetMode::S3Only => self.serve_s3(asset_path, headers, content_type).await,
            AssetMode::LocalWithS3Fallback => {
                // Try local first
                match self.serve_local(asset_path, headers, content_type).await {
                    Ok(response) => Ok(response),
                    Err(StatusCode::NOT_FOUND) => {
                        // Fall back to S3
                        self.serve_s3(asset_path, headers, content_type).await
                    }
                    Err(e) => Err(e),
                }
            }
        }
    }

    /// Serve asset from local filesystem
    async fn serve_local(
        &self,
        asset_path: &str,
        headers: &HeaderMap,
        content_type: Option<&str>,
    ) -> Result<Response, StatusCode> {
        let file_path = self.local_dir.join(asset_path.trim_start_matches('/'));

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

        // Determine content type
        let mime_type = content_type.map(String::from).unwrap_or_else(|| {
            mime_guess::from_path(&file_path)
                .first_or_octet_stream()
                .to_string()
        });

        // Stream file
        let file = File::open(&file_path)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, mime_type)
            .header(header::CONTENT_LENGTH, metadata.len())
            .header(header::CACHE_CONTROL, "public, max-age=86400")
            .header(header::ETAG, etag)
            .body(body)
            .unwrap())
    }

    /// Serve asset from S3
    async fn serve_s3(
        &self,
        asset_path: &str,
        headers: &HeaderMap,
        content_type: Option<&str>,
    ) -> Result<Response, StatusCode> {
        let s3_client = self.s3_client.as_ref().ok_or(StatusCode::NOT_FOUND)?;

        // Normalize the key (remove leading slash)
        let key = asset_path.trim_start_matches('/');

        // Try to get object metadata first for ETag support
        match s3_client.get_object_with_etag(key).await {
            Ok((data, etag, s3_content_type)) => {
                // Check If-None-Match
                if let Some(etag_ref) = &etag
                    && let Some(if_none_match) = headers.get(header::IF_NONE_MATCH)
                    && if_none_match.to_str().ok() == Some(etag_ref)
                {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_MODIFIED)
                        .header(header::CACHE_CONTROL, "public, max-age=86400")
                        .body(Body::empty())
                        .unwrap());
                }

                // Determine content type
                let mime_type = content_type
                    .map(String::from)
                    .or(s3_content_type)
                    .unwrap_or_else(|| {
                        mime_guess::from_path(key)
                            .first_or_octet_stream()
                            .to_string()
                    });

                let mut builder = Response::builder()
                    .status(StatusCode::OK)
                    .header(header::CONTENT_TYPE, mime_type)
                    .header(header::CONTENT_LENGTH, data.len())
                    .header(header::CACHE_CONTROL, "public, max-age=86400");

                if let Some(etag_val) = etag {
                    builder = builder.header(header::ETAG, etag_val);
                }

                Ok(builder.body(Body::from(data)).unwrap())
            }
            Err(_) => Err(StatusCode::NOT_FOUND),
        }
    }
}
