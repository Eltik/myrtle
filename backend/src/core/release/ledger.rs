use sqlx::PgPool;

use crate::{
    core::gamedata::types::GameData,
    core::hypergryph::constants::Server,
    database::queries::release::{SightingInput, UpsertStats, upsert_sightings},
};

pub const KIND_ACTIVITY: &str = "activity";
pub const KIND_POOL: &str = "pool";
pub const KIND_SKIN: &str = "skin";
pub const KIND_SKIN_WINDOW: &str = "skin_window";
pub const KIND_RETRO: &str = "retro";
pub const KIND_CHAR: &str = "char";
/// Override-only: a Fashion Review has no id in either server's data, so its
/// override is keyed by the CN edition's start time in unix seconds. Never a
/// sighting kind.
pub const KIND_REVIEW: &str = "review";

pub fn enabled() -> bool {
    match std::env::var("RELEASE_LEDGER") {
        Ok(v) => !matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "0" | "false" | "off"
        ),
        Err(_) => true,
    }
}

pub const fn ledger_server(server: Server) -> Server {
    match server {
        Server::Bilibili => Server::CN,
        other => other,
    }
}

fn nonzero(t: i64) -> Option<i64> {
    (t > 0).then_some(t)
}

pub fn collect(gd: &GameData) -> Vec<SightingInput> {
    let mut out: Vec<SightingInput> = Vec::new();
    for a in gd.activities.values() {
        out.push(SightingInput {
            kind: KIND_ACTIVITY,
            id: a.id.clone(),
            start_time: nonzero(a.start_time),
            end_time: nonzero(a.end_time),
        });
    }
    for p in &gd.gacha.gacha_pool_client {
        out.push(SightingInput {
            kind: KIND_POOL,
            id: p.gacha_pool_id.clone(),
            start_time: nonzero(p.open_time),
            end_time: nonzero(p.end_time),
        });
    }
    for s in gd.skins.char_skins.values() {
        out.push(SightingInput {
            kind: KIND_SKIN,
            id: s.skin_id.clone(),
            start_time: nonzero(s.display_skin.get_time),
            end_time: None,
        });
    }
    for w in &gd.skin_windows {
        out.push(SightingInput {
            kind: KIND_SKIN_WINDOW,
            id: format!("{}@{}", w.skin_id, w.start_time),
            start_time: nonzero(w.start_time),
            end_time: nonzero(w.end_time),
        });
    }
    for r in gd.retro_acts.values() {
        out.push(SightingInput {
            kind: KIND_RETRO,
            id: r.retro_id.clone(),
            start_time: nonzero(r.start_time),
            end_time: None,
        });
    }
    for id in gd.operators.keys() {
        out.push(SightingInput {
            kind: KIND_CHAR,
            id: id.clone(),
            start_time: None,
            end_time: None,
        });
    }
    out.sort_by(|a, b| a.kind.cmp(b.kind).then(a.id.cmp(&b.id)));
    out.dedup_by(|a, b| a.kind == b.kind && a.id == b.id);
    out
}

pub async fn record(
    db: &PgPool,
    server: Server,
    gd: &GameData,
    res_version: Option<&str>,
) -> Result<UpsertStats, sqlx::Error> {
    let server_id = ledger_server(server).index() as i16;
    let rows = collect(gd);
    let mut total = UpsertStats::default();
    for kind in [
        KIND_ACTIVITY,
        KIND_POOL,
        KIND_SKIN,
        KIND_SKIN_WINDOW,
        KIND_RETRO,
        KIND_CHAR,
    ] {
        let batch: Vec<SightingInput> = rows.iter().filter(|r| r.kind == kind).cloned().collect();
        let s = upsert_sightings(db, server_id, res_version, &batch).await?;
        total.inserted += s.inserted;
        total.refreshed += s.refreshed;
    }
    Ok(total)
}

pub fn spawn_record(
    state: crate::app::state::AppState,
    server: Server,
    res_version: Option<String>,
) {
    if !enabled() {
        return;
    }
    tokio::spawn(async move {
        let sd = state.server_data(server);
        let gd = sd.game_data.load_full();
        let assets_dir = sd.assets_dir.clone();
        let idx = sd.asset_index.load_full();
        let local = {
            let (gd, idx, dir) = (gd.clone(), idx.clone(), assets_dir.clone());
            tokio::task::spawn_blocking(move || {
                super::art::archive(std::path::Path::new(&dir), &gd, &idx)
            })
            .await
            .unwrap_or_default()
        };
        let mirrored = super::art::fill_from_mirrors(
            &state.http_client,
            std::path::Path::new(&assets_dir),
            ledger_server(server).as_str(),
            &gd,
            &idx,
        )
        .await;
        tracing::info!(
            server = server.as_str(),
            copied = local.copied,
            present = local.present,
            mirrored = mirrored.copied,
            failed = local.failed + mirrored.failed,
            "release art archived"
        );
        if local.copied + mirrored.copied > 0 {
            let dir = assets_dir.clone();
            if let Ok(new_idx) = tokio::task::spawn_blocking(move || {
                crate::core::gamedata::assets::AssetIndex::build(std::path::Path::new(&dir))
            })
            .await
            {
                state.swap_asset_index(server, new_idx);
                state.cache.invalidate_by_prefix("static:cn:release:").await;
            }
        }
        match record(&state.db, server, &gd, res_version.as_deref()).await {
            Ok(stats) => tracing::info!(
                server = server.as_str(),
                res_version = res_version.as_deref().unwrap_or("none"),
                inserted = stats.inserted,
                refreshed = stats.refreshed,
                "release ledger recorded"
            ),
            Err(e) => tracing::warn!(
                server = server.as_str(),
                error = %e,
                "release ledger write failed"
            ),
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::gamedata::types::{
        activity::ActivityBasicInfo, gacha::GachaPoolClient, shop::SkinWindow, skin::Skin,
    };

    #[test]
    fn collect_covers_every_kind_and_zero_dates_are_null() {
        let mut gd = GameData::new();
        gd.activities.insert(
            "act1side".into(),
            ActivityBasicInfo {
                id: "act1side".into(),
                start_time: 100,
                end_time: 0,
                ..Default::default()
            },
        );
        gd.gacha.gacha_pool_client.push(GachaPoolClient {
            gacha_pool_id: "DOUBLE_1".into(),
            open_time: 50,
            end_time: 60,
            ..Default::default()
        });
        let mut skin = Skin {
            skin_id: "char_a@x#1".into(),
            ..Default::default()
        };
        skin.display_skin.get_time = 0;
        gd.skins.char_skins.insert(skin.skin_id.clone(), skin);
        gd.skin_windows.push(SkinWindow {
            skin_id: "char_a@x#1".into(),
            start_time: 70,
            end_time: 80,
        });
        gd.operators.insert("char_a".into(), Default::default());

        let rows = collect(&gd);
        let kinds: Vec<&str> = rows.iter().map(|r| r.kind).collect();
        assert_eq!(kinds, ["activity", "char", "pool", "skin", "skin_window"]);
        assert_eq!(rows[0].end_time, None);
        assert_eq!(rows[3].start_time, None);
        assert_eq!(rows[4].id, "char_a@x#1@70");
    }

    #[test]
    fn bilibili_is_recorded_as_cn() {
        assert_eq!(ledger_server(Server::Bilibili), Server::CN);
        assert_eq!(ledger_server(Server::EN), Server::EN);
    }
}
