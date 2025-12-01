use std::sync::Arc;

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
    let data: UserResponse = response.json().await.map_err(FetchError::RequestFailed)?;

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
        if let Some(level) = level_data {
            if let serde_json::Value::Object(ref mut map) = static_data {
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
        }

        skill.r#static = Some(static_data);
    }
}

/// Calculate trust level from favor points
/// TODO: Implement properly when favor_table is loaded
fn calculate_trust(favor_point: i32, _game_data: &GameData) -> i32 {
    // Simplified trust calculation
    // Real implementation would use favor_table to find the correct trust level
    // Trust ranges from 0-200, favor_point threshold varies

    // Placeholder: rough approximation
    // In reality, you'd binary search through favor_table frames
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
                }
                (item_id, enriched)
            })
        })
        .collect();
}
