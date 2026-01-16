//! S3 Bridge Module for Arknights Downloader
//!
//! Provides wrapper functions for S3-compatible storage operations.
//! This module allows the downloader to upload downloaded assets to S3
//! instead of or in addition to local storage.
//!
//! # Architecture
//! ```text
//! Game CDN → Download to Local/Temp → Upload to S3 Output Bucket
//! ```

use anyhow::{Context, Result};
use futures::StreamExt;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

/// Configuration for S3-compatible storage connection
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 endpoint URL (e.g., "https://s3.example.com" or "http://localhost:9000" for MinIO)
    pub endpoint: String,
    /// Bucket name for output files (downloaded/extracted assets)
    pub bucket: String,
    /// AWS region or region name for S3-compatible service
    pub region: String,
    /// Access key ID
    pub access_key: String,
    /// Secret access key
    pub secret_key: String,
    /// Use path-style URLs (required for MinIO and some S3-compatible services)
    pub path_style: bool,
}

impl S3Config {
    /// Create a new S3 configuration from environment variables
    ///
    /// Expected environment variables:
    /// - S3_ENDPOINT: The S3 endpoint URL
    /// - S3_BUCKET: Output bucket name
    /// - S3_REGION: Region name (default: "us-east-1")
    /// - S3_ACCESS_KEY: Access key ID
    /// - S3_SECRET_KEY: Secret access key
    /// - S3_PATH_STYLE: Use path-style URLs ("true" or "false", default: "true")
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            endpoint: std::env::var("S3_ENDPOINT")
                .context("S3_ENDPOINT environment variable not set")?,
            bucket: std::env::var("S3_BUCKET").context("S3_BUCKET environment variable not set")?,
            region: std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("S3_ACCESS_KEY")
                .context("S3_ACCESS_KEY environment variable not set")?,
            secret_key: std::env::var("S3_SECRET_KEY")
                .context("S3_SECRET_KEY environment variable not set")?,
            path_style: std::env::var("S3_PATH_STYLE")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(true),
        })
    }

    /// Create a new S3 configuration with explicit values
    pub fn new(
        endpoint: impl Into<String>,
        bucket: impl Into<String>,
        region: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        path_style: bool,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            bucket: bucket.into(),
            region: region.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            path_style,
        }
    }
}

/// S3 client wrapper for bucket operations
pub struct S3Client {
    bucket: Box<Bucket>,
    config: S3Config,
}

impl S3Client {
    /// Create a new S3 client from configuration
    pub fn new(config: S3Config) -> Result<Self> {
        let credentials = Credentials::new(
            Some(&config.access_key),
            Some(&config.secret_key),
            None,
            None,
            None,
        )?;

        let region = Region::Custom {
            region: config.region.clone(),
            endpoint: config.endpoint.clone(),
        };

        let mut bucket = Bucket::new(&config.bucket, region, credentials)?;

        if config.path_style {
            bucket = bucket.with_path_style();
        }

        Ok(Self {
            bucket: Box::new(bucket),
            config,
        })
    }

    /// Create a new S3 client from environment variables
    pub fn from_env() -> Result<Self> {
        Self::new(S3Config::from_env()?)
    }

    /// Get the bucket reference
    pub fn bucket(&self) -> &Bucket {
        &self.bucket
    }

    /// List objects in the bucket with optional prefix
    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let prefix = prefix.unwrap_or("");
        let mut keys = Vec::new();

        let result = self
            .bucket
            .list(prefix.to_string(), None)
            .await
            .context("Failed to list S3 objects")?;

        for list_result in &result {
            for object in &list_result.contents {
                keys.push(object.key.clone());
            }
        }

        Ok(keys)
    }

    /// Download a single file from bucket to local path
    pub async fn download_file(&self, key: &str, local_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", key, e))?;

        let mut file = fs::File::create(local_path).await?;
        file.write_all(response.bytes()).await?;

        Ok(())
    }

    /// Download a single file from bucket to memory
    pub async fn download_to_memory(&self, key: &str) -> Result<Vec<u8>> {
        let response = self
            .bucket
            .get_object(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", key, e))?;

        Ok(response.bytes().to_vec())
    }

    /// Upload a single file from local path to bucket
    pub async fn upload_file(&self, local_path: &Path, key: &str) -> Result<()> {
        let content = fs::read(local_path).await?;
        self.upload_bytes(&content, key).await
    }

    /// Upload bytes directly to bucket
    pub async fn upload_bytes(&self, content: &[u8], key: &str) -> Result<()> {
        self.bucket
            .put_object(key, content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload {}: {}", key, e))?;

        Ok(())
    }

    /// Check if an object exists in the bucket
    pub async fn object_exists(&self, key: &str) -> Result<bool> {
        match self.bucket.head_object(key).await {
            Ok(_) => Ok(true),
            Err(e) => {
                // Check if it's a 404 error (object not found)
                let err_str = e.to_string();
                if err_str.contains("404")
                    || err_str.contains("NoSuchKey")
                    || err_str.contains("Not Found")
                {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!(
                        "Failed to check existence of {}: {}",
                        key,
                        e
                    ))
                }
            }
        }
    }

    /// Delete an object from the bucket
    pub async fn delete_object(&self, key: &str) -> Result<()> {
        self.bucket
            .delete_object(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to delete {}: {}", key, e))?;
        Ok(())
    }

    /// Get the configuration
    pub fn config(&self) -> &S3Config {
        &self.config
    }
}

/// Batch operations for efficient S3 transfers
pub struct S3BatchOperations {
    client: Arc<S3Client>,
    concurrency: usize,
}

impl S3BatchOperations {
    /// Create a new batch operations handler
    pub fn new(client: Arc<S3Client>, concurrency: usize) -> Self {
        Self {
            client,
            concurrency: concurrency.max(1),
        }
    }

    /// Upload an entire directory to S3 with the given prefix
    pub async fn upload_directory(&self, local_dir: &Path, prefix: &str) -> Result<UploadStats> {
        let mut stats = UploadStats::default();

        // Collect all files to upload
        let files: Vec<_> = WalkDir::new(local_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .map(|e| e.path().to_path_buf())
            .collect();

        stats.total_files = files.len();

        let uploads: Vec<_> = files
            .into_iter()
            .map(|local_path| {
                let client = Arc::clone(&self.client);
                let base_dir = local_dir.to_path_buf();
                let prefix = prefix.to_string();

                async move {
                    let relative = local_path.strip_prefix(&base_dir).unwrap_or(&local_path);

                    let key = if prefix.is_empty() {
                        relative.to_string_lossy().to_string()
                    } else {
                        format!(
                            "{}/{}",
                            prefix.trim_end_matches('/'),
                            relative.to_string_lossy()
                        )
                    };

                    // Replace backslashes with forward slashes for S3 compatibility
                    let key = key.replace('\\', "/");

                    let content = fs::read(&local_path).await?;
                    let size = content.len();
                    client.upload_bytes(&content, &key).await?;

                    Ok::<_, anyhow::Error>(size)
                }
            })
            .collect();

        let results: Vec<Result<usize>> = futures::stream::iter(uploads)
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        for result in results {
            match result {
                Ok(size) => {
                    stats.uploaded_files += 1;
                    stats.total_bytes += size;
                }
                Err(e) => {
                    stats.failed_files += 1;
                    log::error!("Upload failed: {}", e);
                }
            }
        }

        Ok(stats)
    }

    /// Upload multiple files with a custom key mapping
    pub async fn upload_batch(&self, files: &[(PathBuf, String)]) -> Result<UploadStats> {
        let mut stats = UploadStats {
            total_files: files.len(),
            ..Default::default()
        };

        let uploads: Vec<_> = files
            .iter()
            .map(|(local_path, key)| {
                let client = Arc::clone(&self.client);
                let local_path = local_path.clone();
                let key = key.clone();

                async move {
                    let content = fs::read(&local_path).await?;
                    let size = content.len();
                    client.upload_bytes(&content, &key).await?;
                    Ok::<_, anyhow::Error>(size)
                }
            })
            .collect();

        let results: Vec<Result<usize>> = futures::stream::iter(uploads)
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        for result in results {
            match result {
                Ok(size) => {
                    stats.uploaded_files += 1;
                    stats.total_bytes += size;
                }
                Err(e) => {
                    stats.failed_files += 1;
                    log::error!("Upload failed: {}", e);
                }
            }
        }

        Ok(stats)
    }

    /// Download files from S3 to local directory
    pub async fn download_directory(
        &self,
        prefix: &str,
        local_dir: &Path,
    ) -> Result<DownloadStats> {
        let mut stats = DownloadStats::default();

        // List all objects with prefix
        let keys = self.client.list_objects(Some(prefix)).await?;
        stats.total_files = keys.len();

        let downloads: Vec<_> = keys
            .into_iter()
            .map(|key| {
                let client = Arc::clone(&self.client);
                let local_dir = local_dir.to_path_buf();
                let prefix = prefix.to_string();

                async move {
                    let relative = key
                        .strip_prefix(&prefix)
                        .unwrap_or(&key)
                        .trim_start_matches('/');
                    let local_path = local_dir.join(relative);

                    client.download_file(&key, &local_path).await?;

                    let size = fs::metadata(&local_path).await?.len() as usize;
                    Ok::<_, anyhow::Error>(size)
                }
            })
            .collect();

        let results: Vec<Result<usize>> = futures::stream::iter(downloads)
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        for result in results {
            match result {
                Ok(size) => {
                    stats.downloaded_files += 1;
                    stats.total_bytes += size;
                }
                Err(e) => {
                    stats.failed_files += 1;
                    log::error!("Download failed: {}", e);
                }
            }
        }

        Ok(stats)
    }
}

/// Statistics for batch upload operations
#[derive(Debug, Default)]
pub struct UploadStats {
    pub total_files: usize,
    pub uploaded_files: usize,
    pub failed_files: usize,
    pub total_bytes: usize,
}

impl std::fmt::Display for UploadStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Uploaded {}/{} files ({:.2} MB, {} failed)",
            self.uploaded_files,
            self.total_files,
            self.total_bytes as f64 / (1024.0 * 1024.0),
            self.failed_files
        )
    }
}

/// Statistics for batch download operations
#[derive(Debug, Default)]
pub struct DownloadStats {
    pub total_files: usize,
    pub downloaded_files: usize,
    pub failed_files: usize,
    pub total_bytes: usize,
}

impl std::fmt::Display for DownloadStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Downloaded {}/{} files ({:.2} MB, {} failed)",
            self.downloaded_files,
            self.total_files,
            self.total_bytes as f64 / (1024.0 * 1024.0),
            self.failed_files
        )
    }
}

/// High-level workflow for syncing downloaded assets to S3
pub struct S3SyncWorkflow {
    client: Arc<S3Client>,
    batch_ops: S3BatchOperations,
}

impl S3SyncWorkflow {
    /// Create a new S3 sync workflow handler
    pub fn new(client: S3Client, concurrency: usize) -> Self {
        let client = Arc::new(client);
        let batch_ops = S3BatchOperations::new(Arc::clone(&client), concurrency);
        Self { client, batch_ops }
    }

    /// Create from environment variables
    pub fn from_env(concurrency: usize) -> Result<Self> {
        Ok(Self::new(S3Client::from_env()?, concurrency))
    }

    /// Get the underlying S3 client
    pub fn client(&self) -> &S3Client {
        &self.client
    }

    /// Get the batch operations handler
    pub fn batch_ops(&self) -> &S3BatchOperations {
        &self.batch_ops
    }

    /// Sync a local directory to S3 (upload all files)
    pub async fn sync_to_s3(&self, local_dir: &Path, prefix: &str) -> Result<UploadStats> {
        println!(
            "Syncing {} to S3 prefix '{}'...",
            local_dir.display(),
            prefix
        );
        let stats = self.batch_ops.upload_directory(local_dir, prefix).await?;
        println!("{}", stats);
        Ok(stats)
    }

    /// Sync from S3 to local directory (download all files)
    pub async fn sync_from_s3(&self, prefix: &str, local_dir: &Path) -> Result<DownloadStats> {
        println!(
            "Syncing from S3 prefix '{}' to {}...",
            prefix,
            local_dir.display()
        );
        let stats = self.batch_ops.download_directory(prefix, local_dir).await?;
        println!("{}", stats);
        Ok(stats)
    }

    /// List all objects in S3 with optional prefix
    pub async fn list_objects(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        self.client.list_objects(prefix).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config_from_values() {
        let config = S3Config::new(
            "http://localhost:9000",
            "test-bucket",
            "us-east-1",
            "minioadmin",
            "minioadmin",
            true,
        );

        assert_eq!(config.endpoint, "http://localhost:9000");
        assert_eq!(config.bucket, "test-bucket");
        assert!(config.path_style);
    }

    #[test]
    fn test_upload_stats_display() {
        let stats = UploadStats {
            total_files: 100,
            uploaded_files: 95,
            failed_files: 5,
            total_bytes: 1024 * 1024,
        };

        let display = format!("{}", stats);
        assert!(display.contains("95/100"));
        assert!(display.contains("5 failed"));
    }
}
