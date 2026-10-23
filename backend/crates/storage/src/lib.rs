//! S3-compatible object storage for the asset roots the browser loads.
//!
//! The local `assets/output/{server}/` tree stays the source of truth; the
//! bucket is a delivery tier that `asset-sync` mirrors after every extract.
//! With `S3_TYPE` unset nothing here is constructed and the backend serves
//! exactly as it did without this crate. The contract (env vars, key layout,
//! CLI, exit codes) is in this crate's `README.md`.

pub mod config;
pub mod evict;
pub mod key;
pub mod providers;
pub mod restore;
pub mod store;
pub mod sync;

use std::sync::Arc;

pub use config::{ConfigError, DeliveryMode, StorageConfig, StorageKind};
pub use evict::EvictPolicy;
pub use store::{GetOptions, GetResult, ObjectMeta, ObjectStore, PutBody, PutObject, StorageError};
pub use sync::Manifest;

/// The store `config` describes. Construction does no I/O, so it runs outside
/// an async context; the AWS default credential chain, when used, is resolved
/// on the first request.
///
/// # Errors
/// A malformed endpoint URL.
pub fn open_store(config: &StorageConfig) -> Result<Arc<dyn ObjectStore>, StorageError> {
    match config.kind {
        StorageKind::Local => Ok(Arc::new(providers::local::LocalStore::new(
            config.local_dir.clone().unwrap_or_default(),
        ))),
        StorageKind::Cloudflare | StorageKind::Aws | StorageKind::S3 => {
            Ok(Arc::new(providers::s3::S3Store::new(config)?))
        }
    }
}

/// The one startup line the backend logs about delivery:
/// `asset delivery: <mode> (S3_TYPE=<kind>, bucket=<b>, prefix=<p>, public=<url|none>, roots=<..>)`.
#[must_use]
pub fn delivery_log_line(mode: DeliveryMode, storage: Option<&StorageConfig>) -> String {
    let none = || "none".to_owned();
    storage.map_or_else(
        || format!(
            "asset delivery: {mode} (S3_TYPE=off, bucket=none, prefix=none, public=none, roots=none)"
        ),
        |c| format!(
            "asset delivery: {mode} (S3_TYPE={}, bucket={}, prefix={}, public={}, roots={})",
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
