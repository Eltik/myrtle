//! S3 Bridge Module
//!
//! Provides wrapper functions for S3-compatible storage operations.
//! This module allows the unpacker to work with remote S3 storage while
//! keeping the core extraction logic unchanged (operating on local temp files).
//!
//! # Architecture
//! ```text
//! S3 Input Bucket → Download to Temp → Extract (unchanged) → Upload from Temp → S3 Output Bucket
//! ```

use anyhow::{Context, Result};
use futures::StreamExt;
use s3::creds::Credentials;
use s3::{Bucket, Region};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use walkdir::WalkDir;

/// Configuration for S3-compatible storage connection
#[derive(Debug, Clone)]
pub struct S3Config {
    /// S3 endpoint URL (e.g., "https://s3.example.com" or "http://localhost:9000" for MinIO)
    pub endpoint: String,
    /// Bucket name for input files (asset bundles)
    pub input_bucket: String,
    /// Bucket name for output files (extracted assets)
    pub output_bucket: String,
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
    /// - S3_INPUT_BUCKET: Input bucket name
    /// - S3_OUTPUT_BUCKET: Output bucket name
    /// - S3_REGION: Region name (default: "us-east-1")
    /// - S3_ACCESS_KEY: Access key ID
    /// - S3_SECRET_KEY: Secret access key
    /// - S3_PATH_STYLE: Use path-style URLs ("true" or "false", default: "true")
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            endpoint: std::env::var("S3_ENDPOINT")
                .context("S3_ENDPOINT environment variable not set")?,
            input_bucket: std::env::var("S3_INPUT_BUCKET")
                .context("S3_INPUT_BUCKET environment variable not set")?,
            output_bucket: std::env::var("S3_OUTPUT_BUCKET")
                .context("S3_OUTPUT_BUCKET environment variable not set")?,
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
        input_bucket: impl Into<String>,
        output_bucket: impl Into<String>,
        region: impl Into<String>,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
        path_style: bool,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            input_bucket: input_bucket.into(),
            output_bucket: output_bucket.into(),
            region: region.into(),
            access_key: access_key.into(),
            secret_key: secret_key.into(),
            path_style,
        }
    }
}

/// S3 client wrapper for bucket operations
pub struct S3Client {
    input_bucket: Box<Bucket>,
    output_bucket: Box<Bucket>,
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

        let mut input_bucket =
            Bucket::new(&config.input_bucket, region.clone(), credentials.clone())?;
        let mut output_bucket = Bucket::new(&config.output_bucket, region, credentials)?;

        if config.path_style {
            input_bucket = input_bucket.with_path_style();
            output_bucket = output_bucket.with_path_style();
        }

        Ok(Self {
            input_bucket: Box::new(input_bucket),
            output_bucket: Box::new(output_bucket),
            config,
        })
    }

    /// Create a new S3 client from environment variables
    pub fn from_env() -> Result<Self> {
        Self::new(S3Config::from_env()?)
    }

    /// Get the input bucket reference
    pub fn input_bucket(&self) -> &Bucket {
        &self.input_bucket
    }

    /// Get the output bucket reference
    pub fn output_bucket(&self) -> &Bucket {
        &self.output_bucket
    }

    /// List objects in the input bucket with optional prefix
    pub async fn list_input_objects(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        self.list_objects(&self.input_bucket, prefix).await
    }

    /// List objects in the output bucket with optional prefix
    pub async fn list_output_objects(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        self.list_objects(&self.output_bucket, prefix).await
    }

    async fn list_objects(&self, bucket: &Bucket, prefix: Option<&str>) -> Result<Vec<String>> {
        let prefix = prefix.unwrap_or("");
        let mut keys = Vec::new();
        let mut continuation_token: Option<String> = None;

        loop {
            let result = bucket
                .list(prefix.to_string(), None)
                .await
                .context("Failed to list S3 objects")?;

            for list_result in &result {
                for object in &list_result.contents {
                    keys.push(object.key.clone());
                }
                continuation_token = list_result.next_continuation_token.clone();
            }

            if continuation_token.is_none() {
                break;
            }
        }

        Ok(keys)
    }

    /// Download a single file from input bucket to local path
    pub async fn download_file(&self, key: &str, local_path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = local_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        let response = self
            .input_bucket
            .get_object(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", key, e))?;

        let mut file = fs::File::create(local_path).await?;
        file.write_all(response.bytes()).await?;

        Ok(())
    }

    /// Download a single file from input bucket to memory
    pub async fn download_to_memory(&self, key: &str) -> Result<Vec<u8>> {
        let response = self
            .input_bucket
            .get_object(key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to download {}: {}", key, e))?;

        Ok(response.bytes().to_vec())
    }

    /// Upload a single file from local path to output bucket
    pub async fn upload_file(&self, local_path: &Path, key: &str) -> Result<()> {
        let content = fs::read(local_path).await?;
        self.upload_bytes(&content, key).await
    }

    /// Upload bytes directly to output bucket
    pub async fn upload_bytes(&self, content: &[u8], key: &str) -> Result<()> {
        self.output_bucket
            .put_object(key, content)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to upload {}: {}", key, e))?;

        Ok(())
    }

    /// Check if an object exists in the input bucket
    pub async fn input_exists(&self, key: &str) -> Result<bool> {
        self.object_exists(&self.input_bucket, key).await
    }

    /// Check if an object exists in the output bucket
    pub async fn output_exists(&self, key: &str) -> Result<bool> {
        self.object_exists(&self.output_bucket, key).await
    }

    async fn object_exists(&self, bucket: &Bucket, key: &str) -> Result<bool> {
        match bucket.head_object(key).await {
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

    /// Delete an object from the output bucket
    pub async fn delete_output_object(&self, key: &str) -> Result<()> {
        self.output_bucket
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

    /// Download multiple files in parallel to a temp directory
    /// Returns the temp directory and a mapping of keys to local paths
    pub async fn download_batch(
        &self,
        keys: &[String],
        temp_dir: &Path,
    ) -> Result<Vec<(String, PathBuf)>> {
        let downloads: Vec<_> = keys
            .iter()
            .map(|key| {
                let client = Arc::clone(&self.client);
                let key = key.clone();
                let local_path = temp_dir.join(&key);

                async move {
                    client.download_file(&key, &local_path).await?;
                    Ok::<_, anyhow::Error>((key, local_path))
                }
            })
            .collect();

        // Process with limited concurrency
        let results: Vec<Result<(String, PathBuf)>> = futures::stream::iter(downloads)
            .buffer_unordered(self.concurrency)
            .collect()
            .await;

        results.into_iter().collect()
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
            "Uploaded {}/{} files ({} bytes, {} failed)",
            self.uploaded_files, self.total_files, self.total_bytes, self.failed_files
        )
    }
}

/// High-level workflow for processing S3 assets
///
/// This struct manages the complete workflow:
/// 1. Download asset bundles from S3
/// 2. Process them using the existing local extraction logic
/// 3. Upload results back to S3
pub struct S3Workflow {
    client: Arc<S3Client>,
    batch_ops: S3BatchOperations,
}

impl S3Workflow {
    /// Create a new S3 workflow handler
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

    /// Process a single asset bundle from S3
    ///
    /// Downloads the file, processes it with the provided function, and uploads results.
    pub async fn process_single<F>(
        &self,
        input_key: &str,
        output_prefix: &str,
        process_fn: F,
    ) -> Result<UploadStats>
    where
        F: FnOnce(&Path, &Path) -> Result<()>,
    {
        // Create temp directories for input and output
        let temp_input = TempDir::new()?;
        let temp_output = TempDir::new()?;

        // Download the input file
        let input_filename = Path::new(input_key)
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| input_key.replace('/', "_"));
        let local_input = temp_input.path().join(&input_filename);

        println!("Downloading: {}", input_key);
        self.client.download_file(input_key, &local_input).await?;

        // Process using the provided function
        println!("Processing: {}", input_filename);
        process_fn(&local_input, temp_output.path())?;

        // Upload results
        println!("Uploading results to: {}", output_prefix);
        let stats = self
            .batch_ops
            .upload_directory(temp_output.path(), output_prefix)
            .await?;

        println!("{}", stats);
        Ok(stats)
    }

    /// Process multiple asset bundles from S3
    ///
    /// For each file: download → process → upload, then clean up temp files.
    /// Files are processed sequentially to manage memory, but uploads are parallelized.
    pub async fn process_batch<F>(
        &self,
        input_keys: &[String],
        output_prefix: &str,
        mut process_fn: F,
    ) -> Result<UploadStats>
    where
        F: FnMut(&Path, &Path) -> Result<()>,
    {
        let mut total_stats = UploadStats::default();

        for (i, input_key) in input_keys.iter().enumerate() {
            println!("[{}/{}] Processing: {}", i + 1, input_keys.len(), input_key);

            // Create temp directories for this file
            let temp_input = TempDir::new()?;
            let temp_output = TempDir::new()?;

            // Download
            let input_filename = Path::new(input_key)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| input_key.replace('/', "_"));
            let local_input = temp_input.path().join(&input_filename);

            if let Err(e) = self.client.download_file(input_key, &local_input).await {
                log::error!("Failed to download {}: {}", input_key, e);
                continue;
            }

            // Process
            if let Err(e) = process_fn(&local_input, temp_output.path()) {
                log::error!("Failed to process {}: {}", input_key, e);
                continue;
            }

            // Upload
            match self
                .batch_ops
                .upload_directory(temp_output.path(), output_prefix)
                .await
            {
                Ok(stats) => {
                    total_stats.total_files += stats.total_files;
                    total_stats.uploaded_files += stats.uploaded_files;
                    total_stats.failed_files += stats.failed_files;
                    total_stats.total_bytes += stats.total_bytes;
                }
                Err(e) => {
                    log::error!("Failed to upload results for {}: {}", input_key, e);
                }
            }

            // Temp directories are automatically cleaned up when dropped
        }

        Ok(total_stats)
    }

    /// List all asset bundles in the input bucket with optional prefix filter
    pub async fn list_input_bundles(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let all_keys = self.client.list_input_objects(prefix).await?;

        // Filter to only .ab files (asset bundles)
        let ab_files: Vec<String> = all_keys
            .into_iter()
            .filter(|k| k.ends_with(".ab"))
            .collect();

        Ok(ab_files)
    }
}

/// Extract assets from S3 using the existing resolve_ab logic
///
/// This is the main entry point for S3-based extraction.
pub async fn extract_from_s3(
    config: S3Config,
    input_prefix: Option<&str>,
    output_prefix: &str,
    options: ExtractOptions,
) -> Result<UploadStats> {
    let workflow = S3Workflow::new(S3Client::new(config)?, options.concurrency);

    // List all asset bundles
    let bundles = workflow.list_input_bundles(input_prefix).await?;
    println!("Found {} asset bundles to process", bundles.len());

    if bundles.is_empty() {
        return Ok(UploadStats::default());
    }

    // Process each bundle
    let stats = workflow
        .process_batch(&bundles, output_prefix, |input_path, output_path| {
            // Call the existing extraction logic
            crate::resolve_ab::main(
                input_path,
                output_path,
                false, // delete
                options.extract_images,
                options.extract_text,
                options.extract_audio,
                options.extract_spine,
                options.merge_alpha,
                options.group_by_source,
                options.force,
                1, // threads (sequential for S3 workflow)
                0, // skip_large_mb
            )
        })
        .await?;

    Ok(stats)
}

/// Options for S3 extraction
#[derive(Debug, Clone)]
pub struct ExtractOptions {
    pub extract_images: bool,
    pub extract_text: bool,
    pub extract_audio: bool,
    pub extract_spine: bool,
    pub merge_alpha: bool,
    pub group_by_source: bool,
    pub force: bool,
    pub concurrency: usize,
}

impl Default for ExtractOptions {
    fn default() -> Self {
        Self {
            extract_images: true,
            extract_text: true,
            extract_audio: true,
            extract_spine: true,
            merge_alpha: false,
            group_by_source: true,
            force: false,
            concurrency: 4,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s3_config_from_values() {
        let config = S3Config::new(
            "http://localhost:9000",
            "input-bucket",
            "output-bucket",
            "us-east-1",
            "minioadmin",
            "minioadmin",
            true,
        );

        assert_eq!(config.endpoint, "http://localhost:9000");
        assert_eq!(config.input_bucket, "input-bucket");
        assert_eq!(config.output_bucket, "output-bucket");
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
