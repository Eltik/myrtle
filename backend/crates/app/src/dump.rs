mod export;
mod import;
mod legacy;

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};

use myrtle_db::DbError;
use myrtle_db::tools::ForeignKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub use export::{ExportReport, Selection, export};
pub use import::{DEFAULT_BATCH_SIZE, ImportOptions, ImportReport, Mismatch, import};
pub use legacy::{stream_array, translate};

pub const MANIFEST_FILE: &str = "manifest.json";
pub const FORMAT_VERSION: u32 = 1;

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
    "user_squads",
    "user_game_squads",
    "user_lookback_snapshots",
    "siren_albums",
    "siren_songs",
    "music_likes",
    "music_history",
    "music_playlists",
    "music_playlist_tracks",
    "music_track_stats",
    "music_stat_events",
    "locales",
    "ui_message_keys",
    "ui_messages",
    "ui_documents",
    "ui_message_audit_log",
    "translation_permissions",
    "gamedata_overrides",
    "gamedata_sightings",
    "release_overrides",
    "oauth_apps",
    "oauth_grants",
    "oauth_tokens",
    "operator_ownership_stats",
    "operator_skill_choice_stats",
    "operator_mastery_stats",
    "operator_module_choice_stats",
    "operator_module_level_stats",
    "medal_ownership_stats",
    "content_audit_log",
    "audit_log",
];

pub const NOT_EXPORTED: &[(&str, &str)] = &[
    (
        "_migrations",
        "the legacy runner's bookkeeping, adopted by the baseline",
    ),
    (
        "_sqlx_migrations",
        "migrate rebuilds it; restoring it would claim migrations the target never ran",
    ),
    (
        "oauth_codes",
        "codes expire five minutes after they are minted",
    ),
    ("music_stat_dedupe", "an unlogged day of dedupe slots"),
    (
        "jobs",
        "the live queue: restored leases and queued runs belong to the old deployment",
    ),
    ("job_schedules", "rebuilt from config at boot"),
    (
        "service_account_sessions",
        "a restored session signs the source box's service account out",
    ),
    (
        "data_generations",
        "describes the source box's game data, not the target's",
    ),
];

pub const SEEDED_TABLES: &[&str] = &["servers", "locales"];

pub const SERIAL_COLUMNS: &[(&str, &str)] = &[
    ("gacha_records", "id"),
    ("tier_list_flairs", "id"),
    ("tier_list_view_events", "id"),
    ("operator_notes_audit_log", "id"),
    ("leaderboard_snapshots", "id"),
    ("ui_message_audit_log", "id"),
    ("content_audit_log", "id"),
    ("audit_log", "id"),
    ("music_history", "id"),
    ("music_stat_events", "id"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub format_version: u32,
    pub exported_at: String,
    pub database_version: String,
    pub tables: Vec<TableEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEntry {
    pub name: String,
    pub rows: u64,
    pub file: String,
}

impl TableEntry {
    #[must_use]
    pub fn new(name: &str, rows: u64) -> Self {
        Self {
            name: name.to_owned(),
            rows,
            file: file_name(name),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DumpError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error("{table}")]
    Table { table: String, source: DbError },
    #[error("{}", path.display())]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{}", path.display())]
    Json {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("{0}")]
    Invalid(String),
}

impl DumpError {
    fn io(path: &Path) -> impl FnOnce(std::io::Error) -> Self + '_ {
        move |source| Self::Io {
            path: path.to_owned(),
            source,
        }
    }

    fn json(path: &Path) -> impl FnOnce(serde_json::Error) -> Self + '_ {
        move |source| Self::Json {
            path: path.to_owned(),
            source,
        }
    }
}

#[must_use]
pub fn file_name(table: &str) -> String {
    format!("{table}.jsonl")
}

pub fn closure<'a>(
    start: impl IntoIterator<Item = &'a str>,
    edges: &'a [ForeignKey],
    towards_parent: bool,
) -> BTreeSet<&'a str> {
    let mut seen: BTreeSet<&str> = start.into_iter().collect();
    let mut queue: Vec<&str> = seen.iter().copied().collect();
    while let Some(t) = queue.pop() {
        for edge in edges {
            let (from, to) = if towards_parent {
                (&edge.child, &edge.parent)
            } else {
                (&edge.parent, &edge.child)
            };
            if from == t && seen.insert(to.as_str()) {
                queue.push(to);
            }
        }
    }
    seen
}

pub fn upgrade_legacy_row(table: &str, row: &mut Value) {
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
                .or_insert_with(|| Value::from("operator"));
        }
        "tier_lists" => {
            obj.entry("entity_kinds")
                .or_insert_with(|| serde_json::json!(["operator"]));
        }
        _ => {}
    }
}

pub fn read_manifest(dir: &Path) -> Result<Manifest, DumpError> {
    let path = dir.join(MANIFEST_FILE);
    let file = File::open(&path).map_err(DumpError::io(&path))?;
    serde_json::from_reader(BufReader::new(file)).map_err(DumpError::json(&path))
}

pub fn write_manifest(dir: &Path, manifest: &Manifest) -> Result<(), DumpError> {
    let path = dir.join(MANIFEST_FILE);
    let value = serde_json::to_value(manifest).map_err(DumpError::json(&path))?;
    let file = File::create(&path).map_err(DumpError::io(&path))?;
    let mut w = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut w, &value).map_err(DumpError::json(&path))?;
    w.flush().map_err(DumpError::io(&path))
}

pub fn check_manifest(manifest: &Manifest) -> Result<(), DumpError> {
    if manifest.format_version != FORMAT_VERSION {
        return Err(DumpError::Invalid(format!(
            "manifest format_version {} is not supported (expected {FORMAT_VERSION})",
            manifest.format_version
        )));
    }
    let mut rest = TABLES.iter();
    for entry in &manifest.tables {
        if !TABLES.contains(&entry.name.as_str()) {
            return Err(DumpError::Invalid(format!(
                "manifest lists {}, which this build does not know; was the export made by a newer build?",
                entry.name
            )));
        }
        if !rest.any(|t| *t == entry.name) {
            return Err(DumpError::Invalid(format!(
                "manifest lists {} out of order or twice; expected a subsequence of {TABLES:?}",
                entry.name
            )));
        }
        if entry.file != file_name(&entry.name) {
            return Err(DumpError::Invalid(format!(
                "unexpected filename in manifest for {}: {}",
                entry.name, entry.file
            )));
        }
    }
    Ok(())
}
