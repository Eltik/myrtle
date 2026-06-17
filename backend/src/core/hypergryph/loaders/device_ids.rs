use std::path::PathBuf;

use crate::core::hypergryph::config::{DeviceIds, config};

const DEFAULT_DEVICE_IDS_FILE: &str = "device_ids.json";

fn state_path() -> PathBuf {
    PathBuf::from(
        std::env::var("DEVICE_IDS_FILE").unwrap_or_else(|_| DEFAULT_DEVICE_IDS_FILE.into()),
    )
}

/// Explicit override: all three ids supplied via env. Lets an operator pin a
/// known device (e.g. ephemeral containers with no persistent volume). All
/// three must be present and non-empty or the override is ignored.
fn from_env() -> Option<DeviceIds> {
    let ids = DeviceIds {
        device_id: std::env::var("DEVICE_ID").ok()?,
        device_id2: std::env::var("DEVICE_ID2").ok()?,
        device_id3: std::env::var("DEVICE_ID3").ok()?,
    };
    ids.is_complete().then_some(ids)
}

fn read_file(path: &PathBuf) -> Option<DeviceIds> {
    let raw = std::fs::read_to_string(path).ok()?;
    match serde_json::from_str::<DeviceIds>(&raw) {
        Ok(ids) if ids.is_complete() => Some(ids),
        Ok(_) => {
            tracing::warn!(path = %path.display(), "device ids file incomplete, regenerating");
            None
        }
        Err(e) => {
            tracing::warn!(error = %e, path = %path.display(), "device ids file unreadable, regenerating");
            None
        }
    }
}

/// Atomic write via temp file + rename. Called at most once (on first run).
fn write_file(path: &PathBuf, ids: &DeviceIds) -> std::io::Result<()> {
    let body = serde_json::to_vec_pretty(ids)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&tmp, body)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

/// Resolve a stable device identity, in priority order:
///   1. `DEVICE_ID` / `DEVICE_ID2` / `DEVICE_ID3` env vars (explicit pin),
///   2. the persisted JSON file (`DEVICE_IDS_FILE`, default `device_ids.json`),
///   3. a freshly generated set, which is then persisted for next time.
///
/// Device ids must stay stable across restarts: Arknights binds a minted
/// `secret` to the device that requested it, so a regenerated device id breaks
/// re-minting from a stored session and can trip device-change verification.
pub fn resolve_device_ids() -> DeviceIds {
    if let Some(ids) = from_env() {
        tracing::info!("using device ids from environment");
        return ids;
    }

    let path = state_path();
    if let Some(ids) = read_file(&path) {
        tracing::info!(path = %path.display(), "loaded persisted device ids");
        return ids;
    }

    let ids = DeviceIds::generate();
    match write_file(&path, &ids) {
        Ok(()) => {
            tracing::info!(path = %path.display(), "generated and persisted new device ids");
        }
        Err(e) => tracing::warn!(
            error = %e,
            path = %path.display(),
            "failed to persist device ids; they will regenerate on next restart"
        ),
    }
    ids
}

pub async fn load_device_ids() {
    let ids = resolve_device_ids();
    let mut cfg = config().write().await;
    cfg.device_ids = ids;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "myrtle_device_ids_{}_{}.json",
            std::process::id(),
            name
        ));
        p
    }

    #[test]
    fn persisted_ids_round_trip() {
        let path = temp_path("round_trip");
        let _ = std::fs::remove_file(&path);

        let ids = DeviceIds::generate();
        assert!(ids.is_complete());
        write_file(&path, &ids).expect("write");

        let loaded = read_file(&path).expect("read back a complete set");
        assert_eq!(loaded.device_id, ids.device_id);
        assert_eq!(loaded.device_id2, ids.device_id2);
        assert_eq!(loaded.device_id3, ids.device_id3);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_file_reads_none() {
        let path = temp_path("missing");
        let _ = std::fs::remove_file(&path);
        assert!(read_file(&path).is_none());
    }

    #[test]
    fn incomplete_file_is_rejected() {
        let path = temp_path("incomplete");
        std::fs::write(
            &path,
            r#"{"device_id":"abc","device_id2":"","device_id3":""}"#,
        )
        .expect("seed");
        assert!(read_file(&path).is_none());
        let _ = std::fs::remove_file(&path);
    }
}
