//! S3 Manifest Module for Arknights Downloader
//!
//! Provides manifest tracking for S3-based download workflows.
//! The manifest is stored as a JSON file in S3, tracking which files
//! have been uploaded, their hashes, and version information.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::s3_bridge::S3Client;

/// Default manifest key in S3
pub const DEFAULT_MANIFEST_KEY: &str = ".download_manifest.json";

/// Version information for a server
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VersionInfo {
    /// Resource version from game server
    pub res_version: String,
    /// Client version from game server
    pub client_version: String,
    /// When this version was synced
    pub synced_at: Option<DateTime<Utc>>,
}

/// Entry for a synced file in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    /// MD5 hash of the file
    pub md5: String,
    /// Size of the file in bytes
    pub size: u64,
    /// Timestamp when the file was uploaded
    pub uploaded_at: DateTime<Utc>,
    /// Server this file came from
    pub server: String,
}

/// S3-stored manifest for tracking download/sync progress
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct S3DownloadManifest {
    /// Version of the manifest format
    pub version: u32,
    /// When the manifest was last updated
    pub last_updated: Option<DateTime<Utc>>,
    /// Version info per server
    pub server_versions: HashMap<String, VersionInfo>,
    /// Map of S3 keys to their manifest entries
    pub entries: HashMap<String, ManifestEntry>,
    /// Total files synced
    pub total_files: usize,
    /// Total bytes synced
    pub total_bytes: u64,
}

impl S3DownloadManifest {
    /// Create a new empty manifest
    pub fn new() -> Self {
        Self {
            version: 1,
            last_updated: None,
            server_versions: HashMap::new(),
            entries: HashMap::new(),
            total_files: 0,
            total_bytes: 0,
        }
    }

    /// Load manifest from S3
    pub async fn load(client: &S3Client, key: Option<&str>) -> Result<Self> {
        let manifest_key = key.unwrap_or(DEFAULT_MANIFEST_KEY);

        // Check if manifest exists
        if !client.object_exists(manifest_key).await? {
            log::info!(
                "No existing manifest found at {}, creating new one",
                manifest_key
            );
            return Ok(Self::new());
        }

        // Download and parse
        let data = client
            .download_to_memory(manifest_key)
            .await
            .context("Failed to download manifest from S3")?;

        let manifest: S3DownloadManifest =
            serde_json::from_slice(&data).context("Failed to parse manifest JSON")?;

        log::info!(
            "Loaded manifest with {} entries ({} total bytes)",
            manifest.entries.len(),
            manifest.total_bytes
        );

        Ok(manifest)
    }

    /// Save manifest to S3
    pub async fn save(&self, client: &S3Client, key: Option<&str>) -> Result<()> {
        let manifest_key = key.unwrap_or(DEFAULT_MANIFEST_KEY);

        let mut manifest = self.clone();
        manifest.last_updated = Some(Utc::now());

        let json = serde_json::to_vec_pretty(&manifest).context("Failed to serialize manifest")?;

        client
            .upload_bytes(&json, manifest_key)
            .await
            .context("Failed to upload manifest to S3")?;

        log::info!("Saved manifest to {}", manifest_key);
        Ok(())
    }

    /// Update server version info
    pub fn update_server_version(&mut self, server: &str, res_version: &str, client_version: &str) {
        self.server_versions.insert(
            server.to_string(),
            VersionInfo {
                res_version: res_version.to_string(),
                client_version: client_version.to_string(),
                synced_at: Some(Utc::now()),
            },
        );
    }

    /// Get server version info
    pub fn get_server_version(&self, server: &str) -> Option<&VersionInfo> {
        self.server_versions.get(server)
    }

    /// Check if a file needs syncing based on its MD5 hash
    pub fn needs_sync(&self, key: &str, current_md5: &str) -> bool {
        match self.entries.get(key) {
            Some(entry) => entry.md5 != current_md5,
            None => true,
        }
    }

    /// Record that a file was synced
    pub fn record_synced(&mut self, key: String, md5: String, size: u64, server: &str) {
        let entry = ManifestEntry {
            md5,
            size,
            uploaded_at: Utc::now(),
            server: server.to_string(),
        };

        // Update totals
        if let Some(old_entry) = self.entries.get(&key) {
            self.total_bytes = self.total_bytes.saturating_sub(old_entry.size);
        } else {
            self.total_files += 1;
        }
        self.total_bytes += size;

        self.entries.insert(key, entry);
    }

    /// Remove an entry from the manifest
    pub fn remove_entry(&mut self, key: &str) -> Option<ManifestEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.total_files = self.total_files.saturating_sub(1);
            self.total_bytes = self.total_bytes.saturating_sub(entry.size);
            Some(entry)
        } else {
            None
        }
    }

    /// Get entries for a specific server
    pub fn entries_for_server(&self, server: &str) -> Vec<(&String, &ManifestEntry)> {
        self.entries
            .iter()
            .filter(|(_, entry)| entry.server == server)
            .collect()
    }

    /// Get statistics about the manifest
    pub fn stats(&self) -> ManifestStats {
        ManifestStats {
            total_entries: self.entries.len(),
            total_bytes: self.total_bytes,
            servers: self.server_versions.keys().cloned().collect(),
            oldest_entry: self.entries.values().map(|e| e.uploaded_at).min(),
            newest_entry: self.entries.values().map(|e| e.uploaded_at).max(),
        }
    }
}

/// Statistics about the manifest
#[derive(Debug)]
pub struct ManifestStats {
    pub total_entries: usize,
    pub total_bytes: u64,
    pub servers: Vec<String>,
    pub oldest_entry: Option<DateTime<Utc>>,
    pub newest_entry: Option<DateTime<Utc>>,
}

impl std::fmt::Display for ManifestStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Download Manifest Statistics:")?;
        writeln!(f, "  Entries: {}", self.total_entries)?;
        writeln!(
            f,
            "  Total Size: {:.2} MB",
            self.total_bytes as f64 / (1024.0 * 1024.0)
        )?;
        writeln!(f, "  Servers: {}", self.servers.join(", "))?;
        if let Some(oldest) = self.oldest_entry {
            writeln!(
                f,
                "  Oldest Entry: {}",
                oldest.format("%Y-%m-%d %H:%M:%S UTC")
            )?;
        }
        if let Some(newest) = self.newest_entry {
            writeln!(
                f,
                "  Newest Entry: {}",
                newest.format("%Y-%m-%d %H:%M:%S UTC")
            )?;
        }
        Ok(())
    }
}

/// Workflow helper that combines S3 operations with manifest tracking
pub struct TrackedS3Sync {
    workflow: crate::s3_bridge::S3SyncWorkflow,
    manifest: S3DownloadManifest,
    manifest_key: String,
    save_interval: usize,
    synced_since_save: usize,
}

impl TrackedS3Sync {
    /// Create a new tracked sync workflow
    pub async fn new(
        client: crate::s3_bridge::S3Client,
        concurrency: usize,
        manifest_key: Option<&str>,
    ) -> Result<Self> {
        let workflow = crate::s3_bridge::S3SyncWorkflow::new(client, concurrency);
        let manifest_key = manifest_key.unwrap_or(DEFAULT_MANIFEST_KEY).to_string();

        let manifest = S3DownloadManifest::load(workflow.client(), Some(&manifest_key)).await?;

        Ok(Self {
            workflow,
            manifest,
            manifest_key,
            save_interval: 50, // Save manifest every 50 files
            synced_since_save: 0,
        })
    }

    /// Get the underlying workflow
    pub fn workflow(&self) -> &crate::s3_bridge::S3SyncWorkflow {
        &self.workflow
    }

    /// Get the manifest
    pub fn manifest(&self) -> &S3DownloadManifest {
        &self.manifest
    }

    /// Get mutable manifest
    pub fn manifest_mut(&mut self) -> &mut S3DownloadManifest {
        &mut self.manifest
    }

    /// Save the manifest
    pub async fn save_manifest(&self) -> Result<()> {
        self.manifest
            .save(self.workflow.client(), Some(&self.manifest_key))
            .await
    }

    /// Sync a directory to S3 with manifest tracking
    pub async fn sync_directory_tracked(
        &mut self,
        local_dir: &std::path::Path,
        prefix: &str,
        server: &str,
    ) -> Result<crate::s3_bridge::UploadStats> {
        // Upload the directory
        let stats = self.workflow.sync_to_s3(local_dir, prefix).await?;

        // Update manifest with uploaded files
        for entry in walkdir::WalkDir::new(local_dir)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let relative = entry.path().strip_prefix(local_dir).unwrap_or(entry.path());
            let key = if prefix.is_empty() {
                relative.to_string_lossy().to_string()
            } else {
                format!(
                    "{}/{}",
                    prefix.trim_end_matches('/'),
                    relative.to_string_lossy()
                )
            }
            .replace('\\', "/");

            let metadata = std::fs::metadata(entry.path())?;
            let size = metadata.len();

            // Use file size as a simple "hash" for now
            // In production, you might want to compute actual MD5
            let hash = format!("{}", size);

            self.manifest.record_synced(key, hash, size, server);
            self.synced_since_save += 1;

            // Periodic manifest save
            if self.synced_since_save >= self.save_interval {
                self.save_manifest().await?;
                self.synced_since_save = 0;
            }
        }

        // Final manifest save
        self.save_manifest().await?;

        Ok(stats)
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
        let manifest = S3DownloadManifest::new();
        assert_eq!(manifest.version, 1);
        assert!(manifest.entries.is_empty());
    }

    #[test]
    fn test_manifest_record_synced() {
        let mut manifest = S3DownloadManifest::new();

        manifest.record_synced("test/file.ab".to_string(), "abc123".to_string(), 1024, "en");

        assert_eq!(manifest.total_files, 1);
        assert_eq!(manifest.total_bytes, 1024);
        assert!(manifest.entries.contains_key("test/file.ab"));
    }

    #[test]
    fn test_manifest_needs_sync() {
        let mut manifest = S3DownloadManifest::new();

        // New file should need sync
        assert!(manifest.needs_sync("new/file.ab", "hash1"));

        // Record it
        manifest.record_synced("new/file.ab".to_string(), "hash1".to_string(), 512, "en");

        // Same hash should not need sync
        assert!(!manifest.needs_sync("new/file.ab", "hash1"));

        // Different hash should need sync
        assert!(manifest.needs_sync("new/file.ab", "hash2"));
    }

    #[test]
    fn test_version_info() {
        let mut manifest = S3DownloadManifest::new();

        manifest.update_server_version("en", "24-02-09-15-22-33-941721", "1.9.51");

        let version = manifest.get_server_version("en").unwrap();
        assert_eq!(version.res_version, "24-02-09-15-22-33-941721");
        assert_eq!(version.client_version, "1.9.51");
    }
}
