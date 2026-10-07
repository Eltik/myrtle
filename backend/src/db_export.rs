//! Shared table metadata for the `export-database` and `import-database` binaries.
//!
//! TABLES is the topological order dictated by FK constraints: parents first,
//! children second, `audit_log` last. Export writes in this order; import replays
//! it unchanged. A table is only ever appended or inserted, never reordered, so
//! an older export's table list stays a subsequence of this one and still imports.
//!
//! Every table a migration creates belongs here; the test at the bottom fails
//! when one is missing.

pub const TABLES: &[&str] = &[
    "servers",
    "users",
    "user_status",
    "user_settings",
    "user_operators",
    "user_operator_skills",
    "user_operator_modules",
    "user_items",
    "user_skins",
    "user_stage_progress",
    "user_roguelike_progress",
    "user_sandbox_progress",
    "user_medals",
    "user_building",
    "user_checkin",
    "user_scores",
    "user_support_units",
    "user_enemy_progress",
    "user_story_progress",
    "user_game_story_read",
    "user_game_credentials",
    "gacha_records",
    "tier_list_flairs",
    "tier_lists",
    "tiers",
    "tier_placements",
    "tier_list_versions",
    "tier_list_permissions",
    "tier_list_stats",
    "tier_list_view_events",
    "tier_list_favorites",
    "operator_notes",
    "operator_notes_audit_log",
    "leaderboard_snapshots",
    "leaderboard_snapshot_entries",
    "operator_plans",
    "plan_groups",
    "plan_group_members",
    "plan_presets",
    "release_plans",
    "grids",
    "locales",
    "ui_message_keys",
    "ui_messages",
    "ui_documents",
    "ui_message_audit_log",
    "translation_permissions",
    "gamedata_overrides",
    "gamedata_sightings",
    "release_overrides",
    "operator_ownership_stats",
    "operator_skill_choice_stats",
    "operator_mastery_stats",
    "operator_module_choice_stats",
    "operator_module_level_stats",
    "medal_ownership_stats",
    "trevor_versions",
    "trevor_panels",
    "trevor_answers",
    "trevor_jobs",
    "trevor_feedback",
    "audit_log",
];

/// Tables a migration fills with rows of its own (v021 inserts the `locales`
/// rows). A freshly migrated database is not empty there, so import replaces
/// their rows instead of refusing to run.
pub const SEEDED_TABLES: &[&str] = &["locales"];

/// (table, serial column): sequences reset after import so new inserts don't
/// collide with restored ids.
pub const SERIAL_COLUMNS: &[(&str, &str)] = &[
    ("gacha_records", "id"),
    ("tier_list_flairs", "id"),
    ("tier_list_view_events", "id"),
    ("operator_notes_audit_log", "id"),
    ("leaderboard_snapshots", "id"),
    ("ui_message_audit_log", "id"),
    ("audit_log", "id"),
];

/// Every (child, parent) foreign-key edge between two different public
/// tables, read from the live schema so it can never drift from the migrations.
pub async fn foreign_keys(
    conn: &mut sqlx::PgConnection,
) -> Result<Vec<(String, String)>, sqlx::Error> {
    sqlx::query_as(
        "SELECT DISTINCT conrelid::regclass::text, confrelid::regclass::text \
         FROM pg_constraint \
         WHERE contype = 'f' AND connamespace = 'public'::regnamespace \
           AND conrelid <> confrelid",
    )
    .fetch_all(conn)
    .await
}

/// `start` plus every table reachable from it by following `edges` from one
/// end to the other: `(child, parent)` edges walked child-to-parent give the
/// tables a selection references, walked parent-to-child the tables a
/// `TRUNCATE ... CASCADE` reaches.
pub fn closure<'a>(
    start: impl IntoIterator<Item = &'a str>,
    edges: &'a [(String, String)],
    towards_parent: bool,
) -> std::collections::BTreeSet<&'a str> {
    let mut seen: std::collections::BTreeSet<&str> = start.into_iter().collect();
    let mut queue: Vec<&str> = seen.iter().copied().collect();
    while let Some(t) = queue.pop() {
        for (child, parent) in edges {
            let (from, to) = if towards_parent {
                (child, parent)
            } else {
                (parent, child)
            };
            if from == t && seen.insert(to.as_str()) {
                queue.push(to);
            }
        }
    }
    seen
}

pub const MANIFEST_FILE: &str = "manifest.json";
pub const FORMAT_VERSION: u32 = 1;

/// Bring a row exported under an older schema up to the live one, before
/// `jsonb_populate_recordset` reads it.
///
/// That function fills a key the row lacks with NULL, not with the column's
/// default, so a column added NOT NULL with a default fails the import of
/// every export written before it existed. v029 (tier entities) is the first:
/// `tier_placements.operator_id` became `entity_id` beside a new `entity_kind`,
/// and `tier_lists` gained `entity_kinds`. Every pre-v029 placement was an
/// operator, and every pre-v029 list offered operators only.
pub fn upgrade_legacy_row(table: &str, row: &mut serde_json::Value) {
    let Some(obj) = row.as_object_mut() else {
        return;
    };
    match table {
        "tier_placements" => {
            if !obj.contains_key("entity_id")
                && let Some(id) = obj.remove("operator_id")
            {
                obj.insert("entity_id".to_owned(), id);
            }
            obj.entry("entity_kind")
                .or_insert_with(|| serde_json::Value::from("operator"));
        }
        "tier_lists" => {
            obj.entry("entity_kinds")
                .or_insert_with(|| serde_json::json!(["operator"]));
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::{SEEDED_TABLES, SERIAL_COLUMNS, TABLES, upgrade_legacy_row};
    use serde_json::json;
    use std::collections::BTreeSet;

    /// `_migrations` is the migration runner's own bookkeeping; `--migrate`
    /// rebuilds it, and restoring it would mark migrations applied that the
    /// target schema never ran.
    const NOT_EXPORTED: &[&str] = &["_migrations"];

    #[test]
    fn every_migrated_table_is_exported() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/src/database/migrations");
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
            for rest in sql.split("create table ").skip(1) {
                let rest = rest.trim_start().trim_start_matches("if not exists ");
                let name: String = rest
                    .trim_start()
                    .trim_start_matches("public.")
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                created.insert(name);
            }
        }
        let listed: BTreeSet<String> = TABLES
            .iter()
            .chain(NOT_EXPORTED)
            .map(|t| (*t).to_owned())
            .collect();
        assert_eq!(TABLES.len(), TABLES.iter().collect::<BTreeSet<_>>().len());
        let missing: Vec<_> = created.difference(&listed).collect();
        assert!(missing.is_empty(), "add to db_export::TABLES: {missing:?}");
        let stale: Vec<_> = listed
            .difference(&created)
            .filter(|t| !NOT_EXPORTED.contains(&t.as_str()))
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
    }
}
