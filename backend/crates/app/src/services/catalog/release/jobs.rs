//! Job bodies: the sightings ledger and the release art archive.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use myrtle_db::repos::release::{self, Sighting, UpsertStats};
use myrtle_engine_release::kinds::{
    KIND_ACTIVITY, KIND_CHAR, KIND_POOL, KIND_RETRO, KIND_SKIN, KIND_SKIN_WINDOW,
};
use myrtle_gamedata::assets::{AssetIndex, AssetKind};
use myrtle_gamedata::model::GameModel;
use myrtle_jobs::{JobKind, JobSpec};
use myrtle_types::Server;

use crate::cache::key_hash;
use crate::context::Announced;
use crate::snapshots::write_atomic;
use crate::{AppContext, AppError};

pub const ARCHIVE_DIR: &str = "derived/release-art";

pub const DEFAULT_MIRRORS: &[(&str, &str)] = &[
    (
        "gacha:cn",
        "https://raw.githubusercontent.com/fexli/ArknightsResource/main/gacha/{id}.png",
    ),
    (
        "event:cn",
        "https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets2/cn/assets/dyn/arts/ui/stage/%5Buc%5Dhomeentry/{id}.png",
    ),
    (
        "event:cn",
        "https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets/cn/assets/torappu/dynamicassets/arts/ui/stage/%5Buc%5Dhomeentry/{id}.png",
    ),
    (
        "event:en",
        "https://raw.githubusercontent.com/ArknightsAssets/ArknightsAssets2/en/assets/dyn/arts/ui/stage/%5Buc%5Dhomeentry/{id}.png",
    ),
];

const ID_PLACEHOLDER: &str = "{id}";
const MIRROR_MAX_PER_PASS: usize = 1000;

/// `(kind, id, start, end)` per dated thing in `model`, sorted and unique; a zero date is `None`
#[must_use]
pub fn sightings(model: &GameModel) -> Vec<(&'static str, String, Option<i64>, Option<i64>)> {
    let nonzero = |t: i64| (t > 0).then_some(t);
    let mut out = Vec::new();
    for a in model.activities.values() {
        out.push((
            KIND_ACTIVITY,
            a.id.clone(),
            nonzero(a.start_time),
            nonzero(a.end_time),
        ));
    }
    for p in &model.gacha.gacha_pool_client {
        out.push((
            KIND_POOL,
            p.gacha_pool_id.clone(),
            nonzero(p.open_time),
            nonzero(p.end_time),
        ));
    }
    for s in model.skins.char_skins.values() {
        out.push((
            KIND_SKIN,
            s.skin_id.clone(),
            nonzero(s.display_skin.get_time),
            None,
        ));
    }
    for w in &model.skin_windows {
        out.push((
            KIND_SKIN_WINDOW,
            format!("{}@{}", w.skin_id, w.start_time),
            nonzero(w.start_time),
            nonzero(w.end_time),
        ));
    }
    for r in model.retro_acts.values() {
        out.push((KIND_RETRO, r.retro_id.clone(), nonzero(r.start_time), None));
    }
    for id in model.operators.keys() {
        out.push((KIND_CHAR, id.clone(), None, None));
    }
    out.sort_by(|a, b| a.0.cmp(b.0).then_with(|| a.1.cmp(&b.1)));
    out.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1);
    out
}

/// `JobKind::ReleaseLedgerRecord` with its payload's [`Announced`]: Bilibili records as CN
pub async fn record_ledger(
    ctx: &AppContext,
    server: Server,
    res_version: Option<&str>,
    generation: i64,
) -> Result<String, AppError> {
    if !ctx.config.switches.release_ledger.enabled {
        return Ok("release ledger is off".to_owned());
    }
    let server = server.data_server();
    let snapshot = ctx.snapshot_at(server, generation).await?;
    let seen = sightings(&snapshot.model);
    let res_version = res_version.and_then(|_| snapshot.res_version.clone()); // the data's version: the snapshot may be newer than the job
    drop(snapshot);
    let mut conn = ctx.db.conn().await?;
    let mut total = UpsertStats::default();
    for kind in seen.chunk_by(|a, b| a.0 == b.0) {
        let batch: Vec<Sighting<'_>> = kind
            .iter()
            .map(|(kind, id, start_time, end_time)| Sighting {
                kind,
                id,
                start_time: *start_time,
                end_time: *end_time,
            })
            .collect();
        let stats =
            release::upsert_sightings(&mut conn, server, res_version.as_deref(), &batch).await?;
        total.inserted += stats.inserted;
        total.refreshed += stats.refreshed;
    }
    if total.inserted > 0 {
        super::revise(ctx).await; // banner bodies embed debut links
    }
    Ok(format!(
        "{}: {} inserted, {} refreshed",
        server.as_str(),
        total.inserted,
        total.refreshed
    ))
}

/// `(kind:server, url template)` pairs from `release.art_mirrors`, the defaults when it is empty
#[must_use]
pub fn mirrors(configured: &[String]) -> Vec<(String, String)> {
    if configured.is_empty() {
        return DEFAULT_MIRRORS
            .iter()
            .map(|(k, u)| ((*k).to_owned(), (*u).to_owned()))
            .collect();
    }
    configured
        .iter()
        .filter_map(|e| e.split_once('='))
        .map(|(k, u)| (k.trim().to_owned(), u.trim().to_owned()))
        .filter(|(k, u)| !k.is_empty() && u.contains(ID_PLACEHOLDER))
        .collect()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArchiveStats {
    pub copied: u64,
    pub present: u64,
    pub failed: u64,
}

/// The ledger record and the art mirror an announcement for `server` owes
#[must_use]
pub fn follow_ups(ctx: &AppContext, server: Server, announced: &Announced) -> Vec<JobSpec> {
    if !ctx.config.switches.release_ledger.enabled {
        return Vec::new();
    }
    let server = server.data_server();
    let mut kinds = vec![JobKind::ReleaseLedgerRecord];
    if has_own_art(ctx, server) {
        kinds.push(JobKind::ReleaseArtMirror);
    }
    kinds
        .into_iter()
        .map(|kind| JobSpec {
            server: Some(server),
            payload: announced.payload(),
            ..JobSpec::new(kind)
        })
        .collect()
}

/// Queues a ledger pass and the art mirror for every loaded server, with no version; the worker calls it once at boot
pub async fn enqueue_boot_pass(ctx: &AppContext) -> Result<usize, AppError> {
    let mut queued = 0;
    for server in ctx.snapshots.loaded() {
        let announced = Announced {
            generation: ctx.snapshots.get(server).map_or(0, |s| s.generation),
            res_version: None,
        };
        for job in follow_ups(ctx, server, &announced) {
            let spec = JobSpec {
                dedupe_key: Some(format!("boot:{}", server.as_str())),
                ..job
            };
            ctx.queue.enqueue(spec).await?;
            queued += 1;
        }
    }
    Ok(queued)
}

/// A text-only tree is never indexed for art
#[must_use]
pub fn has_own_art(ctx: &AppContext, server: Server) -> bool {
    ctx.snapshots.has_own_art(server)
}

/// `JobKind::ReleaseArtMirror`: archives local release art, mirrors the rest, announces a copy to every process once
pub async fn mirror_art(
    ctx: &AppContext,
    server: Server,
    generation: i64,
) -> Result<String, AppError> {
    let server = server.data_server();
    if !has_own_art(ctx, server) {
        return Ok(format!("{}: text-only, no art to archive", server.as_str()));
    }
    let snapshot = ctx.snapshot_at(server, generation).await?;
    let dir = ctx.snapshots.server_dir(server);
    let local = {
        let (snapshot, dir) = (Arc::clone(&snapshot), dir.clone());
        ctx.cpu
            .offload("release.art", move || archive(&dir, &snapshot.model))
            .await?
    };
    let mirrored = fill_from_mirrors(
        ctx.upstream.client(),
        &dir,
        server.as_str(),
        &snapshot.model,
        &mirrors(&ctx.config.release.art_mirrors),
    )
    .await;
    drop(snapshot);
    if local.copied + mirrored.copied > 0 {
        ctx.announce_art(server).await?;
        ctx.reload_if_changed().await?;
    }
    Ok(format!(
        "{}: {} copied, {} present, {} mirrored, {} failed",
        server.as_str(),
        local.copied,
        local.present,
        mirrored.copied,
        local.failed + mirrored.failed
    ))
}

#[must_use]
pub fn archive(assets_dir: &Path, model: &GameModel) -> ArchiveStats {
    let idx = &model.assets;
    let mut stats = ArchiveStats::default();
    for p in &model.gacha.gacha_pool_client {
        if let Some(rel) = idx.gacha_banner_path(&p.gacha_pool_id) {
            archive_one(assets_dir, rel, "gacha", &p.gacha_pool_id, &mut stats);
        }
    }
    for a in model.activities.values() {
        if let Some(rel) = idx.path(AssetKind::EventBanner, &a.id) {
            archive_one(assets_dir, rel, "event", &a.id, &mut stats);
        }
    }
    stats
}

fn archive_one(assets_dir: &Path, rel: &str, kind: &str, id: &str, stats: &mut ArchiveStats) {
    let rel = rel.trim_start_matches('/');
    if rel.starts_with(ARCHIVE_DIR) {
        stats.present += 1;
        return;
    }
    let source = assets_dir.join(rel);
    if std::fs::metadata(&source).is_ok_and(|m| m.len() == 0) {
        stats.failed += 1; // an evicted stub; the archive outlives its source
        return;
    }
    let target = archive_path(assets_dir, kind, id);
    if target.exists() && same_bytes(&source, &target) {
        stats.present += 1;
        return;
    }
    let copied = target
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()
        .and_then(|_| std::fs::copy(&source, &target));
    match copied {
        Ok(_) => stats.copied += 1,
        Err(e) => {
            stats.failed += 1;
            tracing::debug!(source = %source.display(), error = %e, "release art copy failed");
        }
    }
}

fn archive_path(assets_dir: &Path, kind: &str, id: &str) -> PathBuf {
    assets_dir
        .join(ARCHIVE_DIR)
        .join(kind)
        .join(format!("{id}.png"))
}

fn same_bytes(a: &Path, b: &Path) -> bool {
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(ma), Ok(mb)) if ma.len() == mb.len() => {
            matches!((std::fs::read(a), std::fs::read(b)), (Ok(x), Ok(y)) if x == y)
        }
        _ => false,
    }
}

/// Marks an id every template answered 404 for, keyed by the template set so new mirrors retry
fn absent_marker(assets_dir: &Path, kind: &str, templates: &[&str], id: &str) -> PathBuf {
    assets_dir
        .join(ARCHIVE_DIR)
        .join(".absent")
        .join(format!(
            "{kind}-{}",
            key_hash(templates.join("\n").as_bytes())
        ))
        .join(id)
}

async fn fill_from_mirrors(
    client: &reqwest::Client,
    assets_dir: &Path,
    server: &str,
    model: &GameModel,
    mirrors: &[(String, String)],
) -> ArchiveStats {
    let mut stats = ArchiveStats::default();
    let mirrors: Vec<(&str, &str)> = mirrors
        .iter()
        .filter_map(|(k, u)| {
            let (kind, srv) = k.split_once(':')?;
            (srv == server).then_some((kind, u.as_str()))
        })
        .collect();
    if mirrors.is_empty() {
        return stats;
    }
    let idx: &AssetIndex = &model.assets;
    let mut wanted: Vec<(&str, &str, i64)> = model
        .gacha
        .gacha_pool_client
        .iter()
        .filter(|p| idx.gacha_banner_path(&p.gacha_pool_id).is_none())
        .map(|p| ("gacha", p.gacha_pool_id.as_str(), p.open_time))
        .chain(
            model
                .activities
                .values()
                .filter(|a| idx.path(AssetKind::EventBanner, &a.id).is_none())
                .map(|a| ("event", a.id.as_str(), a.start_time)),
        )
        .collect();
    wanted.sort_by_key(|w| std::cmp::Reverse(w.2));
    let mut asked = 0usize;
    for (kind, id, _) in wanted {
        if asked >= MIRROR_MAX_PER_PASS {
            break;
        }
        let target = archive_path(assets_dir, kind, id);
        let templates: Vec<&str> = mirrors
            .iter()
            .filter(|(k, _)| *k == kind)
            .map(|(_, u)| *u)
            .collect();
        let marker = absent_marker(assets_dir, kind, &templates, id);
        if templates.is_empty() || target.exists() || marker.exists() {
            continue;
        }
        asked += 1;
        let mut all_absent = true;
        for template in &templates {
            match fetch(client, &template.replace(ID_PLACEHOLDER, id)).await {
                Fetch::Found(bytes) => {
                    let path = target.clone();
                    let written = tokio::task::spawn_blocking(move || write_atomic(&path, &bytes))
                        .await
                        .unwrap_or_else(|e| Err(std::io::Error::other(e)));
                    match written {
                        Ok(()) => stats.copied += 1,
                        Err(e) => {
                            stats.failed += 1;
                            tracing::warn!(target = %target.display(), error = %e, "release art write failed");
                        }
                    }
                    all_absent = false;
                    break;
                }
                Fetch::Absent => {}
                Fetch::Failed => {
                    stats.failed += 1;
                    all_absent = false;
                }
            }
            tokio::time::sleep(Duration::from_millis(50)).await; // stay polite to the mirror hosts
        }
        if all_absent {
            if let Some(dir) = marker.parent() {
                let _ = tokio::fs::create_dir_all(dir).await;
            }
            let _ = tokio::fs::write(&marker, b"").await;
        }
    }
    stats
}

enum Fetch {
    Found(Vec<u8>),
    Absent,
    Failed,
}

async fn fetch(client: &reqwest::Client, url: &str) -> Fetch {
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::debug!(url, error = %e, "mirror fetch failed");
            return Fetch::Failed;
        }
    };
    if resp.status() == reqwest::StatusCode::NOT_FOUND {
        return Fetch::Absent;
    }
    match resp.error_for_status() {
        Ok(r) => r
            .bytes()
            .await
            .map_or(Fetch::Failed, |b| Fetch::Found(b.to_vec())),
        Err(e) => {
            tracing::debug!(url, error = %e, "mirror status");
            Fetch::Failed
        }
    }
}
