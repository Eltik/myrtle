use std::path::Path;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime};

use tokio::sync::Semaphore;

use crate::AppError;

const TOUCH_AFTER: Duration = Duration::from_hours(24);

pub(super) static PICTURE_RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));
pub(super) static THUMB_RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));
pub(super) static HEAD_RENDERS: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(2));

#[must_use]
pub fn list_story_tree(root: &Path, rel: &str) -> Vec<String> {
    let tree = root.join(rel);
    walkdir::WalkDir::new(&tree)
        .min_depth(1)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| !entry.file_type().is_dir())
        .filter_map(|entry| {
            let rel = entry.path().strip_prefix(&tree).ok()?.to_str()?;
            Some(rel.replace('\\', "/"))
        })
        .collect()
}

#[must_use]
pub fn modified(path: &Path) -> Option<SystemTime> {
    std::fs::metadata(path).and_then(|m| m.modified()).ok()
}

pub(super) fn fresh(target: &Path, sources: &[&Path]) -> bool {
    modified(target).is_some_and(|t| sources.iter().all(|s| modified(s).is_some_and(|m| t >= m)))
}

pub fn touch_if_old(path: &Path) {
    let old = modified(path)
        .and_then(|m| SystemTime::now().duration_since(m).ok())
        .is_some_and(|age| age > TOUCH_AFTER);
    if old {
        let _ = std::fs::OpenOptions::new()
            .write(true)
            .open(path)
            .and_then(|f| f.set_modified(SystemTime::now()));
    }
}

pub fn write_atomically(target: &Path, bytes: &[u8]) -> std::io::Result<()> {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = target
        .parent()
        .ok_or_else(|| std::io::Error::other("cache path has no parent"))?;
    std::fs::create_dir_all(dir)?;
    let stem: String = target
        .file_name()
        .map(|n| n.to_string_lossy())
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_'))
        .collect();
    let tmp = dir.join(format!(
        "{stem}.{}.{}.tmp",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    if let Err(e) = std::fs::write(&tmp, bytes).and_then(|()| std::fs::rename(&tmp, target)) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e);
    }
    Ok(())
}

pub(super) async fn render_permit(
    pool: &'static Semaphore,
) -> Result<tokio::sync::SemaphorePermit<'static>, AppError> {
    pool.acquire()
        .await
        .map_err(|_| AppError::internal("story render pool closed"))
}
