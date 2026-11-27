//! The provider-neutral object store interface.

use std::fmt::{self, Debug, Formatter};
use std::fs::File;
use std::io::{self, Error};
use std::path::{Path, PathBuf};

use async_trait::async_trait;
use bytes::Bytes;
use futures::stream::BoxStream;
use mime_guess::from_path;
use thiserror::Error;

use crate::config::StorageKind;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("object not found: {0}")]
    NotFound(String),
    #[error("i/o: {0}")]
    Io(#[from] Error),
    #[error("invalid key {0:?}")]
    InvalidKey(String),
    #[error("{0}")]
    Config(String),
    #[error("store request failed: {0}")]
    Request(String),
}

/// One object as a listing or HEAD reports it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjectMeta {
    pub key: String,
    pub size: u64,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

/// Conditional and partial GET headers
#[derive(Debug, Clone, Default)]
pub struct GetOptions {
    pub range: Option<String>,
    pub if_none_match: Option<String>,
}

/// A GET answer
pub struct GetResult {
    pub status: u16,
    pub etag: Option<String>,
    pub content_type: Option<String>,
    pub content_length: Option<u64>,
    pub content_range: Option<String>,
    pub body: BoxStream<'static, Result<Bytes, Error>>,
}

impl Debug for GetResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("GetResult")
            .field("status", &self.status)
            .field("etag", &self.etag)
            .field("content_type", &self.content_type)
            .field("content_length", &self.content_length)
            .field("content_range", &self.content_range)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub enum PutBody {
    Bytes(Bytes),
    /// Streamed from disk
    File {
        path: PathBuf,
        size: u64,
    },
}

#[derive(Debug, Clone)]
pub struct PutObject {
    pub body: PutBody,
    pub content_type: String,
    pub cache_control: Option<String>,
    /// Base64 of the raw MD5
    pub content_md5: Option<String>,
}

#[async_trait]
pub trait ObjectStore: Send + Sync {
    fn kind(&self) -> StorageKind;

    /// `None` when the object does not exist
    async fn head(&self, key: &str) -> Result<Option<ObjectMeta>, StorageError>;

    async fn get(&self, key: &str, opts: GetOptions) -> Result<GetResult, StorageError>;

    /// Single-request upload
    async fn put(&self, key: &str, object: PutObject) -> Result<String, StorageError>;

    /// Multipart upload of the file at `path` in `part_size` pieces.
    /// Returns the stored `ETag` without quotes.
    async fn put_multipart_file(
        &self,
        key: &str,
        path: &Path,
        size: u64,
        part_size: u64,
        content_type: &str,
        cache_control: Option<&str>,
    ) -> Result<String, StorageError>;

    /// Deletes `keys`
    async fn delete_many(&self, keys: &[String]) -> Vec<(String, Result<(), StorageError>)>;

    /// Every object whose key starts with `prefix`
    fn list<'a>(&'a self, prefix: &'a str) -> BoxStream<'a, Result<ObjectMeta, StorageError>>;

    /// Allows browser GET and HEAD from `origins`
    async fn put_bucket_cors(&self, origins: &[String]) -> Result<(), StorageError>;
}

/// The `Content-Type` an asset goes out with
#[must_use]
pub fn content_type_for(path: &str) -> String {
    let is_m4a = Path::new(path)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("m4a"));
    if is_m4a {
        "audio/mp4".to_owned()
    } else {
        from_path(path).first_or_octet_stream().to_string()
    }
}

/// Strips the surrounding quotes (and a weak `W/` marker) from an `ETag`
#[must_use]
pub fn unquote_etag(etag: &str) -> String {
    etag.trim()
        .trim_start_matches("W/")
        .trim_matches('"')
        .to_owned()
}

/// MD5 of the file at `path`, read in 1 MiB blocks. Blocking: call it from
/// `spawn_blocking`.
///
/// # Errors
/// The file cannot be opened or read.
pub fn md5_file(path: &Path) -> io::Result<[u8; 16]> {
    use md5::{Digest, Md5};
    use std::io::Read;
    let mut file = File::open(path)?;
    let mut hasher = Md5::new();
    let mut buf = vec![0u8; 1 << 20];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hasher.finalize().into())
}
