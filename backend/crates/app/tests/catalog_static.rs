#![allow(clippy::unwrap_used)]

mod common;

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use myrtle_app::cache::CachedJson;
use myrtle_app::services::catalog::levels::{
    self, camel_key, camelize_keys, level_path, read_error, reshape_map_matrix,
};
use myrtle_app::services::catalog::static_data::{
    self, RESOURCES, collect_level_enemy_ids, serialize_resource,
};
use myrtle_app::snapshots::{Snapshot, SnapshotRegistry};
use myrtle_app::{AppContext, AppError};
use myrtle_gamedata::model::GameModel;
use myrtle_testkit::db::TestDb;
use myrtle_types::Server;
use serde_json::{Value, json};

async fn context_with_files() -> Option<(AppContext, TestDb, Arc<Snapshot>)> {
    let (mut ctx, db, snapshot) = common::context_with_data().await?;
    let mut config = common::config();
    config.assets.dir = std::env::var("MYRTLE_TEST_ASSETS").unwrap_or_else(|_| {
        concat!(env!("CARGO_MANIFEST_DIR"), "/../../../assets/output").to_owned()
    });
    let registry = SnapshotRegistry::new(&config);
    registry.install_shared(Server::EN, Arc::clone(&snapshot));
    ctx.snapshots = Arc::new(registry);
    Some((ctx, db, snapshot))
}

fn parse(body: &str) -> Value {
    serde_json::from_str(body).unwrap()
}

fn set(ids: &[&str]) -> HashSet<String> {
    ids.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn every_resource_serializes_and_an_unknown_one_is_not_found() {
    let gd = GameModel::default();
    for name in RESOURCES {
        assert!(serialize_resource(&gd, name).is_ok(), "{name}");
    }
    assert!(matches!(
        serialize_resource(&gd, "nope"),
        Err(AppError::NotFound(None))
    ));
}

#[tokio::test]
async fn a_table_is_served_once_and_hit_with_the_same_etag() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let first = static_data::resource(&ctx, Server::EN, "ranges")
        .await
        .unwrap();
    assert_eq!(
        parse(&first.body).as_object().unwrap().len(),
        snapshot.model.ranges.len()
    );
    assert_eq!(
        ctx.cache
            .get_json::<CachedJson>("static:en:ranges:0:0")
            .await,
        None,
        "held in process, not in the store"
    );
    let second = static_data::resource(&ctx, Server::EN, "ranges")
        .await
        .unwrap();
    assert_eq!(first.etag, second.etag);
    assert!(
        Arc::ptr_eq(&first.body, &second.body),
        "a hit shares the body"
    );

    ctx.cache.invalidate_prefix("static:en:").await;
    let rebuilt = static_data::resource(&ctx, Server::EN, "ranges")
        .await
        .unwrap();
    assert!(!Arc::ptr_eq(&first.body, &rebuilt.body));
    assert_eq!(rebuilt, first);

    assert!(matches!(
        static_data::resource(&ctx, Server::EN, "users").await,
        Err(AppError::NotFound(None))
    ));
    assert!(matches!(
        static_data::resource(&ctx, Server::JP, "ranges").await,
        Err(AppError::NotFound(_))
    ));
    db.close().await;
}

#[tokio::test]
async fn the_skins_index_keeps_only_the_stats_tab_fields() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let body = parse(
        &static_data::skins_index(&ctx, Server::EN)
            .await
            .unwrap()
            .body,
    );
    let index = body.as_object().unwrap();
    assert_eq!(index.len(), snapshot.model.skins.char_skins.len());
    let (id, skin) = index.iter().next().unwrap();
    let mut keys: Vec<&str> = skin
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    assert_eq!(keys, ["charId", "displaySkin", "obtainChannel", "skinId"]);
    assert_eq!(skin["skinId"], id.as_str());
    assert_eq!(skin["displaySkin"].as_object().unwrap().len(), 14);
    assert!(skin["displaySkin"].get("modelName").is_none());
    db.close().await;
}

#[tokio::test]
async fn enemy_and_chibi_pages_cut_one_record() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let gd = &snapshot.model;
    let enemy_id = gd
        .enemy_stage_index
        .keys()
        .find(|id| gd.enemies.enemy_data.contains_key(*id))
        .unwrap();
    let detail = parse(
        &static_data::enemy_detail(&ctx, Server::EN, enemy_id)
            .await
            .unwrap()
            .body,
    );
    assert_eq!(detail["enemy"]["enemyId"], enemy_id.as_str());
    assert!(detail["raceData"].is_object());
    assert!(detail["audio"].is_array());
    let stages = parse(
        &static_data::enemy_stages(&ctx, Server::EN, enemy_id)
            .await
            .unwrap()
            .body,
    );
    assert_eq!(
        stages.as_array().unwrap().len(),
        gd.enemy_stage_index[enemy_id].len()
    );
    assert_eq!(
        &*static_data::enemy_stages(&ctx, Server::EN, "enemy_none")
            .await
            .unwrap()
            .body,
        "[]"
    );
    assert!(matches!(
        static_data::enemy_detail(&ctx, Server::EN, "enemy_none").await,
        Err(AppError::NotFound(None))
    ));

    if let Some(char_id) = gd.chibis.by_operator.keys().next() {
        static_data::chibi(&ctx, Server::EN, char_id).await.unwrap();
    }
    assert!(matches!(
        static_data::chibi(&ctx, Server::EN, "char_none").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_stage_detail_carries_its_level_enemies_and_drops() {
    let Some((ctx, db, snapshot)) = context_with_files().await else {
        return;
    };
    let body = parse(
        &static_data::stage_detail(&ctx, Server::EN, "main_01-07")
            .await
            .unwrap()
            .body,
    );
    assert_eq!(body["stage"]["stageId"], "main_01-07");
    assert_eq!(body["synthetic"], false);
    assert!(body["zone"].is_object());
    let level = &body["levelData"];
    assert!(level["mapData"]["map"].is_array());
    let enemies = body["enemies"].as_object().unwrap();
    assert!(!enemies.is_empty());
    let referenced = collect_level_enemy_ids(level);
    assert!(enemies.keys().all(|id| referenced.contains(id)));
    let materials = body["materials"].as_object().unwrap();
    assert!(
        materials
            .keys()
            .all(|id| snapshot.model.materials.items.contains_key(id))
    );

    assert!(matches!(
        static_data::stage_detail(&ctx, Server::EN, "main_99-99").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[tokio::test]
async fn a_level_is_camelized_and_an_escaping_id_is_refused() {
    let Some((ctx, db, _)) = context_with_files().await else {
        return;
    };
    let level = parse(
        &levels::level(&ctx, Server::EN, "main_01-07")
            .await
            .unwrap()
            .body,
    );
    assert!(level["mapData"]["map"][0].is_array());
    assert!(level.get("MapData").is_none());
    for id in ["../../../etc/passwd", "a/b", "a\\b", ".."] {
        assert!(
            matches!(
                levels::level(&ctx, Server::EN, id).await,
                Err(AppError::BadRequest(_))
            ),
            "{id}"
        );
    }
    assert!(matches!(
        levels::level(&ctx, Server::EN, "main_99-99").await,
        Err(AppError::NotFound(None))
    ));
    db.close().await;
}

#[test]
fn level_paths_stay_inside_the_levels_directory() {
    let root = Path::new("/data/levels");
    assert_eq!(
        level_path(root, "Obt/Main/level_main_01-07"),
        Some(PathBuf::from("/data/levels/obt/main/level_main_01-07.json"))
    );
    assert_eq!(
        level_path(root, "Obt\\Main\\level_main_01-07"),
        Some(PathBuf::from("/data/levels/obt/main/level_main_01-07.json"))
    );
    for bad in [
        "../secret",
        "obt/../../secret",
        "/etc/passwd",
        "obt//x",
        "./x",
        "c:/x",
        "",
    ] {
        assert_eq!(level_path(root, bad), None, "{bad}");
    }
}

#[test]
fn level_enemies_union_db_refs_and_wave_action_keys() {
    let level = json!({
        "enemyDbRefs": [
            { "id": "enemy_1007_slime", "level": 0 },
            { "id": "enemy_1000_gopro" },
            { "level": 1 },
        ],
        "waves": [
            { "fragments": [
                { "actions": [
                    { "key": "enemy_1007_slime" },
                    { "key": "" },
                    { "key": "enemy_1001_bigbo" },
                    { "noKey": true },
                ]},
                { "noActions": true },
            ]},
            { "noFragments": true },
        ],
    });
    assert_eq!(
        collect_level_enemy_ids(&level),
        set(&["enemy_1007_slime", "enemy_1000_gopro", "enemy_1001_bigbo"])
    );
    let trap =
        json!({ "waves": [ { "fragments": [ { "actions": [ { "key": "trap_001_crate" } ] } ] } ] });
    assert_eq!(collect_level_enemy_ids(&trap), set(&["trap_001_crate"]));
    assert!(collect_level_enemy_ids(&json!({})).is_empty());
    assert!(collect_level_enemy_ids(&json!({ "enemyDbRefs": {}, "waves": "x" })).is_empty());
}

#[test]
fn only_a_missing_level_file_is_not_found() {
    let missing = std::io::Error::from(std::io::ErrorKind::NotFound);
    assert!(matches!(read_error(missing), AppError::NotFound(None)));
    let denied = std::io::Error::from(std::io::ErrorKind::PermissionDenied);
    assert!(matches!(read_error(denied), AppError::Internal(_)));
}

#[test]
fn camel_key_lowercases_the_first_char_and_drops_one_trailing_underscore() {
    assert_eq!(camel_key("MapData"), "mapData");
    assert_eq!(camel_key("Type_"), "type");
    assert_eq!(camel_key("alreadyCamel"), "alreadyCamel");
    assert_eq!(camel_key("ID"), "iD");
    assert_eq!(camel_key("_"), "");
    assert_eq!(camel_key(""), "");
    assert_eq!(camel_key("Key__"), "key_");
}

#[test]
fn camelize_keys_recurses_through_objects_and_arrays() {
    let raw = json!({
        "Options": { "CharacterLimit": 8, "Type_": "NORMAL" },
        "Waves": [ { "Fragments": [ { "Actions": [ { "Key": "enemy_1007_slime" } ] } ] } ],
        "Plain": "Value Stays",
    });
    assert_eq!(
        camelize_keys(raw),
        json!({
            "options": { "characterLimit": 8, "type": "NORMAL" },
            "waves": [ { "fragments": [ { "actions": [ { "key": "enemy_1007_slime" } ] } ] } ],
            "plain": "Value Stays",
        })
    );
    assert_eq!(camelize_keys(json!([1, "A", null])), json!([1, "A", null]));
}

#[test]
fn reshape_turns_the_flat_matrix_into_rows() {
    let mut level = json!({ "MapData": { "Map": {
        "Column_size": 3, "Row_size": 2, "Matrix_data": [1, 2, 3, 4, 5, 6],
    }, "Other": true } });
    reshape_map_matrix(&mut level);
    assert_eq!(
        level,
        json!({ "MapData": { "Map": [[1, 2, 3], [4, 5, 6]], "Other": true } })
    );

    let mut ragged =
        json!({ "MapData": { "Map": { "Column_size": 3, "Matrix_data": [1, 2, 3, 4] } } });
    reshape_map_matrix(&mut ragged);
    assert_eq!(ragged, json!({ "MapData": { "Map": [[1, 2, 3], [4]] } }));

    let mut empty = json!({ "MapData": { "Map": { "Column_size": 2 } } });
    reshape_map_matrix(&mut empty);
    assert_eq!(empty, json!({ "MapData": { "Map": [] } }));
}

#[test]
fn reshape_is_a_no_op_when_there_is_nothing_to_reshape() {
    for original in [
        json!({ "MapData": { "Map": [[0, 1], [1, 0]] } }),
        json!({ "MapData": { "Map": { "Column_size": 0, "Matrix_data": [1] } } }),
        json!({ "MapData": { "Map": { "Matrix_data": [1] } } }),
        json!({ "MapData": "not an object" }),
        json!({ "Other": 1 }),
    ] {
        let mut level = original.clone();
        reshape_map_matrix(&mut level);
        assert_eq!(level, original);
    }
}

#[tokio::test]
async fn community_pages_read_the_aggregates() {
    let Some((ctx, db, snapshot)) = common::context_with_data().await else {
        return;
    };
    let popularity = parse(&static_data::skin_popularity(&ctx).await.unwrap().body);
    assert_eq!(popularity["totalUsers"], 0);
    assert!(popularity["counts"].as_object().unwrap().is_empty());
    let average = parse(
        &static_data::community_enemy_average(&ctx)
            .await
            .unwrap()
            .body,
    );
    let visible = snapshot
        .model
        .enemies
        .enemy_data
        .values()
        .filter(|e| !e.hide_in_handbook)
        .count();
    assert_eq!(average["handbookTotal"], visible);
    assert_eq!(average["userCount"], 0);
    db.close().await;
}
