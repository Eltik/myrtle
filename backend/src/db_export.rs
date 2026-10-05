//! Shared table metadata for the `export-database` and `import-database` binaries.
//!
//! TABLES is the topological order dictated by FK constraints: parents first,
//! children second, `audit_log` last. Export writes in this order; import replays
//! it unchanged.

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
    "audit_log",
];

/// (table, serial column): sequences reset after import so new inserts don't
/// collide with restored ids.
pub const SERIAL_COLUMNS: &[(&str, &str)] = &[
    ("gacha_records", "id"),
    ("tier_list_flairs", "id"),
    ("tier_list_view_events", "id"),
    ("operator_notes_audit_log", "id"),
    ("leaderboard_snapshots", "id"),
    ("audit_log", "id"),
];

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
    use super::upgrade_legacy_row;
    use serde_json::json;

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
