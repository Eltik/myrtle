use std::sync::Arc;

use chrono::Utc;
use reqwest::Client;
use tokio::sync::RwLock;

use super::types::{User, UserResponse};
use crate::core::{
    authentication::{
        auth_request,
        config::GlobalConfig,
        constants::{AuthSession, FetchError, Server},
    },
    local::types::GameData,
    user::{CharacterData, CharacterSkill},
};

const DEBUG_LOGS_DIR: &str = "debug_logs";

/// Maximum number of debug log directories to keep
const MAX_DEBUG_LOGS: usize = 10;

/// Cleans up old debug log directories, keeping only the most recent ones
fn cleanup_old_debug_logs() {
    use std::fs;
    use std::path::Path;

    let path = Path::new(DEBUG_LOGS_DIR);
    if !path.exists() {
        return;
    }

    let Ok(entries) = fs::read_dir(path) else {
        return;
    };

    // Collect all debug log directories
    let mut dirs: Vec<_> = entries
        .flatten()
        .filter(|e| {
            e.path().is_dir()
                && e.file_name()
                    .to_str()
                    .is_some_and(|name| name.starts_with("parse_error_"))
        })
        .collect();

    // Only cleanup if we exceed the limit
    if dirs.len() <= MAX_DEBUG_LOGS {
        return;
    }

    // Sort by name (which includes timestamp, so alphabetical = chronological)
    dirs.sort_by_key(|d| d.file_name());

    // Remove oldest directories (keep the last MAX_DEBUG_LOGS)
    let to_remove = dirs.len() - MAX_DEBUG_LOGS;
    for dir in dirs.into_iter().take(to_remove) {
        if let Err(e) = fs::remove_dir_all(dir.path()) {
            eprintln!("Failed to remove old debug log directory: {e}");
        }
    }
}

/// Saves debug information when JSON parsing fails
fn save_debug_logs(raw_json: &str, error: &serde_json::Error) {
    use std::fs;
    use std::path::Path;

    // Clean up old logs first to prevent unbounded growth
    cleanup_old_debug_logs();

    let col = error.column();
    let line = error.line();

    // Create timestamped folder name
    let timestamp = Utc::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let folder_name = format!("parse_error_{timestamp}");
    let folder_path = Path::new(DEBUG_LOGS_DIR).join(&folder_name);

    // Create directories
    if let Err(e) = fs::create_dir_all(&folder_path) {
        eprintln!("Failed to create debug folder: {e}");
        return;
    }

    // Save raw JSON
    let json_path = folder_path.join("sync_data.json");
    if let Err(e) = fs::write(&json_path, raw_json) {
        eprintln!("Failed to write JSON: {e}");
    }

    // Build error report
    let start = col.saturating_sub(200);
    let end = (col + 200).min(raw_json.len());
    let context = &raw_json[start..end];

    // Find the likely problematic field
    let before_error = &raw_json[col.saturating_sub(100)..col];
    let field_name = if let Some(last_quote) = before_error.rfind("\":") {
        let field_start = before_error[..last_quote].rfind('"').unwrap_or(0);
        Some(&before_error[field_start + 1..last_quote])
    } else {
        None
    };

    let error_report = format!(
        r#"Parse Error Report
==================

Timestamp: {}
Error: {}

Location:
  Line: {}
  Column: {}

Likely Problematic Field: {}

Context Around Error (col {}-{}):
{}

Raw Error:
{:?}
"#,
        timestamp,
        error,
        line,
        col,
        field_name.unwrap_or("unknown"),
        start,
        end,
        context,
        error
    );

    // Save error report
    let report_path = folder_path.join("error_report.txt");
    if let Err(e) = fs::write(&report_path, &error_report) {
        eprintln!("Failed to write error report: {e}");
    }

    // Also print to console
    eprintln!("\n{error_report}");
    eprintln!("Debug files saved to: {}\n", folder_path.display());
}

pub async fn get(
    client: &Client,
    config: &Arc<RwLock<GlobalConfig>>,
    session: &mut AuthSession,
    server: Server,
    game_data: &GameData,
) -> Result<Option<User>, FetchError> {
    let body = serde_json::json!({ "platform": 1 });

    let response = auth_request(
        client,
        config,
        "account/syncData",
        Some(body),
        session,
        server,
    )
    .await?;

    // Get raw JSON text first for debugging
    let raw_json = response.text().await.map_err(FetchError::RequestFailed)?;

    // Try to parse and save debug info on error
    let data: UserResponse = match serde_json::from_str(&raw_json) {
        Ok(data) => data,
        Err(e) => {
            save_debug_logs(&raw_json, &e);
            return Err(FetchError::ParseError(e.to_string()));
        }
    };

    Ok(data.user.map(|mut user| {
        format_user(&mut user, game_data);
        user
    }))
}

/// Enriches user data in-place with static game data
///
/// This function is O(n) where n is the number of characters,
/// but each lookup is O(1) thanks to HashMap-based GameData.
pub fn format_user(
    user: &mut User,
    game_data: &GameData, // Pre-loaded reference
) {
    // Enrich characters with static operator data
    for character in user.troop.chars.values_mut() {
        format_character(character, game_data);
    }

    format_inventory(user, game_data);
}

/// Enriches a single character with static data
fn format_character(character: &mut CharacterData, game_data: &GameData) {
    // Look up operator static data (O(1) HashMap lookup)
    if let Some(operator) = game_data.operators.get(&character.char_id) {
        // Calculate trust
        let trust = calculate_trust(character.favor_point, game_data);

        // Build static data JSON
        let mut static_data = serde_json::to_value(operator).unwrap_or_default();
        if let serde_json::Value::Object(ref mut map) = static_data {
            map.insert("trust".to_string(), serde_json::json!(trust));
        }

        character.r#static = Some(static_data);

        // Enrich skills
        for skill in &mut character.skills {
            format_skill(skill, character.main_skill_lvl, game_data);
        }
    }
}

/// Enriches a single skill with static data at the correct level
fn format_skill(skill: &mut CharacterSkill, main_skill_lvl: i32, game_data: &GameData) {
    if let Some(skill_data) = game_data.skills.get(&skill.skill_id) {
        // Calculate the correct level index
        // mainSkillLvl is 1-7, specializeLevel is 0-3
        // Level index = (mainSkillLvl - 1) + specializeLevel
        // But skill levels array is 0-indexed with 7 base levels + 3 mastery = 10 total
        let level_index = (main_skill_lvl - 1 + skill.specialize_level) as usize;

        // Get the specific level data if it exists
        let level_data = skill_data.levels.get(level_index);

        // Build skill static data
        let mut static_data = serde_json::json!({
            "skillId": skill_data.skill_id,
            "iconId": skill_data.icon_id,
            "hidden": skill_data.hidden,
            "image": skill_data.image,
        });

        // Add level-specific data if available
        if let Some(level) = level_data
            && let serde_json::Value::Object(ref mut map) = static_data
        {
            map.insert("name".to_string(), serde_json::json!(level.name));
            map.insert(
                "description".to_string(),
                serde_json::json!(level.description),
            );
            map.insert("duration".to_string(), serde_json::json!(level.duration));
            map.insert(
                "spData".to_string(),
                serde_json::to_value(&level.sp_data).unwrap_or_default(),
            );
        }

        skill.r#static = Some(static_data);
    }
}

/// Calculates trust level from favor points.
///
/// Uses a linear approximation. Trust ranges from 0-200.
/// TODO: Use favor_table lookup for accurate thresholds.
fn calculate_trust(favor_point: i32, _game_data: &GameData) -> i32 {
    (favor_point / 100).min(200)
}

fn format_inventory(user: &mut User, game_data: &GameData) {
    let raw_inventory = std::mem::take(&mut user.inventory);

    user.inventory = raw_inventory
        .into_iter()
        .filter_map(|(item_id, count)| {
            game_data.materials.items.get(&item_id).map(|material| {
                let mut enriched = serde_json::to_value(material).unwrap_or_default();
                if let serde_json::Value::Object(ref mut map) = enriched {
                    map.insert("amount".to_string(), serde_json::json!(count));
                    // Add resolved image path using asset mappings
                    let image_path = game_data
                        .asset_mappings
                        .get_item_icon_path(&material.icon_id);
                    map.insert("image".to_string(), serde_json::json!(image_path));
                }
                (item_id, enriched)
            })
        })
        .collect();
}
