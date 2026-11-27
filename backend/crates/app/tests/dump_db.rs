#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::Path;

use myrtle_app::dump::{
    self, DumpError, ImportOptions, MANIFEST_FILE, NOT_EXPORTED, SEEDED_TABLES, SERIAL_COLUMNS,
    Selection, TABLES,
};
use myrtle_db::repos::gacha::{self, BatchOutcome};
use myrtle_db::tools;
use myrtle_db::{Conn, Db};
use myrtle_testkit::db::{TestDb, database};
use myrtle_types::{Server, UserId};
use serde_json::{Value, json};

async fn count(db: &Db, name: &str) -> i64 {
    let mut conn = db.conn().await.unwrap();
    let table = tools::table(&mut conn, name).await.unwrap().unwrap();
    tools::count_rows(&mut conn, &table).await.unwrap()
}

async fn counts(db: &Db) -> BTreeMap<&'static str, i64> {
    let mut all = BTreeMap::new();
    for &t in TABLES {
        all.insert(t, count(db, t).await);
    }
    all
}

async fn restore(conn: &mut Conn, name: &str, rows: Value) {
    let table = tools::table(conn, name).await.unwrap().unwrap();
    tools::restore_rows(conn, &table, &rows).await.unwrap();
}

async fn scalar(db: &Db, sql: &'static str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(&mut *db.conn().await.unwrap())
        .await
        .unwrap()
}

fn pull(char_id: &str, ts: i64) -> Value {
    json!({
        "char_id": char_id,
        "pool_id": "NORM_0_0_1",
        "rarity": 5,
        "pull_timestamp": ts,
        "pool_name": "Standard",
        "gacha_type": "normal",
        "batch_index": 0,
    })
}

/// users (and the audit rows their trigger writes), sequence-assigned pulls, an extra
/// locale beside the seeded one, a data generation
async fn populated() -> (TestDb, UserId) {
    let test = database().await;
    let user = test.user(1, Server::EN).await;
    test.user(2, Server::CN).await;
    let mut conn = test.db.conn().await.unwrap();
    let stored = gacha::insert_batch(
        &mut conn,
        user,
        &json!([
            pull("char_002_amiya", 100),
            pull("char_003_kalts", 100),
            pull("char_017_huang", 200)
        ]),
    )
    .await
    .unwrap();
    assert_eq!(stored, BatchOutcome::Stored);
    restore(
        &mut conn,
        "locales",
        json!([{
            "code": "ja", "english_name": "Japanese", "native_name": "日本語",
            "fallback_locale": "en", "gamedata_server": "jp", "enabled": true,
            "sort_order": 3, "created_at": "2026-01-02T03:04:05Z", "is_source": false
        }]),
    )
    .await;
    restore(
        &mut conn,
        "data_generations",
        json!([{
            "server_id": 0, "generation": 7, "res_version": "26-01-01", "pipeline_seq": 3,
            "recorded_seq": 2, "updated_at": "2026-01-02T03:04:05Z"
        }]),
    )
    .await;
    (test, user)
}

/// each table's rows, sorted, as `row_to_json` writes them
fn contents(dir: &Path) -> BTreeMap<String, Vec<String>> {
    let manifest = dump::read_manifest(dir).unwrap();
    manifest
        .tables
        .iter()
        .map(|t| {
            let text = std::fs::read_to_string(dir.join(&t.file)).unwrap();
            let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
            lines.sort();
            (t.name.clone(), lines)
        })
        .collect()
}

#[tokio::test]
async fn a_round_trip_restores_every_row_and_advances_the_sequences() {
    let (source, user) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let exported = dump::export(&source.db, dir.path(), &Selection::All)
        .await
        .unwrap();
    assert_eq!(exported.manifest.tables.len(), TABLES.len());
    assert!(exported.left_out.is_empty());
    let source_counts = counts(&source.db).await;
    for entry in &exported.manifest.tables {
        assert_eq!(
            i64::try_from(entry.rows).unwrap(),
            source_counts[entry.name.as_str()],
            "{}",
            entry.name
        );
    }
    assert_eq!(source_counts["gacha_records"], 3);
    assert_eq!(source_counts["locales"], 2);
    assert_eq!(source_counts["servers"], 6);
    assert!(
        source_counts["audit_log"] >= 2,
        "the users trigger wrote audit rows"
    );

    let target = database().await;
    let report = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap();
    assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
    assert!(report.skipped.is_empty() && report.kept_seeded.is_empty());
    assert_eq!(report.truncated, 0);
    assert_eq!(counts(&target.db).await, source_counts);

    let again = tempfile::tempdir().unwrap();
    dump::export(&target.db, again.path(), &Selection::All)
        .await
        .unwrap();
    let (before, after) = (contents(dir.path()), contents(again.path()));
    for t in ["gacha_records", "locales", "servers", "users", "audit_log"] {
        assert!(!before[t].is_empty(), "{t}");
    }
    assert_eq!(before, after);
    for t in ["data_generations", "service_account_sessions"] {
        assert!(!dir.path().join(format!("{t}.jsonl")).exists(), "{t}");
    }
    assert_eq!(count(&source.db, "data_generations").await, 1);
    assert_eq!(count(&target.db, "data_generations").await, 0);

    let mut conn = target.db.conn().await.unwrap();
    let stored = gacha::insert_batch(&mut conn, user, &json!([pull("char_002_amiya", 300)]))
        .await
        .unwrap();
    assert_eq!(stored, BatchOutcome::Stored);
    drop(conn);
    assert_eq!(
        scalar(&target.db, "SELECT max(id) FROM gacha_records").await,
        4
    );
    source.close().await;
    target.close().await;
}

#[tokio::test]
async fn an_occupied_database_is_refused_unless_truncated() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let exported = dump::export(&source.db, dir.path(), &Selection::All)
        .await
        .unwrap();
    let before = counts(&source.db).await;

    let refused = dump::import(&source.db, dir.path(), ImportOptions::default()).await;
    match refused {
        Err(DumpError::Invalid(message)) => {
            assert!(message.contains("refusing to import"), "{message}");
            assert!(message.contains("--truncate"), "{message}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(counts(&source.db).await, before);

    let mut options = ImportOptions {
        truncate: true,
        batch_size: NonZeroUsize::new(2).unwrap(),
        ..ImportOptions::default()
    };
    match dump::import(&source.db, dir.path(), options).await {
        Err(DumpError::Invalid(message)) => {
            assert!(message.contains("data_generations (1 rows)"), "{message}");
            assert!(message.contains("--cascade"), "{message}");
            assert!(
                !message.contains("oauth_codes"),
                "empty tables lose nothing"
            );
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(count(&source.db, "data_generations").await, 1);

    options.cascade = true;
    let report = dump::import(&source.db, dir.path(), options).await.unwrap();
    assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
    assert_eq!(report.truncated, exported.manifest.tables.len());
    let mut cascaded = report.cascaded.clone();
    cascaded.sort();
    assert_eq!(
        cascaded,
        [
            "data_generations",
            "oauth_codes",
            "service_account_sessions"
        ]
    );
    assert_eq!(count(&source.db, "data_generations").await, 0);
    assert_eq!(counts(&source.db).await, before);
    source.close().await;
}

#[tokio::test]
async fn a_truncate_refuses_to_cascade_into_rows_it_does_not_restore() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let lists = Selection::Tables {
        names: vec!["tier_lists".to_owned()],
        with_parents: true,
    };
    dump::export(&source.db, dir.path(), &lists).await.unwrap();
    let options = ImportOptions {
        truncate: true,
        ..ImportOptions::default()
    };
    match dump::import(&source.db, dir.path(), options).await {
        Err(DumpError::Invalid(message)) => {
            assert!(message.contains("gacha_records (3 rows)"), "{message}");
            assert!(message.contains("data_generations (1 rows)"), "{message}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(count(&source.db, "gacha_records").await, 3);

    let report = dump::import(
        &source.db,
        dir.path(),
        ImportOptions {
            cascade: true,
            ..options
        },
    )
    .await
    .unwrap();
    assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
    assert!(report.cascaded.iter().any(|t| t == "gacha_records"));
    assert_eq!(count(&source.db, "gacha_records").await, 0);
    assert_eq!(count(&source.db, "users").await, 2);
    source.close().await;
}

#[tokio::test]
async fn replacing_a_seeded_table_refuses_to_orphan_rows() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let servers = Selection::Tables {
        names: vec!["servers".to_owned()],
        with_parents: false,
    };
    dump::export(&source.db, dir.path(), &servers)
        .await
        .unwrap();
    match dump::import(&source.db, dir.path(), ImportOptions::default()).await {
        Err(DumpError::Invalid(message)) => {
            assert!(message.contains("replacing servers"), "{message}");
            assert!(message.contains("users (2 rows)"), "{message}");
        }
        other => panic!("expected a refusal, got {other:?}"),
    }
    assert_eq!(count(&source.db, "servers").await, 6);

    let target = database().await;
    let report = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap();
    assert!(report.mismatches.is_empty());
    assert_eq!(count(&target.db, "servers").await, 6);
    source.close().await;
    target.close().await;
}

#[tokio::test]
async fn a_skipped_table_keeps_its_sequence() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let locales = Selection::Tables {
        names: vec!["locales".to_owned()],
        with_parents: false,
    };
    dump::export(&source.db, dir.path(), &locales)
        .await
        .unwrap();

    let target = database().await;
    let target_user = target.user(1, Server::EN).await;
    let mut conn = target.db.conn().await.unwrap();
    for ts in [1, 2, 3] {
        let stored =
            gacha::insert_batch(&mut conn, target_user, &json!([pull("char_002_amiya", ts)]))
                .await
                .unwrap();
        assert_eq!(stored, BatchOutcome::Stored);
    }
    sqlx::query("DELETE FROM gacha_records")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);

    let report = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap();
    assert!(report.skipped.contains(&"gacha_records"));
    let mut conn = target.db.conn().await.unwrap();
    let stored = gacha::insert_batch(&mut conn, target_user, &json!([pull("char_003_kalts", 9)]))
        .await
        .unwrap();
    assert_eq!(stored, BatchOutcome::Stored);
    drop(conn);
    assert_eq!(
        scalar(&target.db, "SELECT max(id) FROM gacha_records").await,
        4,
        "a skipped table's sequence must not restart at 1"
    );
    source.close().await;
    target.close().await;
}

#[tokio::test]
async fn an_export_into_a_used_directory_drops_the_old_manifest_first() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    dump::export(&source.db, dir.path(), &Selection::All)
        .await
        .unwrap();
    assert!(dir.path().join(MANIFEST_FILE).is_file());

    std::fs::remove_file(dir.path().join("users.jsonl")).unwrap();
    std::fs::create_dir(dir.path().join("users.jsonl")).unwrap();
    assert!(matches!(
        dump::export(&source.db, dir.path(), &Selection::All).await,
        Err(DumpError::Io { .. })
    ));
    assert!(
        !dir.path().join(MANIFEST_FILE).exists(),
        "an interrupted export has no manifest"
    );
    source.close().await;
}

#[tokio::test]
async fn a_partial_export_names_or_adds_the_tables_it_references() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    let only = Selection::Tables {
        names: vec!["gacha_records".to_owned()],
        with_parents: false,
    };
    let report = dump::export(&source.db, dir.path(), &only).await.unwrap();
    let names: Vec<&str> = report
        .manifest
        .tables
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names, ["gacha_records"]);
    assert_eq!(report.left_out, ["servers", "users"]);

    let widened = tempfile::tempdir().unwrap();
    let with_parents = Selection::Tables {
        names: vec!["gacha_records".to_owned(), "locales".to_owned()],
        with_parents: true,
    };
    let report = dump::export(&source.db, widened.path(), &with_parents)
        .await
        .unwrap();
    let names: Vec<&str> = report
        .manifest
        .tables
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert_eq!(names, ["servers", "users", "gacha_records", "locales"]);
    assert_eq!(report.added, ["servers", "users"]);
    assert!(report.left_out.is_empty());

    for names in [vec!["jobs".to_owned()], vec!["nope".to_owned()], vec![]] {
        let bad = Selection::Tables {
            names,
            with_parents: false,
        };
        assert!(matches!(
            dump::export(&source.db, dir.path(), &bad).await,
            Err(DumpError::Invalid(_))
        ));
    }

    let target = database().await;
    let report = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap();
    assert!(report.mismatches.is_empty());
    assert!(report.skipped.contains(&"users"));
    assert_eq!(count(&target.db, "gacha_records").await, 3);
    assert_eq!(count(&target.db, "users").await, 0);
    source.close().await;
    target.close().await;
}

#[tokio::test]
async fn a_seeded_table_without_rows_keeps_its_migration_rows() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = json!({
        "format_version": 1, "exported_at": "x", "database_version": "y",
        "tables": [
            {"name": "servers", "rows": 0, "file": "servers.jsonl"},
            {"name": "users", "rows": 1, "file": "users.jsonl"},
            {"name": "tier_lists", "rows": 1, "file": "tier_lists.jsonl"},
            {"name": "tiers", "rows": 1, "file": "tiers.jsonl"},
            {"name": "tier_placements", "rows": 1, "file": "tier_placements.jsonl"},
            {"name": "locales", "rows": 0, "file": "locales.jsonl"}
        ]
    });
    std::fs::write(dir.path().join(MANIFEST_FILE), manifest.to_string()).unwrap();
    let user = "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a00";
    let list = "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01";
    let tier = "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02";
    let files = [
        ("servers", String::new()),
        ("locales", String::new()),
        (
            "users",
            json!({"id": user, "uid": "9", "server_id": 3, "nickname": "Doc", "level": 120,
                   "role": "user", "created_at": "2026-01-02T03:04:05Z",
                   "updated_at": "2026-01-02T03:04:05Z"})
            .to_string(),
        ),
        (
            "tier_lists",
            json!({"id": list, "name": "Old", "slug": "old", "list_type": "official",
                   "created_by": user, "is_active": true, "is_listed": true,
                   "created_at": "2026-01-02T03:04:05Z", "updated_at": "2026-01-02T03:04:05Z"})
            .to_string(),
        ),
        (
            "tiers",
            json!({"id": tier, "tier_list_id": list, "name": "S", "display_order": 0}).to_string(),
        ),
        (
            "tier_placements",
            json!({"tier_id": tier, "operator_id": "char_002_amiya", "sub_order": 0,
                   "updated_at": "2026-01-02T03:04:05Z"})
            .to_string(),
        ),
    ];
    for (t, body) in files {
        std::fs::write(dir.path().join(format!("{t}.jsonl")), body + "\n").unwrap();
    }

    for truncate in [false, true] {
        let target = database().await;
        let options = ImportOptions {
            truncate,
            ..ImportOptions::default()
        };
        let report = dump::import(&target.db, dir.path(), options).await.unwrap();
        assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
        assert_eq!(report.kept_seeded, ["servers", "locales"]);
        assert_eq!(count(&target.db, "servers").await, 6);
        assert_eq!(count(&target.db, "locales").await, 1);
        assert_eq!(count(&target.db, "users").await, 1);
        let (kind, id): (String, String) =
            sqlx::query_as("SELECT entity_kind, entity_id FROM tier_placements")
                .fetch_one(&mut *target.db.conn().await.unwrap())
                .await
                .unwrap();
        assert_eq!((kind.as_str(), id.as_str()), ("operator", "char_002_amiya"));
        assert_eq!(
            scalar(
                &target.db,
                "SELECT cardinality(entity_kinds)::bigint FROM tier_lists"
            )
            .await,
            1
        );
        target.close().await;
    }
}

#[tokio::test]
async fn a_broken_export_changes_nothing() {
    let (source, _) = populated().await;
    let dir = tempfile::tempdir().unwrap();
    dump::export(&source.db, dir.path(), &Selection::All)
        .await
        .unwrap();
    let target = database().await;
    let before = counts(&target.db).await;

    std::fs::remove_file(dir.path().join("gacha_records.jsonl")).unwrap();
    assert!(matches!(
        dump::import(&target.db, dir.path(), ImportOptions::default()).await,
        Err(DumpError::Invalid(m)) if m.contains("missing file")
    ));
    std::fs::write(
        dir.path().join("gacha_records.jsonl"),
        "{\"id\": 1, \"user_id\": null}\n",
    )
    .unwrap();
    assert!(matches!(
        dump::import(&target.db, dir.path(), ImportOptions::default()).await,
        Err(DumpError::Table { table, .. }) if table == "gacha_records"
    ));
    assert_eq!(counts(&target.db).await, before);

    std::fs::remove_file(dir.path().join(MANIFEST_FILE)).unwrap();
    assert!(matches!(
        dump::import(&target.db, dir.path(), ImportOptions::default()).await,
        Err(DumpError::Io { .. })
    ));
    source.close().await;
    target.close().await;
}

#[tokio::test]
async fn every_live_table_is_exported_or_excluded() {
    let test = database().await;
    let mut conn = test.db.conn().await.unwrap();
    let live: Vec<String> = tools::table_estimates(&mut conn)
        .await
        .unwrap()
        .into_iter()
        .map(|e| e.table.name().to_owned())
        .collect();
    for t in &live {
        assert!(
            TABLES.contains(&t.as_str()) || NOT_EXPORTED.iter().any(|(n, _)| n == t),
            "{t} is neither exported nor excluded"
        );
    }
    for t in TABLES {
        assert!(live.iter().any(|l| l == t), "{t} is not in the live schema");
    }
    drop(conn);
    test.close().await;
}

#[tokio::test]
async fn tables_come_parents_first() {
    let test = database().await;
    let edges = tools::foreign_keys(&mut test.db.conn().await.unwrap())
        .await
        .unwrap();
    let at = |t: &str| TABLES.iter().position(|x| *x == t);
    for e in &edges {
        if let (Some(child), Some(parent)) = (at(&e.child), at(&e.parent)) {
            assert!(parent < child, "{} must come before {}", e.parent, e.child);
        }
    }
    assert!(
        edges
            .iter()
            .any(|e| e.child == "users" && e.parent == "servers")
    );
    test.close().await;
}

#[tokio::test]
async fn every_sequence_an_exported_table_owns_is_reset() {
    let test = database().await;
    let owned: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT c.table_name::text, c.column_name::text, c.is_identity::text
         FROM information_schema.columns c
         WHERE c.table_schema = 'public'
           AND (pg_get_serial_sequence(format('public.%I', c.table_name), c.column_name) IS NOT NULL)",
    )
    .fetch_all(&mut *test.db.conn().await.unwrap())
    .await
    .unwrap();
    let mut found: Vec<(&str, &str)> = owned
        .iter()
        .filter(|(t, _, _)| TABLES.contains(&t.as_str()))
        .map(|(t, c, _)| (t.as_str(), c.as_str()))
        .collect();
    found.sort_unstable();
    let mut listed = SERIAL_COLUMNS.to_vec();
    listed.sort_unstable();
    assert_eq!(found, listed);
    for (t, c, identity) in &owned {
        assert!(
            identity == "NO" || !TABLES.contains(&t.as_str()),
            "{t}.{c} is an identity column; a restore would need OVERRIDING SYSTEM VALUE"
        );
    }
    assert!(SEEDED_TABLES.iter().all(|t| TABLES.contains(t)));
    test.close().await;
}

#[tokio::test]
async fn a_gzipped_table_file_is_read_in_place() {
    let dir = tempfile::tempdir().unwrap();
    let manifest = json!({
        "format_version": 1, "exported_at": "x", "database_version": "y",
        "tables": [
            {"name": "servers", "rows": 0, "file": "servers.jsonl"},
            {"name": "users", "rows": 2, "file": "users.jsonl"}
        ]
    });
    std::fs::write(dir.path().join(MANIFEST_FILE), manifest.to_string()).unwrap();
    std::fs::write(dir.path().join("servers.jsonl"), "").unwrap();
    let users: String = [
        "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a10",
        "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a11",
    ]
    .iter()
    .enumerate()
    .map(|(n, id)| {
        json!({"id": id, "uid": n.to_string(), "server_id": 0, "nickname": "Doc",
                   "level": 1, "role": "user", "created_at": "2026-01-02T03:04:05Z",
                   "updated_at": "2026-01-02T03:04:05Z"})
        .to_string()
            + "\n"
    })
    .collect();
    let file = std::fs::File::create(dir.path().join("users.jsonl.gz")).unwrap();
    let mut gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
    std::io::Write::write_all(&mut gz, users.as_bytes()).unwrap();
    gz.finish().unwrap();

    let target = database().await;
    let report = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap();
    assert!(report.mismatches.is_empty(), "{:?}", report.mismatches);
    assert_eq!(count(&target.db, "users").await, 2);

    std::fs::remove_file(dir.path().join("users.jsonl.gz")).unwrap();
    let missing = dump::import(&target.db, dir.path(), ImportOptions::default())
        .await
        .unwrap_err();
    assert!(missing.to_string().contains("missing file"), "{missing}");
    target.close().await;
}
