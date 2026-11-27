use std::fs;
use std::io::{self, ErrorKind, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::id;
use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use base64::engine::general_purpose::STANDARD;
use futures::StreamExt;
use futures::stream::{self, BoxStream, iter, once};
use hex::encode;
use serde::{Deserialize, Serialize};
use serde_json::{from_slice, to_vec};
use tokio::fs::{File, copy, create_dir_all, metadata, read, remove_file, rename, write};
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio::task::spawn_blocking;
use tokio_util::io::ReaderStream;
use tracing::info;
use walkdir::WalkDir;

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
    path.with_file_name(format!(".{name}.{}-{n}.tmp", id()))
}

/// Writes `bytes` to `path` through sibling temp file and rename
async fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        create_dir_all(dir).await?;
    }
    let tmp = tmp_sibling(path);
    write(&tmp, bytes).await?;
    rename(&tmp, path).await
}

async fn copy_atomic(from: &Path, to: &Path) -> io::Result<()> {
    if let Some(dir) = to.parent() {
        create_dir_all(dir).await?;
    }
    let tmp = tmp_sibling(to);
    copy(from, &tmp).await?;
    rename(&tmp, to).await
}

/// Parses a single `bytes=a-b` or suffix `bytes=-n` range against `size`
/// into inclusive bounds
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

    async fn read_meta(&self, key: &str, size: u64) -> Option<LocalMeta> {
        let raw = read(self.meta_path(key)).await.ok()?;
        from_slice::<LocalMeta>(&raw)
            .ok()
            .filter(|m| m.size == size)
    }

    async fn write_meta(&self, key: &str, meta: &LocalMeta) -> Result<(), StorageError> {
        let raw = to_vec(meta).map_err(|e| StorageError::Request(e.to_string()))?;
        write_atomic(&self.meta_path(key), &raw).await?;
        Ok(())
    }

    async fn describe(&self, key: &str, path: &Path, size: u64) -> Result<LocalMeta, StorageError> {
        if let Some(meta) = self.read_meta(key, size).await {
            return Ok(meta);
        }
        let p = path.to_path_buf();
        let digest = spawn_blocking(move || md5_file(&p))
            .await
            .map_err(|e| StorageError::Request(e.to_string()))??;
        Ok(LocalMeta {
            size,
            etag: encode(digest),
            content_type: content_type_for(key),
            cache_control: None,
        })
    }

    pub async fn meta(&self, key: &str) -> Option<LocalMeta> {
        let path = self.object_path(key).ok()?;
        let size = metadata(&path).await.ok()?.len();
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
        let Ok(md) = metadata(&path).await else {
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
        let md = match metadata(&path).await {
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

        let mut file = File::open(&path).await?;
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
                let d = spawn_blocking(move || md5_file(&p))
                    .await
                    .map_err(|e| StorageError::Request(e.to_string()))??;
                (d, *size)
            }
        };
        if let Some(sent) = &object.content_md5 {
            use base64::Engine;
            let expected = STANDARD.encode(digest);
            if *sent != expected {
                let _ = remove_file(&path).await;
                return Err(StorageError::Request(format!(
                    "Content-MD5 mismatch for {key}: sent {sent}, stored {expected}"
                )));
            }
        }
        let etag = encode(digest);
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
        let etag = spawn_blocking(move || multipart_etag(&src, part_size))
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
                match remove_file(&path).await {
                    Ok(()) => {}
                    Err(e) if e.kind() == ErrorKind::NotFound => {}
                    Err(e) => return Err(e.into()),
                }
                let _ = remove_file(self.meta_path(key)).await;
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
            let keys = spawn_blocking(move || walk_keys(&root, &prefix))
                .await
                .map_err(|e| StorageError::Request(e.to_string()))?;
            Ok::<_, StorageError>(keys)
        };
        once(walk)
            .flat_map(move |found| match found {
                Ok(keys) => iter(keys)
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
                Err(e) => iter([Err(e)]).boxed(),
            })
            .boxed()
    }

    async fn put_bucket_cors(&self, origins: &[String]) -> Result<(), StorageError> {
        info!(?origins, "local store has no CORS; nothing to set");
        Ok(())
    }
}

fn md5_bytes(bytes: &[u8]) -> [u8; 16] {
    use md5::{Digest, Md5};
    Md5::digest(bytes).into()
}

fn multipart_etag(path: &Path, part_size: u64) -> io::Result<String> {
    use md5::{Digest, Md5};
    use std::io::Read;
    let mut file = fs::File::open(path)?;
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
    Ok(format!("{}-{parts}", encode(Md5::digest(&digests))))
}

/// Every object key under `root` starting with `prefix`
fn walk_keys(root: &Path, prefix: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    if !root.is_dir() {
        return out;
    }
    let walker = WalkDir::new(root)
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
