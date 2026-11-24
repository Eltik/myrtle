#![allow(clippy::unwrap_used)]

use std::collections::BTreeSet;

use myrtle_app::dump::{
    self, DumpError, FORMAT_VERSION, MANIFEST_FILE, Manifest, NOT_EXPORTED, SEEDED_TABLES,
    SERIAL_COLUMNS, TABLES, TableEntry, check_manifest, closure, upgrade_legacy_row,
};
use myrtle_db::tools::ForeignKey;
use serde_json::{Value, json};

fn manifest(tables: &[(&str, u64)]) -> Manifest {
    Manifest {
        format_version: FORMAT_VERSION,
        exported_at: "2026-01-02T03:04:05+00:00".to_owned(),
        database_version: "17.2".to_owned(),
        tables: tables.iter().map(|(t, n)| TableEntry::new(t, *n)).collect(),
    }
}

fn edge(child: &str, parent: &str) -> ForeignKey {
    ForeignKey {
        child: child.to_owned(),
        parent: parent.to_owned(),
    }
}

#[test]
fn every_migrated_table_is_exported_or_excluded() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../db/migrations");
    let create = regex::Regex::new(
        r#"create\s+(?:unlogged\s+)?table\s+(?:if\s+not\s+exists\s+)?(?:public\.)?"?([a-z0-9_]+)"#,
    )
    .unwrap();
    let mut created = BTreeSet::new();
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|e| e != "sql") {
            continue;
        }
        let sql = std::fs::read_to_string(&path).unwrap().to_lowercase();
        assert!(
            !sql.contains("drop table") && !sql.contains("rename to"),
            "{}: teach this test about dropped or renamed tables",
            path.display()
        );
        for c in create.captures_iter(&sql) {
            created.insert(c[1].to_owned());
        }
    }
    assert!(
        created.contains("music_stat_dedupe"),
        "unlogged tables count"
    );
    assert!(created.contains("jobs"));

    let excluded: Vec<&str> = NOT_EXPORTED.iter().map(|(t, _)| *t).collect();
    assert_eq!(TABLES.len(), TABLES.iter().collect::<BTreeSet<_>>().len());
    for t in &excluded {
        assert!(!TABLES.contains(t), "{t} is both exported and excluded");
    }
    let listed: BTreeSet<String> = TABLES
        .iter()
        .chain(&excluded)
        .map(|t| (*t).to_owned())
        .collect();
    let missing: Vec<_> = created.difference(&listed).collect();
    assert!(
        missing.is_empty(),
        "add to dump::TABLES or NOT_EXPORTED: {missing:?}"
    );
    let stale: Vec<_> = listed
        .difference(&created)
        .filter(|t| *t != "_sqlx_migrations")
        .collect();
    assert!(stale.is_empty(), "no migration creates: {stale:?}");
    for t in SEEDED_TABLES
        .iter()
        .chain(SERIAL_COLUMNS.iter().map(|(t, _)| t))
    {
        assert!(TABLES.contains(t), "{t} is not in TABLES");
    }
}

#[test]
fn a_pre_v029_placement_imports_as_an_operator() {
    let mut row = json!({
        "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
        "operator_id": "char_002_amiya",
        "sub_order": 0,
        "description": null,
        "updated_at": "2026-01-02T03:04:05Z"
    });
    upgrade_legacy_row("tier_placements", &mut row);
    assert_eq!(row["entity_id"], "char_002_amiya");
    assert_eq!(row["entity_kind"], "operator");
    assert!(row.get("operator_id").is_none());
}

#[test]
fn a_current_placement_is_left_alone() {
    let before = json!({
        "tier_id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a01",
        "entity_kind": "enemy",
        "entity_id": "enemy_1007_slime",
        "sub_order": 0
    });
    let mut row = before.clone();
    upgrade_legacy_row("tier_placements", &mut row);
    assert_eq!(row, before);
}

#[test]
fn a_pre_v029_list_offers_operators() {
    let mut row = json!({"id": "6f1d0c52-6f43-4a39-9d0e-6b8f0f6c1a02", "name": "x"});
    upgrade_legacy_row("tier_lists", &mut row);
    assert_eq!(row["entity_kinds"], json!(["operator"]));
    let mut current = json!({"entity_kinds": ["operator", "enemy"]});
    upgrade_legacy_row("tier_lists", &mut current);
    assert_eq!(current["entity_kinds"], json!(["operator", "enemy"]));
    let mut other = json!({"operator_id": "char_002_amiya"});
    upgrade_legacy_row("operator_notes", &mut other);
    assert_eq!(other, json!({"operator_id": "char_002_amiya"}));
    let mut scalar = json!(3);
    upgrade_legacy_row("tier_lists", &mut scalar);
    assert_eq!(scalar, json!(3));
}

#[test]
fn closure_walks_parents_or_children_transitively() {
    let edges = [
        edge("tiers", "tier_lists"),
        edge("tier_placements", "tiers"),
        edge("tier_lists", "users"),
        edge("users", "servers"),
        edge("gacha_records", "users"),
    ];
    let up = closure(["tier_placements"], &edges, true);
    assert_eq!(
        up.into_iter().collect::<Vec<_>>(),
        ["servers", "tier_lists", "tier_placements", "tiers", "users"]
    );
    let down = closure(["users"], &edges, false);
    assert_eq!(
        down.into_iter().collect::<Vec<_>>(),
        [
            "gacha_records",
            "tier_lists",
            "tier_placements",
            "tiers",
            "users"
        ]
    );
    let alone = closure(["servers"], &edges, true);
    assert_eq!(alone.into_iter().collect::<Vec<_>>(), ["servers"]);
}

#[test]
fn a_manifest_must_be_a_subsequence_of_tables_under_its_own_file_names() {
    check_manifest(&manifest(&[])).unwrap();
    check_manifest(&manifest(&[("servers", 6), ("users", 2), ("audit_log", 0)])).unwrap();
    let all: Vec<(&str, u64)> = TABLES.iter().map(|t| (*t, 0)).collect();
    check_manifest(&manifest(&all)).unwrap();

    let refused = |m: &Manifest| match check_manifest(m) {
        Err(DumpError::Invalid(message)) => message,
        other => panic!("expected a refusal, got {other:?}"),
    };
    assert!(refused(&manifest(&[("users", 1), ("servers", 6)])).contains("out of order"));
    assert!(refused(&manifest(&[("users", 1), ("users", 1)])).contains("out of order or twice"));
    assert!(refused(&manifest(&[("jobs", 1)])).contains("does not know"));
    let mut renamed = manifest(&[("users", 1)]);
    renamed.tables[0].file = "people.jsonl".to_owned();
    assert!(refused(&renamed).contains("unexpected filename"));
    let mut newer = manifest(&[]);
    newer.format_version = FORMAT_VERSION + 1;
    assert!(refused(&newer).contains("format_version"));
}

#[test]
fn the_manifest_file_matches_mains_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let m = manifest(&[("servers", 6), ("users", 2)]);
    dump::write_manifest(dir.path(), &m).unwrap();
    let written = std::fs::read_to_string(dir.path().join(MANIFEST_FILE)).unwrap();
    let main = r#"{
  "database_version": "17.2",
  "exported_at": "2026-01-02T03:04:05+00:00",
  "format_version": 1,
  "tables": [
    {
      "file": "servers.jsonl",
      "name": "servers",
      "rows": 6
    },
    {
      "file": "users.jsonl",
      "name": "users",
      "rows": 2
    }
  ]
}"#;
    assert_eq!(written, main);
    assert_eq!(dump::read_manifest(dir.path()).unwrap(), m);
}

#[test]
fn a_manifest_with_extra_keys_still_reads() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(MANIFEST_FILE),
        r#"{"format_version":1,"exported_at":"x","database_version":"y","note":"z",
            "tables":[{"name":"users","rows":3,"file":"users.jsonl"}]}"#,
    )
    .unwrap();
    let m = dump::read_manifest(dir.path()).unwrap();
    assert_eq!(m.tables, [TableEntry::new("users", 3)]);
    assert!(matches!(
        dump::read_manifest(&dir.path().join("nowhere")),
        Err(DumpError::Io { .. })
    ));
}

fn streamed(text: &str) -> Result<Vec<Value>, DumpError> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.json");
    std::fs::write(&path, text).unwrap();
    let mut items = Vec::new();
    dump::stream_array(&path, |v| {
        items.push(v);
        Ok(())
    })?;
    Ok(items)
}

#[test]
fn streaming_an_array_yields_each_element_whole() {
    let text = r#"  [ {"a": "]}\"[{", "b": [1, {"c": null}]},
        {"d": "\\"} , [2,3], "s,]", 4 ,true]"#;
    assert_eq!(
        streamed(text).unwrap(),
        [
            json!({"a": "]}\"[{", "b": [1, {"c": null}]}),
            json!({"d": "\\"}),
            json!([2, 3]),
            json!("s,]"),
            json!(4),
            json!(true),
        ]
    );
    assert!(streamed("[]").unwrap().is_empty());
    assert!(streamed("[\n]\n").unwrap().is_empty());
}

#[test]
fn a_truncated_or_non_array_stream_is_refused() {
    for text in [r#"[{"a": 1}, {"b": "#, r#"[{"a": 1}"#, "", r#"{"a": 1}"#] {
        assert!(streamed(text).is_err(), "{text:?}");
    }
}

#[test]
fn a_callback_error_stops_the_stream() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("a.json");
    std::fs::write(&path, "[1, 2, 3]").unwrap();
    let mut seen = 0;
    let result = dump::stream_array(&path, |_| {
        seen += 1;
        Err(DumpError::Invalid("stop".to_owned()))
    });
    assert!(result.is_err());
    assert_eq!(seen, 1);
}
