//! Every tier list as `GET /tier-lists/{slug}` serves it, written to disk.
//!
//! Exists to prove a schema or DTO change leaves every stored list reading the
//! same: run it once before the change and once after, then diff the two
//! directories. Each list goes through `services::tier_list::get_by_slug`, the
//! handler's own path, over a real `AppState` with the EN game data. Every
//! `tier_list_versions` snapshot is written alongside, raw, and also read back
//! through the typed snapshot reader so an old shape that no longer parses is
//! counted rather than missed.
//!
//! Needs a Postgres it may migrate (it runs `database::init`) and the EN
//! extract, so it is ignored by default. Point it at a COPY of the real data:
//! `TL_DUMP_DB=postgres://... TL_DUMP_DIR=/path cargo test --test tier_list_dump_test -- --ignored --nocapture`

mod common;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use backend::app::cache::store::CacheStore;
use backend::app::error::ApiError;
use backend::app::services::tier_list::{get_by_slug, read_snapshot};
use backend::app::state::{AppConfig, AppState, ServerData};
use backend::core::auth::credentials::CredentialKey;
use backend::core::gamedata::assets::AssetIndex;
use backend::core::hypergryph::constants::Server;

fn en_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../assets/output/en")
}

fn state(db: sqlx::PgPool) -> AppState {
    use std::sync::atomic::AtomicBool;
    let dir = en_root();
    let server_data = Arc::new(ServerData {
        game_data: arc_swap::ArcSwap::new(common::shared_game_data()),
        asset_index: arc_swap::ArcSwap::from_pointee(AssetIndex::build(&dir)),
        game_data_dir: dir.join("gamedata/excel").display().to_string(),
        assets_dir: dir.display().to_string(),
        art_dir: dir.display().to_string(),
        loaded: AtomicBool::new(true),
        residency: Default::default(),
    });
    let config = AppConfig {
        jwt_secret: "test-jwt-secret-not-used-by-this-test".into(),
        game_credential_key: CredentialKey::parse(&"11".repeat(32)).expect("key"),
        rate_limit_rpm: 100,
        service_key: "test-service-key".into(),
        assets_base_dir: dir
            .parent()
            .map_or_else(|| dir.display().to_string(), |p| p.display().to_string()),
        servers: vec![Server::EN],
        default_server: Server::EN,
        asset_ws_urls: HashMap::new(),
    };
    AppState::new(
        db,
        CacheStore::new_memory(),
        HashMap::from([(Server::EN, server_data)]),
        Server::EN,
        config,
        reqwest::Client::new(),
        backend::core::service_account::ServiceAccounts::default(),
    )
}

#[tokio::test]
#[ignore = "needs a migratable Postgres copy and the EN extract"]
async fn dump_every_tier_list() {
    let url = std::env::var("TL_DUMP_DB").expect("TL_DUMP_DB must name a COPY of the database");
    let out = PathBuf::from(std::env::var("TL_DUMP_DIR").expect("TL_DUMP_DIR"));
    std::fs::create_dir_all(out.join("lists")).expect("mkdir lists");
    std::fs::create_dir_all(out.join("versions")).expect("mkdir versions");

    let db = backend::database::init(&url).await.expect("database");
    let state = state(db.clone());

    let slugs: Vec<String> = sqlx::query_scalar("SELECT slug FROM tier_lists ORDER BY slug")
        .fetch_all(&db)
        .await
        .expect("slugs");
    let (mut ok, mut failed, mut not_found, mut tiers, mut placements) = (0, 0, 0, 0, 0);
    let mut unresolved: Vec<String> = Vec::new();
    for slug in &slugs {
        match get_by_slug(&state, slug, Server::EN).await {
            Ok(detail) => {
                ok += 1;
                tiers += detail.tiers.len();
                for p in detail.tiers.iter().flat_map(|t| &t.placements) {
                    placements += 1;
                    if p.entity.is_none() {
                        unresolved.push(format!("{slug}:{}", p.placement.entity_id));
                    }
                }
                let json = serde_json::to_string_pretty(&detail).expect("serialize");
                std::fs::write(out.join("lists").join(format!("{slug}.json")), json)
                    .expect("write list");
            }
            // An inactive list 404s on the live route too; that is its answer.
            Err(ApiError::NotFound) => {
                not_found += 1;
                eprintln!("NOT FOUND {slug}");
            }
            Err(e) => {
                failed += 1;
                eprintln!("FAILED {slug}: {e:?}");
            }
        }
    }

    let versions: Vec<(String, i32, serde_json::Value)> = sqlx::query_as(
        "SELECT tl.slug, v.version, v.snapshot FROM tier_list_versions v
           JOIN tier_lists tl ON tl.id = v.tier_list_id ORDER BY tl.slug, v.version",
    )
    .fetch_all(&db)
    .await
    .expect("versions");
    let (mut read_ok, mut read_failed) = (0, 0);
    for (slug, version, snapshot) in &versions {
        match read_snapshot(snapshot) {
            Ok(_) => read_ok += 1,
            Err(e) => {
                read_failed += 1;
                eprintln!("SNAPSHOT FAILED {slug} v{version}: {e}");
            }
        }
        let json = serde_json::to_string_pretty(snapshot).expect("serialize");
        std::fs::write(
            out.join("versions").join(format!("{slug}.v{version}.json")),
            json,
        )
        .expect("write version");
    }

    println!(
        "lists {} (ok {ok}, not found {not_found}, failed {failed}), tiers {tiers}, placements {placements} (unresolved {}), versions {} (typed read ok {read_ok}, failed {read_failed})",
        slugs.len(),
        unresolved.len(),
        versions.len()
    );
    for u in &unresolved {
        println!("UNRESOLVED {u}");
    }
    assert_eq!(failed, 0);
    assert_eq!(read_failed, 0);
}
