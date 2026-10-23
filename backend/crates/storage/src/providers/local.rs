//! A directory standing in for the bucket (`S3_TYPE=local`).
//!
//! Objects live at `<root>/<key>`. The `ETag` is the MD5 of the file, as S3
//! reports for a single-part upload. A sidecar under `<root>/.meta/<key>.json`
//! keeps the `Content-Type`, `Cache-Control` and `ETag` an upload set, so a
//! multipart upload keeps its multipart-shaped `ETag` and tests can read back
//! what the sync engine sent. A missing sidecar falls back to the guessed type
//! and the computed MD5.

use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::{self, BoxStream};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::io::ReaderStream;

use crate::config::StorageKind;
use crate::key::normalize_rel;
use crate::store::{
    GetOptions, GetResult, ObjectMeta, ObjectStore, PutBody, PutObject, StorageError,
    content_type_for, md5_file,
};

const META_DIR: &str = ".meta";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LocalMeta {
    pub size: u64,
    pub etag: String,
    pub content_type: String,
    pub cache_control: Option<String>,
}

pub struct LocalStore {
    root: PathBuf,
}

static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

fn tmp_sibling(path: &Path) -> PathBuf {
    let n = TMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    path.with_file_name(format!(".{name}.{}-{n}.tmp", std::process::id()))
}

/// Writes `bytes` to `path` through a sibling temp file and a rename, so a
/// reader never sees half a file.
async fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let tmp = tmp_sibling(path);
    tokio::fs::write(&tmp, bytes).await?;
    tokio::fs::rename(&tmp, path).await
}

async fn copy_atomic(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(dir) = to.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    let tmp = tmp_sibling(to);
    tokio::fs::copy(from, &tmp).await?;
    tokio::fs::rename(&tmp, to).await
}

/// Parses a single `bytes=a-b` or suffix `bytes=-n` range against `size`
/// into inclusive bounds, as the backend's `parse_range` does: `start >=
/// size` is unsatisfiable, `None` means ignore the header.
fn parse_range(value: &str, size: u64) -> Option<(u64, u64)> {
    let spec = value.strip_prefix("bytes=")?.trim();
    if spec.contains(',') {
        return None;
    }
    let (start_s, end_s) = spec.split_once('-')?;
    let (start_s, end_s) = (start_s.trim(), end_s.trim());
    if start_s.is_empty() {
        let n: u64 = end_s.parse().ok()?;
        if n == 0 || size == 0 {
            return Some((size, size));
        }
        let n = n.min(size);
        return Some((size - n, size - 1));
    }
    let start: u64 = start_s.parse().ok()?;
    let end = if end_s.is_empty() {
        size.saturating_sub(1)
    } else {
        end_s.parse::<u64>().ok()?.min(size.saturating_sub(1))
    };
    if start > end {
        return None;
    }
    Some((start, end))
}

fn etag_matches(if_none_match: &str, quoted: &str) -> bool {
    if_none_match
        .split(',')
        .map(str::trim)
        .any(|t| t == "*" || t.trim_start_matches("W/") == quoted)
}

impl LocalStore {
    #[must_use]
    pub const fn new(root: PathBuf) -> Self {
        Self { root }
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    fn object_path(&self, key: &str) -> Result<PathBuf, StorageError> {
        match normalize_rel(key) {
            Some(k) if k == key && !k.starts_with(&format!("{META_DIR}/")) => {
                Ok(self.root.join(key))
            }
            _ => Err(StorageError::InvalidKey(key.to_owned())),
        }
    }

    fn meta_path(&self, key: &str) -> PathBuf {
        self.root.join(META_DIR).join(format!("{key}.json"))
    }

    /// The sidecar for `key`, when present and still describing a file of
    /// `size` bytes.
    async fn read_meta(&self, key: &str, size: u64) -> Option<LocalMeta> {
        let raw = tokio::fs::read(self.meta_path(key)).await.ok()?;
        serde_json::from_slice::<LocalMeta>(&raw)
            .ok()
            .filter(|m| m.size == size)
    }

    async fn write_meta(&self, key: &str, meta: &LocalMeta) -> Result<(), StorageError> {
        let raw = serde_json::to_vec(meta).map_err(|e| StorageError::Request(e.to_string()))?;
        write_atomic(&self.meta_path(key), &raw).await?;
        Ok(())
    }

    async fn describe(&self, key: &str, path: &Path, size: u64) -> Result<LocalMeta, StorageError> {
        if let Some(meta) = self.read_meta(key, size).await {
            return Ok(meta);
        }
        let p = path.to_path_buf();
        let digest = tokio::task::spawn_blocking(move || md5_file(&p))
            .await
            .map_err(|e| StorageError::Request(e.to_string()))??;
        Ok(LocalMeta {
            size,
            etag: hex::encode(digest),
            content_type: content_type_for(key),
            cache_control: None,
        })
    }

    /// The sidecar of `key`, for tests that check what an upload set.
    pub async fn meta(&self, key: &str) -> Option<LocalMeta> {
        let path = self.object_path(key).ok()?;
        let size = tokio::fs::metadata(&path).await.ok()?.len();
        self.read_meta(key, size).await
    }
}

#[async_trait]
impl ObjectStore for LocalStore {
    fn kind(&self) -> StorageKind {
        StorageKind::Local
    }

    async fn head(&self, key: &str) -> Result<Option<ObjectMeta>, StorageError> {
        let path = self.object_path(key)?;
        let Ok(md) = tokio::fs::metadata(&path).await else {
            return Ok(None);
        };
        if !md.is_file() {
            return Ok(None);
        }
        let meta = self.describe(key, &path, md.len()).await?;
        Ok(Some(ObjectMeta {
            key: key.to_owned(),
            size: md.len(),
            etag: Some(meta.etag),
            content_type: Some(meta.content_type),
        }))
    }

    async fn get(&self, key: &str, opts: GetOptions) -> Result<GetResult, StorageError> {
        let path = self.object_path(key)?;
        let md = match tokio::fs::metadata(&path).await {
            Ok(md) if md.is_file() => md,
            _ => return Err(StorageError::NotFound(key.to_owned())),
        };
        let size = md.len();
        let meta = self.describe(key, &path, size).await?;
        let etag = format!("\"{}\"", meta.etag);
        let empty = || stream::empty().boxed();

        if opts
            .if_none_match
            .as_deref()
            .is_some_and(|inm| etag_matches(inm, &etag))
        {
            return Ok(GetResult {
                status: 304,
                etag: Some(etag),
                content_type: None,
                content_length: None,
                content_range: None,
                body: empty(),
            });
        }

        let mut file = tokio::fs::File::open(&path).await?;
        match opts.range.as_deref().and_then(|r| parse_range(r, size)) {
            Some((start, _)) if start >= size => Ok(GetResult {
                status: 416,
                etag: Some(etag),
                content_type: None,
                content_length: None,
                content_range: Some(format!("bytes */{size}")),
                body: empty(),
            }),
            Some((start, end)) => {
                let len = end - start + 1;
                file.seek(SeekFrom::Start(start)).await?;
                Ok(GetResult {
                    status: 206,
                    etag: Some(etag),
                    content_type: Some(meta.content_type),
                    content_length: Some(len),
                    content_range: Some(format!("bytes {start}-{end}/{size}")),
                    body: ReaderStream::new(file.take(len)).boxed(),
                })
            }
            None => Ok(GetResult {
                status: 200,
                etag: Some(etag),
                content_type: Some(meta.content_type),
                content_length: Some(size),
                content_range: None,
                body: ReaderStream::new(file).boxed(),
            }),
        }
    }

    async fn put(&self, key: &str, object: PutObject) -> Result<String, StorageError> {
        let path = self.object_path(key)?;
        let (digest, size) = match &object.body {
            PutBody::Bytes(b) => {
                write_atomic(&path, b).await?;
                (md5_bytes(b), b.len() as u64)
            }
            PutBody::File { path: src, size } => {
                copy_atomic(src, &path).await?;
                let p = path.clone();
                let d = tokio::task::spawn_blocking(move || md5_file(&p))
                    .await
                    .map_err(|e| StorageError::Request(e.to_string()))??;
                (d, *size)
            }
        };
        if let Some(sent) = &object.content_md5 {
            use base64::Engine;
            let expected = base64::engine::general_purpose::STANDARD.encode(digest);
            if *sent != expected {
                let _ = tokio::fs::remove_file(&path).await;
                return Err(StorageError::Request(format!(
                    "Content-MD5 mismatch for {key}: sent {sent}, stored {expected}"
                )));
            }
        }
        let etag = hex::encode(digest);
        self.write_meta(
            key,
            &LocalMeta {
                size,
                etag: etag.clone(),
                content_type: object.content_type,
                cache_control: object.cache_control,
            },
        )
        .await?;
        Ok(etag)
    }

    async fn put_multipart_file(
        &self,
        key: &str,
        path: &Path,
        size: u64,
        part_size: u64,
        content_type: &str,
        cache_control: Option<&str>,
    ) -> Result<String, StorageError> {
        let dest = self.object_path(key)?;
        copy_atomic(path, &dest).await?;
        // S3's multipart ETag: MD5 of the concatenated part MD5s, then `-N`.
        let src = dest.clone();
        let etag = tokio::task::spawn_blocking(move || multipart_etag(&src, part_size))
            .await
            .map_err(|e| StorageError::Request(e.to_string()))??;
        self.write_meta(
            key,
            &LocalMeta {
                size,
                etag: etag.clone(),
                content_type: content_type.to_owned(),
                cache_control: cache_control.map(ToOwned::to_owned),
            },
        )
        .await?;
        Ok(etag)
    }

    async fn delete_many(&self, keys: &[String]) -> Vec<(String, Result<(), StorageError>)> {
        let mut out = Vec::with_capacity(keys.len());
        for key in keys {
            let result = async {
                let path = self.object_path(key)?;
                match tokio::fs::remove_file(&path).await {
                    Ok(()) => {}
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
                let _ = tokio::fs::remove_file(self.meta_path(key)).await;
                Ok(())
            }
            .await;
            out.push((key.clone(), result));
        }
        out
    }

    fn list<'a>(&'a self, prefix: &'a str) -> BoxStream<'a, Result<ObjectMeta, StorageError>> {
        let root = self.root.clone();
        let prefix = prefix.to_owned();
        let walk = async move {
            let keys = tokio::task::spawn_blocking(move || walk_keys(&root, &prefix))
                .await
                .map_err(|e| StorageError::Request(e.to_string()))?;
            Ok::<_, StorageError>(keys)
        };
        stream::once(walk)
            .flat_map(move |found| match found {
                Ok(keys) => stream::iter(keys)
                    .then(move |(key, size)| async move {
                        let path = self.object_path(&key)?;
                        let meta = self.describe(&key, &path, size).await?;
                        Ok(ObjectMeta {
                            key,
                            size,
                            etag: Some(meta.etag),
                            content_type: Some(meta.content_type),
                        })
                    })
                    .boxed(),
                Err(e) => stream::iter([Err(e)]).boxed(),
            })
            .boxed()
    }

    async fn put_bucket_cors(&self, origins: &[String]) -> Result<(), StorageError> {
        tracing::info!(?origins, "local store has no CORS; nothing to set");
        Ok(())
    }
}

fn md5_bytes(bytes: &[u8]) -> [u8; 16] {
    use md5::{Digest, Md5};
    Md5::digest(bytes).into()
}

fn multipart_etag(path: &Path, part_size: u64) -> std::io::Result<String> {
    use md5::{Digest, Md5};
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut digests = Vec::new();
    let mut parts = 0usize;
    let part_size = usize::try_from(part_size.max(1)).unwrap_or(usize::MAX);
    loop {
        let mut hasher = Md5::new();
        let mut read = 0usize;
        let mut buf = vec![0u8; part_size.min(1 << 20)];
        while read < part_size {
            let want = buf.len().min(part_size - read);
            let n = file.read(&mut buf[..want])?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            read += n;
        }
        if read == 0 {
            break;
        }
        digests.extend_from_slice(&hasher.finalize());
        parts += 1;
        if read < part_size {
            break;
        }
    }
    Ok(format!("{}-{parts}", hex::encode(Md5::digest(&digests))))
}

/// Every object key under `root` starting with `prefix`, with its size,
/// skipping the sidecar tree and in-flight temp files.
fn walk_keys(root: &Path, prefix: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return out;
    }
    let walker = walkdir::WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !(e.depth() == 1 && e.file_name() == META_DIR));
    for entry in walker.filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if name.starts_with('.') && name.ends_with(".tmp") {
            continue;
        }
        let Ok(rel) = entry.path().strip_prefix(root) else {
            continue;
        };
        let Some(key) = rel.to_str().map(|s| s.replace('\\', "/")) else {
            continue;
        };
        if !key.starts_with(prefix) {
            continue;
        }
        let size = entry.metadata().map_or(0, |m| m.len());
        out.push((key, size));
    }
    out.sort();
    out
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    async fn collect(body: BoxStream<'static, Result<Bytes, std::io::Error>>) -> Vec<u8> {
        let chunks: Vec<_> = body.collect().await;
        chunks
            .into_iter()
            .flat_map(|c| c.unwrap().to_vec())
            .collect()
    }

    fn put_bytes(b: &'static [u8]) -> PutObject {
        PutObject {
            body: PutBody::Bytes(Bytes::from_static(b)),
            content_type: "text/plain".into(),
            cache_control: Some("public, max-age=1".into()),
            content_md5: None,
        }
    }

    #[tokio::test]
    async fn put_get_head_list_delete_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());
        let etag = store
            .put("en/textures/a.txt", put_bytes(b"hello world"))
            .await
            .unwrap();
        assert_eq!(etag, "5eb63bbbe01eeed093cb22bb8f5acdc3");

        let head = store.head("en/textures/a.txt").await.unwrap().unwrap();
        assert_eq!(head.size, 11);
        assert_eq!(head.etag.as_deref(), Some(etag.as_str()));
        assert_eq!(head.content_type.as_deref(), Some("text/plain"));
        assert_eq!(store.head("en/textures/b.txt").await.unwrap(), None);

        let full = store
            .get("en/textures/a.txt", GetOptions::default())
            .await
            .unwrap();
        assert_eq!(full.status, 200);
        assert_eq!(full.content_length, Some(11));
        assert_eq!(collect(full.body).await, b"hello world");

        let part = store
            .get(
                "en/textures/a.txt",
                GetOptions {
                    range: Some("bytes=6-".into()),
                    ..GetOptions::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(part.status, 206);
        assert_eq!(part.content_range.as_deref(), Some("bytes 6-10/11"));
        assert_eq!(collect(part.body).await, b"world");

        let suffix = store
            .get(
                "en/textures/a.txt",
                GetOptions {
                    range: Some("bytes=-5".into()),
                    ..GetOptions::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(collect(suffix.body).await, b"world");

        let past = store
            .get(
                "en/textures/a.txt",
                GetOptions {
                    range: Some("bytes=-0".into()),
                    ..GetOptions::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(past.status, 416);
        assert_eq!(past.content_range.as_deref(), Some("bytes */11"));

        let cached = store
            .get(
                "en/textures/a.txt",
                GetOptions {
                    if_none_match: Some(format!("\"{etag}\"")),
                    ..GetOptions::default()
                },
            )
            .await
            .unwrap();
        assert_eq!(cached.status, 304);

        assert!(matches!(
            store.get("en/nope.txt", GetOptions::default()).await,
            Err(StorageError::NotFound(_))
        ));

        let listed: Vec<_> = store.list("en/").collect().await;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].as_ref().unwrap().key, "en/textures/a.txt");

        let deleted = store.delete_many(&["en/textures/a.txt".into()]).await;
        assert!(deleted[0].1.is_ok());
        assert_eq!(store.head("en/textures/a.txt").await.unwrap(), None);
        assert!(store.list("").collect::<Vec<_>>().await.is_empty());
    }

    #[tokio::test]
    async fn bad_keys_and_wrong_md5_are_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = LocalStore::new(dir.path().to_path_buf());
        assert!(matches!(
            store.put("en/../x.txt", put_bytes(b"x")).await,
            Err(StorageError::InvalidKey(_))
        ));
        assert!(matches!(
            store.put(".meta/x.txt", put_bytes(b"x")).await,
            Err(StorageError::InvalidKey(_))
        ));
        let mut obj = put_bytes(b"x");
        obj.content_md5 = Some("AAAAAAAAAAAAAAAAAAAAAA==".into());
        assert!(store.put("en/x.txt", obj).await.is_err());
        assert_eq!(store.head("en/x.txt").await.unwrap(), None);
    }

    #[tokio::test]
    async fn multipart_etag_has_the_part_count() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("src.bin");
        std::fs::write(&src, vec![7u8; 2500]).unwrap();
        let store = LocalStore::new(dir.path().join("bucket"));
        let etag = store
            .put_multipart_file("en/video/a.mp4", &src, 2500, 1000, "video/mp4", None)
            .await
            .unwrap();
        assert!(etag.ends_with("-3"), "{etag}");
        let head = store.head("en/video/a.mp4").await.unwrap().unwrap();
        assert_eq!(head.etag.as_deref(), Some(etag.as_str()));
    }
}
