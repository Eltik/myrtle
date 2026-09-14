use std::path::{Path, PathBuf};

use crate::core::gamedata::{assets::AssetIndex, types::GameData};

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

/// Token a mirror template substitutes with the art id.
const ID_PLACEHOLDER: &str = "{id}";

const MIRROR_MAX_PER_PASS: usize = 1000;

pub fn mirrors() -> Vec<(String, String)> {
    match std::env::var("RELEASE_ART_MIRRORS") {
        Ok(v) => v
            .split(';')
            .filter_map(|e| e.split_once('='))
            .map(|(k, u)| (k.trim().to_string(), u.trim().to_string()))
            .filter(|(k, u)| !k.is_empty() && u.contains(ID_PLACEHOLDER))
            .collect(),
        Err(_) => DEFAULT_MIRRORS
            .iter()
            .map(|(k, u)| ((*k).to_string(), (*u).to_string()))
            .collect(),
    }
}

fn absent_marker(assets_dir: &Path, kind: &str, templates: &[&str], id: &str) -> PathBuf {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    templates.hash(&mut h);
    assets_dir
        .join(ARCHIVE_DIR)
        .join(".absent")
        .join(format!("{kind}-{:016x}", h.finish()))
        .join(id)
}

pub async fn fill_from_mirrors(
    client: &reqwest::Client,
    assets_dir: &Path,
    server: &str,
    gd: &GameData,
    idx: &AssetIndex,
) -> ArchiveStats {
    let mut stats = ArchiveStats::default();
    let mirrors: Vec<(String, String)> = mirrors()
        .into_iter()
        .filter_map(|(k, u)| {
            let (kind, srv) = k.split_once(':')?;
            (srv == server).then(|| (kind.to_string(), u))
        })
        .collect();
    if mirrors.is_empty() {
        return stats;
    }
    let mut wanted: Vec<(&str, String, i64)> = Vec::new();
    for p in &gd.gacha.gacha_pool_client {
        if idx.gacha_banner_path(&p.gacha_pool_id).is_none() {
            wanted.push(("gacha", p.gacha_pool_id.clone(), p.open_time));
        }
    }
    for a in gd.activities.values() {
        if idx
            .path(crate::core::gamedata::assets::AssetKind::EventBanner, &a.id)
            .is_none()
        {
            wanted.push(("event", a.id.clone(), a.start_time));
        }
    }
    wanted.sort_by_key(|w| std::cmp::Reverse(w.2));
    let mut asked = 0usize;
    for (kind, id, _) in wanted {
        if asked >= MIRROR_MAX_PER_PASS {
            break;
        }
        let target = assets_dir
            .join(ARCHIVE_DIR)
            .join(kind)
            .join(format!("{id}.png"));
        let templates: Vec<&str> = mirrors
            .iter()
            .filter(|(k, _)| k == kind)
            .map(|(_, u)| u.as_str())
            .collect();
        if templates.is_empty()
            || target.exists()
            || absent_marker(assets_dir, kind, &templates, &id).exists()
        {
            continue;
        }
        asked += 1;
        let mut all_absent = true;
        for template in &templates {
            let url = template.replace(ID_PLACEHOLDER, &id);
            match fetch_one(client, &url).await {
                Fetch::Found(bytes) => {
                    if let Some(dir) = target.parent() {
                        let _ = tokio::fs::create_dir_all(dir).await;
                    }
                    match tokio::fs::write(&target, &bytes).await {
                        Ok(()) => stats.copied += 1,
                        Err(e) => {
                            stats.failed += 1;
                            tracing::debug!(target = %target.display(), error = %e, "archive write failed");
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
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }
        if all_absent {
            let marker = absent_marker(assets_dir, kind, &templates, &id);
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

async fn fetch_one(client: &reqwest::Client, url: &str) -> Fetch {
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
        Ok(r) => match r.bytes().await {
            Ok(b) => Fetch::Found(b.to_vec()),
            Err(e) => {
                tracing::debug!(url, error = %e, "mirror body failed");
                Fetch::Failed
            }
        },
        Err(e) => {
            tracing::debug!(url, error = %e, "mirror status");
            Fetch::Failed
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ArchiveStats {
    pub copied: u64,
    pub present: u64,
    pub failed: u64,
}

fn archive_one(assets_dir: &Path, rel: &str, kind: &str, id: &str, stats: &mut ArchiveStats) {
    let target: PathBuf = assets_dir
        .join(ARCHIVE_DIR)
        .join(kind)
        .join(format!("{id}.png"));
    if rel.trim_start_matches('/').starts_with(ARCHIVE_DIR) {
        stats.present += 1;
        return;
    }
    let source = assets_dir.join(rel.trim_start_matches('/'));
    if target.exists() && same_bytes(&source, &target) {
        stats.present += 1;
        return;
    }
    let ok = target
        .parent()
        .map(std::fs::create_dir_all)
        .transpose()
        .and_then(|_| std::fs::copy(&source, &target));
    match ok {
        Ok(_) => stats.copied += 1,
        Err(e) => {
            stats.failed += 1;
            tracing::debug!(source = %source.display(), error = %e, "release art copy failed");
        }
    }
}

fn same_bytes(a: &Path, b: &Path) -> bool {
    match (std::fs::metadata(a), std::fs::metadata(b)) {
        (Ok(ma), Ok(mb)) if ma.len() != mb.len() => return false,
        (Ok(_), Ok(_)) => {}
        _ => return false,
    }
    matches!((std::fs::read(a), std::fs::read(b)), (Ok(x), Ok(y)) if x == y)
}

pub fn archive(assets_dir: &Path, gd: &GameData, idx: &AssetIndex) -> ArchiveStats {
    let mut stats = ArchiveStats::default();
    for p in &gd.gacha.gacha_pool_client {
        if let Some(rel) = idx.gacha_banner_path(&p.gacha_pool_id) {
            archive_one(assets_dir, rel, "gacha", &p.gacha_pool_id, &mut stats);
        }
    }
    for a in gd.activities.values() {
        if let Some(rel) = idx.path(crate::core::gamedata::assets::AssetKind::EventBanner, &a.id) {
            archive_one(assets_dir, rel, "event", &a.id, &mut stats);
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::{activity::ActivityBasicInfo, gacha::GachaPoolClient};

    #[tokio::test]
    #[ignore = "network"]
    async fn live_mirrors_fill_and_mark_absent() {
        let root = std::env::temp_dir().join(format!("myrtle_mirror_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let mut gd = GameData::new();
        for id in ["LIMITED_67_0_1", "DOUBLE_77_0_2", "NOPE_1_0_1"] {
            gd.gacha.gacha_pool_client.push(GachaPoolClient {
                gacha_pool_id: id.into(),
                ..Default::default()
            });
        }
        for id in ["act41side", "act50side", "act16mini", "actnope"] {
            gd.activities.insert(
                id.into(),
                ActivityBasicInfo {
                    id: id.into(),
                    ..Default::default()
                },
            );
        }
        let idx = AssetIndex::build(&root);
        let stats = fill_from_mirrors(&reqwest::Client::new(), &root, "cn", &gd, &idx).await;
        println!("mirror stats: {stats:?}");
        assert_eq!((stats.copied, stats.failed), (5, 0));
        assert!(
            root.join(ARCHIVE_DIR)
                .join("gacha/LIMITED_67_0_1.png")
                .exists()
        );
        assert!(root.join(ARCHIVE_DIR).join("event/act41side.png").exists());
        assert!(root.join(ARCHIVE_DIR).join("event/act16mini.png").exists());
        let urls = |kind: &str| -> Vec<String> {
            mirrors()
                .into_iter()
                .filter(|(k, _)| k == &format!("{kind}:cn"))
                .map(|(_, u)| u)
                .collect()
        };
        let gacha_urls = urls("gacha");
        let event_urls = urls("event");
        fn as_refs(v: &[String]) -> Vec<&str> {
            v.iter().map(String::as_str).collect()
        }
        assert!(absent_marker(&root, "gacha", &as_refs(&gacha_urls), "NOPE_1_0_1").exists());
        assert!(absent_marker(&root, "event", &as_refs(&event_urls), "actnope").exists());
        let idx2 = AssetIndex::build(&root);
        assert!(idx2.gacha_banner_path("LIMITED_67_0_1").is_some());
        assert!(idx2.event_banner_path("act50side").is_some());
        let again = fill_from_mirrors(&reqwest::Client::new(), &root, "cn", &gd, &idx).await;
        assert_eq!(again.copied, 0);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn archive_copies_once_and_index_reads_it_back() {
        let root = std::env::temp_dir().join(format!("myrtle_release_art_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let root = root.as_path();
        let pack = root.join("textures/spritepack/ui_home_act_banner_gacha_h2_0");
        std::fs::create_dir_all(&pack).unwrap();
        std::fs::write(pack.join("picLimited_76_0_1.png"), b"png").unwrap();
        let entry = root.join("textures/arts/ui/stage/[uc]homeentry");
        std::fs::create_dir_all(&entry).unwrap();
        std::fs::write(entry.join("act54side.png"), b"png").unwrap();

        let mut gd = GameData::new();
        gd.gacha.gacha_pool_client.push(GachaPoolClient {
            gacha_pool_id: "LIMITED_76_0_1".into(),
            ..Default::default()
        });
        gd.activities.insert(
            "act54side".into(),
            ActivityBasicInfo {
                id: "act54side".into(),
                ..Default::default()
            },
        );
        let idx = AssetIndex::build(root);
        let first = archive(root, &gd, &idx);
        assert_eq!((first.copied, first.present, first.failed), (2, 0, 0));
        let second = archive(root, &gd, &idx);
        assert_eq!((second.copied, second.present), (0, 2));

        std::fs::write(entry.join("act54side.png"), b"png2").unwrap();
        let third = archive(root, &gd, &idx);
        assert_eq!((third.copied, third.present), (1, 1));
        assert_eq!(
            std::fs::read(root.join("derived/release-art/event/act54side.png")).unwrap(),
            b"png2"
        );

        let theme = root.join("textures/spritepack/ui_zone_home_theme_act54side");
        std::fs::create_dir_all(&theme).unwrap();
        std::fs::write(theme.join("act54side.png"), b"sheet").unwrap();
        let idx_theme = AssetIndex::build(root);
        assert_eq!(
            idx_theme.event_banner_path("act54side"),
            Some("/textures/arts/ui/stage/[uc]homeentry/act54side.png")
        );

        std::fs::remove_dir_all(root.join("textures/spritepack")).unwrap();
        std::fs::remove_dir_all(root.join("textures/arts")).unwrap();
        let idx2 = AssetIndex::build(root);
        assert_eq!(
            idx2.gacha_banner_path("LIMITED_76_0_1"),
            Some("/derived/release-art/gacha/LIMITED_76_0_1.png")
        );
        assert_eq!(
            idx2.event_banner_path("act54side"),
            Some("/derived/release-art/event/act54side.png")
        );
        let _ = std::fs::remove_dir_all(root);
    }
}
