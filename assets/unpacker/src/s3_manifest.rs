//! S3 Manifest Module
//!
//! Provides manifest tracking for S3-based extraction workflows.
//! The manifest is stored as a JSON file in S3, tracking which files
//! have been processed and their hashes for incremental updates.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::s3_bridge::S3Client;

/// Default manifest key in S3
pub const DEFAULT_MANIFEST_KEY: &str = ".extraction_manifest.json";

/// Entry for a processed file in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// Hash of the input file (for change detection)
    pub hash: String,
    /// Timestamp when the file was processed
    pub processed_at: DateTime<Utc>,
    /// Number of assets extracted from this file
    pub asset_count: usize,
    /// Size of the input file in bytes
    pub input_size: u64,
    /// Output prefix where extracted files were stored
    pub output_prefix: String,
}

/// S3-stored manifest for tracking extraction progress
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3Manifest {
    /// Version of the manifest format
    pub version: u32,
    /// When the manifest was last updated
    pub last_updated: Option<DateTime<Utc>>,
    /// Map of input keys to their manifest entries
    pub entries: HashMap<String, ManifestEntry>,
    /// Total files processed
    pub total_processed: usize,
    /// Total assets extracted
    pub total_assets: usize,
}

impl S3Manifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            version: 1,
            last_updated: None,
            entries: HashMap::new(),
            total_processed: 0,
            total_assets: 0,
        }
    }

    /// Load manifest from S3
    pub async fn load(client: &S3Client, key: Option<&str>) -> Result<Self> {
        let manifest_key = key.unwrap_or(DEFAULT_MANIFEST_KEY);

        // Check if manifest exists
        if !client.output_exists(manifest_key).await? {
            log::info!("No existing manifest found at {}, creating new one", manifest_key);
            return Ok(Self::new());
        }

        // Download and parse
        let data = client.download_to_memory(manifest_key).await
            .context("Failed to download manifest from S3")?;

        let manifest: S3Manifest = serde_json::from_slice(&data)
            .context("Failed to parse manifest JSON")?;

        log::info!(
            "Loaded manifest with {} entries ({} total assets)",
            manifest.entries.len(),
            manifest.total_assets
        );

        Ok(manifest)
    }

    /// Save manifest to S3
    pub async fn save(&self, client: &S3Client, key: Option<&str>) -> Result<()> {
        let manifest_key = key.unwrap_or(DEFAULT_MANIFEST_KEY);

        let mut manifest = self.clone();
        manifest.last_updated = Some(Utc::now());

        let json = serde_json::to_vec_pretty(&manifest)
            .context("Failed to serialize manifest")?;

        client.upload_bytes(&json, manifest_key).await
            .context("Failed to upload manifest to S3")?;

        log::info!("Saved manifest to {}", manifest_key);
        Ok(())
    }

    /// Check if a file needs processing based on its hash
    pub fn needs_processing(&self, key: &str, current_hash: &str) -> bool {
        match self.entries.get(key) {
            Some(entry) => entry.hash != current_hash,
            None => true,
        }
    }

    /// Record that a file was processed
    pub fn record_processed(
        &mut self,
        key: String,
        hash: String,
        asset_count: usize,
        input_size: u64,
        output_prefix: String,
    ) {
        let entry = ManifestEntry {
            hash,
            processed_at: Utc::now(),
            asset_count,
            input_size,
            output_prefix,
        };

        // Update totals
        if let Some(old_entry) = self.entries.get(&key) {
            self.total_assets = self.total_assets.saturating_sub(old_entry.asset_count);
        } else {
            self.total_processed += 1;
        }
        self.total_assets += asset_count;

        self.entries.insert(key, entry);
    }

    /// Remove an entry from the manifest
    pub fn remove_entry(&mut self, key: &str) -> Option<ManifestEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.total_processed = self.total_processed.saturating_sub(1);
            self.total_assets = self.total_assets.saturating_sub(entry.asset_count);
            Some(entry)
        } else {
            None
        }
    }

    /// Get entries that haven't been updated since a certain time
    pub fn stale_entries(&self, since: DateTime<Utc>) -> Vec<(&String, &ManifestEntry)> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.processed_at < since)
            .collect()
    }

    /// Get statistics about the manifest
    pub fn stats(&self) -> ManifestStats {
        ManifestStats {
            total_entries: self.entries.len(),
            total_assets: self.total_assets,
            total_input_bytes: self.entries.values().map(|e| e.input_size).sum(),
            oldest_entry: self.entries.values().map(|e| e.processed_at).min(),
            newest_entry: self.entries.values().map(|e| e.processed_at).max(),
        }
    }
}

/// Statistics about the manifest
#[derive(Debug)]
pub struct ManifestStats {
    pub total_entries: usize,
    pub total_assets: usize,
    pub total_input_bytes: u64,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

impl std::fmt::Display for ManifestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Manifest Statistics:")?;
        writeln!(f, "  Entries: {}", self.total_entries)?;
        writeln!(f, "  Total Assets: {}", self.total_assets)?;
        writeln!(
            f,
            "  Total Input Size: {:.2} MB",
            self.total_input_bytes as f64 / (1024.0 * 1024.0)
        )?;
        if let Some(oldest) = self.oldest_entry {
            writeln!(f, "  Oldest Entry: {}", oldest.format("%Y-%m-%d %H:%M:%S UTC"))?;
        }
        if let Some(newest) = self.newest_entry {
            writeln!(f, "  Newest Entry: {}", newest.format("%Y-%m-%d %H:%M:%S UTC"))?;
        }
        Ok(())
    }
}

/// Helper to compute a simple hash from file metadata
/// Uses size + modification time as a proxy for content hash
/// (Actual content hashing would require downloading the entire file)
pub fn compute_metadata_hash(size: u64, etag: Option<&str>) -> String {
    match etag {
        Some(etag) => format!("{}:{}", size, etag.trim_matches('"')),
        None => format!("{}:unknown", size),
    }
}

/// Workflow helper that combines S3 operations with manifest tracking
pub struct TrackedS3Workflow {
    workflow: crate::s3_bridge::S3Workflow,
    manifest: S3Manifest,
    manifest_key: String,
    save_interval: usize,
    processed_since_save: usize,
}

impl TrackedS3Workflow {
    /// Create a new tracked workflow
    pub async fn new(
        client: crate::s3_bridge::S3Client,
        concurrency: usize,
        manifest_key: Option<&str>,
    ) -> Result<Self> {
        let workflow = crate::s3_bridge::S3Workflow::new(client, concurrency);
        let manifest_key = manifest_key.unwrap_or(DEFAULT_MANIFEST_KEY).to_string();

        let manifest = S3Manifest::load(workflow.client(), Some(&manifest_key)).await?;

        Ok(Self {
            workflow,
            manifest,
            manifest_key,
            save_interval: 50, // Save manifest every 50 files
            processed_since_save: 0,
        })
    }

    /// Get the underlying workflow
    pub fn workflow(&self) -> &crate::s3_bridge::S3Workflow {
        &self.workflow
    }

    /// Get the manifest
    pub fn manifest(&self) -> &S3Manifest {
        &self.manifest
    }

    /// Process files with manifest tracking
    pub async fn process_with_tracking<F>(
        &mut self,
        input_keys: &[String],
        output_prefix: &str,
        force: bool,
        mut process_fn: F,
    ) -> Result<crate::s3_bridge::UploadStats>
    where
        F: FnMut(&std::path::Path, &std::path::Path) -> Result<usize>,
    {
        let mut total_stats = crate::s3_bridge::UploadStats::default();

        // Filter to files that need processing (unless force is set)
        let files_to_process: Vec<_> = if force {
            input_keys.to_vec()
        } else {
            // For now, process all files since we don't have S3 metadata easily available
            // In a full implementation, we'd fetch HEAD for each object to get size/etag
            input_keys.to_vec()
        };

        println!(
            "Processing {} files ({} total, force={})",
            files_to_process.len(),
            input_keys.len(),
            force
        );

        for (i, input_key) in files_to_process.iter().enumerate() {
            println!(
                "[{}/{}] Processing: {}",
                i + 1,
                files_to_process.len(),
                input_key
            );

            // Create temp directories
            let temp_input = tempfile::TempDir::new()?;
            let temp_output = tempfile::TempDir::new()?;

            // Download
            let input_filename = std::path::Path::new(input_key)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| input_key.replace('/', "_"));
            let local_input = temp_input.path().join(&input_filename);

            if let Err(e) = self
                .workflow
                .client()
                .download_file(input_key, &local_input)
                .await
            {
                log::error!("Failed to download {}: {}", input_key, e);
                continue;
            }

            // Get file size for manifest
            let input_size = std::fs::metadata(&local_input)
                .map(|m| m.len())
                .unwrap_or(0);

            // Process and count assets
            let asset_count = match process_fn(&local_input, temp_output.path()) {
                Ok(count) => count,
                Err(e) => {
                    log::error!("Failed to process {}: {}", input_key, e);
                    continue;
                }
            };

            // Upload results
            match self
                .workflow
                .batch_ops()
                .upload_directory(temp_output.path(), output_prefix)
                .await
            {
                Ok(stats) => {
                    total_stats.total_files += stats.total_files;
                    total_stats.uploaded_files += stats.uploaded_files;
                    total_stats.failed_files += stats.failed_files;
                    total_stats.total_bytes += stats.total_bytes;

                    // Record in manifest
                    self.manifest.record_processed(
                        input_key.clone(),
                        compute_metadata_hash(input_size, None),
                        asset_count,
                        input_size,
                        output_prefix.to_string(),
                    );
                }
                Err(e) => {
                    log::error!("Failed to upload results for {}: {}", input_key, e);
                    continue;
                }
            }

            // Periodic manifest save
            self.processed_since_save += 1;
            if self.processed_since_save >= self.save_interval {
                if let Err(e) = self
                    .manifest
                    .save(self.workflow.client(), Some(&self.manifest_key))
                    .await
                {
                    log::warn!("Failed to save manifest: {}", e);
                } else {
                    self.processed_since_save = 0;
                }
            }
        }

        // Final manifest save
        self.manifest
            .save(self.workflow.client(), Some(&self.manifest_key))
            .await?;

        Ok(total_stats)
    }

    /// Print manifest statistics
    pub fn print_stats(&self) {
        println!("{}", self.manifest.stats());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_new() {
        let manifest = S3Manifest::new();
        assert_eq!(manifest.version, 1);
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn test_manifest_record_processed() {
        let mut manifest = S3Manifest::new();

        manifest.record_processed(
            "test/file.ab".to_string(),
            "abc123".to_string(),
            10,
            1024,
            "output/".to_string(),
        );

        assert_eq!(manifest.total_processed, 1);
        assert_eq!(manifest.total_assets, 10);
        assert!(manifest.entries.contains_key("test/file.ab"));
    }

    #[test]
    fn test_manifest_needs_processing() {
        let mut manifest = S3Manifest::new();

        // New file should need processing
        assert!(manifest.needs_processing("new/file.ab", "hash1"));

        // Record it
        manifest.record_processed(
            "new/file.ab".to_string(),
            "hash1".to_string(),
            5,
            512,
            "output/".to_string(),
        );

        // Same hash should not need processing
        assert!(!manifest.needs_processing("new/file.ab", "hash1"));

        // Different hash should need processing
        assert!(manifest.needs_processing("new/file.ab", "hash2"));
    }

    #[test]
    fn test_compute_metadata_hash() {
        let hash1 = compute_metadata_hash(1024, Some("\"abc123\""));
        assert_eq!(hash1, "1024:abc123");

        let hash2 = compute_metadata_hash(1024, None);
        assert_eq!(hash2, "1024:unknown");
    }
}
