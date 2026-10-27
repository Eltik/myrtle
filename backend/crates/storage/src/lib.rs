//! S3-compatible object storage for the asset roots the browser loads.

pub mod config;
pub mod evict;
pub mod key;
pub mod providers;
pub mod restore;
pub mod store;
pub mod sync;

use std::sync::Arc;

use crate::providers::local::LocalStore;
use crate::providers::s3::S3Store;

pub use config::{ConfigError, DeliveryMode, StorageConfig, StorageKind};
pub use evict::EvictPolicy;
pub use store::{GetOptions, GetResult, ObjectMeta, ObjectStore, PutBody, PutObject, StorageError};
pub use sync::Manifest;

/// The store `config` describes.
///
/// # Errors
/// A `cloudflare` config without its endpoint or credentials.
pub fn open_store(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>, StorageError> {
    match config.kind {
        StorageKind::Local => Ok(Arc::new(LocalStore::new(
            config.local_dir.clone().unwrap_or_default(),
        ))),
        StorageKind::Cloudflare => Ok(Arc::new(S3Store::new(config)?)),
    }
}

#[must_use]
pub fn delivery_log_line(mode: DeliveryMode, storage: Option<&StorageConfig>) -> String {
    let none = || "none".to_owned();
    storage.map_or_else(
        || format!(
            "asset delivery: {mode} (STORAGE_TYPE=off, bucket=none, prefix=none, public=none, roots=none)"
        ),
        |c| format!(
            "asset delivery: {mode} (STORAGE_TYPE={}, bucket={}, prefix={}, public={}, roots={})",
            c.kind,
            if c.bucket.is_empty() {
                none()
            } else {
                c.bucket.clone()
            },
            if c.prefix.is_empty() {
                none()
            } else {
                c.prefix.clone()
            },
            c.public_url.clone().unwrap_or_else(none),
            c.roots.join(","),
        ),
    )
}
