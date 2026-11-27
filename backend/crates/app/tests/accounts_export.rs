#![allow(clippy::unwrap_used)]

mod common;

use myrtle_app::AppError;
use myrtle_app::services::accounts::export::{
    ImportOptions, TableLoad, UserExport, export_user, import_user, parse_export, usable_columns,
    user_tables,
};
use myrtle_db::Db;
use myrtle_db::repos::gacha::{self, BatchOutcome};
use myrtle_db::tools;
use myrtle_testkit::db::database;
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};

const GROUP: &str = "6f1d3c1e-8a52-4c1b-9a43-1a2b3c4d5e6f";

async fn seed(db: &Db, user: UserId) {
    let mut conn = db.conn().await.unwrap();
    let pulls = json!([
        {"char_id": "char_a", "pool_id": "NORM_1", "rarity": 6, "pull_timestamp": 100,
         "pool_name": "Banner", "gacha_type": "normal", "batch_index": 0},
        {"char_id": "char_b", "pool_id": "NORM_1", "rarity": 4, "pull_timestamp": 100,
         "pool_name": "Banner", "gacha_type": "normal", "batch_index": 1},
    ]);
    assert_eq!(
        gacha::insert_batch(&mut conn, user, &pulls).await.unwrap(),
        BatchOutcome::Stored
    );
    drop(conn);

    let mut file = export_user(db, user, false).await.unwrap();
    let id = json!(user.0);
    file.tables.insert(
        "user_items".into(),
        vec![
            json!({"user_id": id, "item_id": "4001", "quantity": 1200}),
            json!({"user_id": id, "item_id": "30012", "quantity": 5}),
        ],
    );
    file.tables.insert(
        "plan_groups".into(),
        vec![json!({"id": GROUP, "user_id": id, "name": "E2 queue", "pinned": true})],
    );
    import_user(
        db,
        &file,
        ImportOptions {
            replace: true,
            ..ImportOptions::default()
        },
    )
    .await
    .unwrap();
}

fn comparable(mut file: UserExport) -> (Value, Value) {
    for row in file.tables.get_mut("gacha_records").unwrap() {
        row.as_object_mut().unwrap().remove("id");
    }
    (json!(file.user), json!(file.tables))
}

fn load(report: &myrtle_app::services::accounts::export::ImportReport, table: &str) -> TableLoad {
    report
        .tables
        .iter()
        .find(|(name, _)| *name == table)
        .map(|(_, load)| *load)
        .unwrap()
}

#[tokio::test]
async fn export_then_import_reproduces_the_rows_in_another_database() {
    let (ctx, source) = common::context().await;
    let user = source.user(7, Server::EN).await;
    seed(&ctx.db, user).await;
    let file = export_user(&ctx.db, user, false).await.unwrap();
    assert_eq!(file.counts["gacha_records"], 2);
    assert_eq!(file.counts["user_items"], 2);
    assert_eq!(file.counts["plan_groups"], 1);
    assert_eq!(file.source.uid, "7");
    assert_eq!(file.source.server_code, "en");
    assert!(!file.tables.contains_key("user_game_credentials"));
    let group = &file.tables["plan_groups"][0];
    assert!(
        group["created_at"].is_string(),
        "a missing column takes its DEFAULT"
    );

    let bytes = serde_json::to_vec(&file).unwrap();
    let target = database().await;
    let report = import_user(
        &target.db,
        &parse_export(&bytes).unwrap(),
        ImportOptions::default(),
    )
    .await
    .unwrap();
    assert!(report.committed);
    assert_eq!(report.existing, None);
    assert_eq!(
        report.target, user,
        "an empty database reuses the file's id"
    );
    assert_eq!(report.inserted(), 5);
    assert_eq!(
        load(&report, "gacha_records"),
        TableLoad::Loaded {
            inserted: 2,
            skipped: 0
        }
    );

    let copy = export_user(&target.db, user, false).await.unwrap();
    assert_eq!(comparable(copy), comparable(file));
    target.close().await;
    source.close().await;
}

#[tokio::test]
async fn an_account_holding_rows_needs_replace_and_keeps_its_id() {
    let (ctx, db) = common::context().await;
    let user = db.user(8, Server::EN).await;
    seed(&ctx.db, user).await;
    let mut file = export_user(&ctx.db, user, false).await.unwrap();
    file.source.id = uuid::Uuid::new_v4();
    file.tables.insert(
        "user_items".into(),
        vec![json!({"user_id": file.source.id, "item_id": "4001", "quantity": 1})],
    );

    let refused = import_user(&ctx.db, &file, ImportOptions::default()).await;
    assert!(
        matches!(&refused, Err(AppError::Conflict(m)) if m.contains("--replace")),
        "{refused:?}"
    );

    let options = ImportOptions {
        replace: true,
        ..ImportOptions::default()
    };
    let report = import_user(&ctx.db, &file, options).await.unwrap();
    assert_eq!(report.target, user);
    assert_eq!(report.existing.map(|(id, _)| id), Some(user));
    assert_eq!(report.existing.map(|(_, rows)| rows > 0), Some(true));
    let after = export_user(&ctx.db, user, false).await.unwrap();
    assert_eq!(
        after.tables["user_items"],
        [json!({"user_id": user.0, "item_id": "4001", "quantity": 1})]
    );
    db.close().await;
}

#[tokio::test]
async fn a_dry_run_reports_the_load_and_writes_nothing() {
    let (ctx, source) = common::context().await;
    let user = source.user(9, Server::EN).await;
    seed(&ctx.db, user).await;
    let file = export_user(&ctx.db, user, false).await.unwrap();
    let target = database().await;
    let dry = ImportOptions {
        dry_run: true,
        ..ImportOptions::default()
    };

    let report = import_user(&target.db, &file, dry).await.unwrap();
    assert!(!report.committed);
    assert_eq!(report.inserted(), 5);
    let mut conn = target.db.conn().await.unwrap();
    assert_eq!(
        tools::find_user_id(&mut conn, "9", Server::EN)
            .await
            .unwrap(),
        None
    );
    drop(conn);

    let before = export_user(&ctx.db, user, false).await.unwrap();
    let report = import_user(&ctx.db, &file, dry).await.unwrap();
    assert_eq!(
        report.existing.map(|(id, _)| id),
        Some(user),
        "no --replace needed"
    );
    let after = export_user(&ctx.db, user, false).await.unwrap();
    assert_eq!(json!(after.tables), json!(before.tables));
    target.close().await;
    source.close().await;
}

#[tokio::test]
async fn a_taken_id_is_minted_fresh_and_absent_tables_are_reported() {
    let (ctx, source) = common::context().await;
    let user = source.user(10, Server::EN).await;
    seed(&ctx.db, user).await;
    let mut file = export_user(&ctx.db, user, false).await.unwrap();
    let target = database().await;

    let mut other = file.clone();
    other.user.insert("uid".into(), json!("other"));
    other.tables.clear();
    import_user(&target.db, &other, ImportOptions::default())
        .await
        .unwrap();

    file.tables.remove("user_medals");
    file.tables.insert(
        "tier_list_favorites".into(),
        vec![json!({"tier_list_id": uuid::Uuid::new_v4(), "user_id": user.0})],
    );
    let report = import_user(&target.db, &file, ImportOptions::default())
        .await
        .unwrap();
    assert_ne!(report.target, user);
    assert_eq!(load(&report, "user_medals"), TableLoad::Absent);
    assert_eq!(
        load(&report, "tier_list_favorites"),
        TableLoad::Loaded {
            inserted: 0,
            skipped: 1
        }
    );
    assert_eq!(report.skipped(), 1);
    let copy = export_user(&target.db, report.target, false).await.unwrap();
    assert_eq!(copy.counts["user_items"], 2);
    assert_eq!(
        copy.tables["user_items"][0]["user_id"],
        json!(report.target.0)
    );
    target.close().await;
    source.close().await;
}

#[tokio::test]
async fn credentials_travel_only_when_asked() {
    let (ctx, db) = common::context().await;
    let user = db.user(11, Server::EN).await;
    let with = export_user(&ctx.db, user, true).await.unwrap();
    assert!(with.includes_credentials);
    assert_eq!(with.counts["user_game_credentials"], 0);
    assert_eq!(
        user_tables(true).last().unwrap().name,
        "user_game_credentials"
    );
    assert_eq!(user_tables(false).len() + 1, user_tables(true).len());
    db.close().await;
}

#[tokio::test]
async fn usable_columns_intersect_the_file_and_the_live_table() {
    let (ctx, db) = common::context().await;
    let mut conn = ctx.db.conn().await.unwrap();
    let table = tools::table(&mut conn, "gacha_records")
        .await
        .unwrap()
        .unwrap();
    let live = tools::columns(&mut conn, &table).await.unwrap();
    let rows = [
        json!({"id": 1, "user_id": "x", "char_id": "c", "dropped_long_ago": 1}),
        json!({"rarity": 6}),
    ];
    let names: Vec<String> = usable_columns(live, &rows, &["id"])
        .iter()
        .map(|c| c.name().to_owned())
        .collect();
    assert_eq!(names, ["user_id", "char_id", "rarity"]);
    drop(conn);
    db.close().await;
}

#[test]
fn parse_export_refuses_other_files() {
    let ok = json!({
        "kind": "myrtle-user-export", "format_version": 1, "exported_at": "x",
        "source": {"id": uuid::Uuid::nil(), "uid": "1", "server_id": 0, "server_code": "EN",
                   "nickname": null, "database_version": "16"},
        "includes_credentials": false, "user": {"uid": "1", "server_id": 0},
        "counts": {}, "tables": {}
    });
    assert!(parse_export(ok.to_string().as_bytes()).is_ok());
    let mut kind = ok.clone();
    kind["kind"] = json!("myrtle-db-export");
    assert!(
        matches!(parse_export(kind.to_string().as_bytes()), Err(AppError::BadRequest(m)) if m.contains("kind"))
    );
    let mut version = ok;
    version["format_version"] = json!(2);
    assert!(
        matches!(parse_export(version.to_string().as_bytes()), Err(AppError::BadRequest(m)) if m.contains("format_version"))
    );
    assert!(parse_export(b"[]").is_err());
}
