//! Shared table metadata for the `export-database` and `import-database` binaries.
//!
//! TABLES is the topological order dictated by FK constraints: parents first,
//! children second, audit_log last. Export writes in this order; import replays
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
    "gacha_records",
    "tier_lists",
    "tiers",
    "tier_placements",
    "tier_list_versions",
    "tier_list_permissions",
    "operator_notes",
    "operator_notes_audit_log",
    "audit_log",
];

/// (table, serial column) — sequences that must be reset after import so
/// future inserts don't collide with restored ids.
pub const SERIAL_COLUMNS: &[(&str, &str)] = &[
    ("gacha_records", "id"),
    ("operator_notes_audit_log", "id"),
    ("audit_log", "id"),
];

pub const MANIFEST_FILE: &str = "manifest.json";
pub const FORMAT_VERSION: u32 = 1;
