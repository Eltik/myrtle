//! Auto-generated FlatBuffer decode dispatch
//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs

use serde_json::{Value, json};
use std::panic::{self, AssertUnwindSafe};

/// Check if data is likely a FlatBuffer
pub fn is_flatbuffer(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }
    let root_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
    if root_offset >= data.len() || root_offset < 4 {
        return false;
    }
    if root_offset + 4 > data.len() {
        return false;
    }
    let vtable_offset = i32::from_le_bytes([
        data[root_offset],
        data[root_offset + 1],
        data[root_offset + 2],
        data[root_offset + 3],
    ]);
    let vtable_pos = (root_offset as i32 - vtable_offset) as usize;
    if vtable_pos >= data.len() || vtable_pos < 4 {
        return false;
    }
    let vtable_size = u16::from_le_bytes([data[vtable_pos], data[vtable_pos + 1]]) as usize;
    (4..1000).contains(&vtable_size) && vtable_pos + vtable_size <= data.len()
}

/// Guess the root type from filename
fn guess_root_type(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();

    if lower.starts_with("level_") {
        "level_data"
    } else if lower.contains("enemy_database") {
        "enemy_database"
    } else if lower.contains("enemy_handbook") {
        "enemy_handbook_table"
    } else if lower.contains("character_table") || lower.contains("char_table") {
        "character_table"
    } else if lower.contains("char_master") {
        "char_master_table"
    } else if lower.contains("char_meta") {
        "char_meta_table"
    } else if lower.contains("char_patch") {
        "char_patch_table"
    } else if lower.contains("charword") {
        "charword_table"
    } else if lower.contains("skill_table") {
        "skill_table"
    } else if lower.contains("item_table") {
        "item_table"
    } else if lower.contains("gacha_table") {
        "gacha_table"
    } else if lower.contains("skin_table") {
        "skin_table"
    } else if lower.contains("handbook_info") {
        "handbook_info_table"
    } else if lower.contains("handbook_team") {
        "handbook_team_table"
    } else if lower.contains("uniequip_table") {
        "uniequip_table"
    } else if lower.contains("battle_equip") {
        "battle_equip_table"
    } else if lower.contains("stage_table") {
        "stage_table"
    } else if lower.contains("activity_table") {
        "activity_table"
    } else if lower.contains("audio_data") {
        "audio_data"
    } else if lower.contains("building_local") {
        "building_local_data"
    } else if lower.contains("building_data") {
        "building_data"
    } else if lower.contains("campaign_table") {
        "campaign_table"
    } else if lower.contains("chapter_table") {
        "chapter_table"
    } else if lower.contains("charm_table") {
        "charm_table"
    } else if lower.contains("checkin_table") {
        "checkin_table"
    } else if lower.contains("climb_tower") {
        "climb_tower_table"
    } else if lower.contains("clue_data") {
        "clue_data"
    } else if lower.contains("crisis_v2") {
        "crisis_v2_table"
    } else if lower.contains("crisis_table") {
        "crisis_table"
    } else if lower.contains("display_meta") {
        "display_meta_table"
    } else if lower.contains("favor_table") {
        "favor_table"
    } else if lower.contains("gamedata_const") {
        "gamedata_const"
    } else if lower.contains("hotupdate_meta") {
        "hotupdate_meta_table"
    } else if lower.contains("medal_table") {
        "medal_table"
    } else if lower.contains("meta_ui") {
        "meta_ui_table"
    } else if lower.contains("mission_table") {
        "mission_table"
    } else if lower.contains("open_server") {
        "open_server_table"
    } else if lower.contains("retro_table") {
        "retro_table"
    } else if lower.contains("roguelike") {
        "roguelike_topic_table"
    } else if lower.contains("sandbox_perm") {
        "sandbox_perm_table"
    } else if lower.contains("sandbox_table") {
        "sandbox_table"
    } else if lower.contains("shop_client") {
        "shop_client_table"
    } else if lower.contains("special_operator") {
        "special_operator_table"
    } else if lower.contains("story_review_meta") {
        "story_review_meta_table"
    } else if lower.contains("story_review") {
        "story_review_table"
    } else if lower.contains("story_table") {
        "story_table"
    } else if lower.contains("tip_table") {
        "tip_table"
    } else if lower.contains("zone_table") {
        "zone_table"
    } else if lower.contains("ep_breakbuff") {
        "ep_breakbuff_table"
    } else if lower.contains("buff_table") {
        "buff_table"
    } else if lower.contains("cooperate") {
        "cooperate_battle_table"
    } else if lower.contains("init_text") || lower.contains("main_text") {
        "language_data"
    } else if lower.contains("extra_battlelog") {
        "extra_battlelog_table"
    } else if lower.contains("replicate") {
        "replicate_table"
    } else if lower.contains("legion_mode") {
        "legion_mode_buff_table"
    } else if lower.contains("token_table") {
        "token_table"
    } else {
        "unknown"
    }
}

/// Check if a schema type has a Yostar variant
fn has_yostar_schema(schema_type: &str) -> bool {
    matches!(
        schema_type,
        "character_table" | "battle_equip_table" | "token_table" | "ep_breakbuff_table"
    )
}

/// Try decoding with Yostar-specific schemas
fn decode_flatbuffer_yostar(data: &[u8], schema_type: &str) -> Result<Value, String> {
    use crate::fb_json_macros::FlatBufferToJson;
    let data_clone = data.to_vec();
    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {
        let data = &data_clone;
        match schema_type {
            "character_table" => {
                use crate::generated_fbs_yostar::character_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "battle_equip_table" => {
                use crate::generated_fbs_yostar::battle_equip_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_unchecked(data)
                };
                Ok(root.to_json())
            }
            "token_table" => {
                use crate::generated_fbs_yostar::token_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "ep_breakbuff_table" => {
                use crate::generated_fbs_yostar::ep_breakbuff_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            _ => Err(format!("No Yostar schema for {}", schema_type)),
        }
    }));
    match decode_result {
        Ok(Ok(value)) => {
            if value.as_object().is_some_and(|o| o.is_empty()) {
                Err("Yostar decode returned empty".to_string())
            } else {
                Ok(value)
            }
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Yostar decode panic".to_string()),
    }
}

/// Decode FlatBuffer data to JSON using schema-based decoding
pub fn decode_flatbuffer(data: &[u8], filename: &str) -> Result<Value, String> {
    use crate::fb_json_macros::FlatBufferToJson;

    if !is_flatbuffer(data) {
        return Err("Data is not a valid FlatBuffer".to_string());
    }

    let schema_type = guess_root_type(filename);
    let data_clone = data.to_vec();

    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {
        let data = &data_clone;
        match schema_type {
            "character_table" => {
                use crate::generated_fbs::character_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "char_master_table" => {
                use crate::generated_fbs::char_master_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_master_data_bundle_unchecked(data)
                };
                Ok(root.to_json())
            }
            "char_meta_table" => {
                use crate::generated_fbs::char_meta_table_generated::*;
                let root = unsafe { root_as_clz_torappu_char_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "char_patch_table" => {
                use crate::generated_fbs::char_patch_table_generated::*;
                let root = unsafe { root_as_clz_torappu_char_patch_data_unchecked(data) };
                Ok(root.to_json())
            }
            "charword_table" => {
                use crate::generated_fbs::charword_table_generated::*;
                let root = unsafe { root_as_clz_torappu_char_word_table_unchecked(data) };
                Ok(root.to_json())
            }
            "skill_table" => {
                use crate::generated_fbs::skill_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_skill_data_bundle_unchecked(data)
                };
                Ok(root.to_json())
            }
            "enemy_database" => {
                use crate::generated_fbs::enemy_database_generated::*;
                let root = unsafe { root_as_clz_torappu_enemy_database_unchecked(data) };
                Ok(root.to_json())
            }
            "enemy_handbook_table" => {
                use crate::generated_fbs::enemy_handbook_table_generated::*;
                let root =
                    unsafe { root_as_clz_torappu_enemy_hand_book_data_group_unchecked(data) };
                Ok(root.to_json())
            }
            "item_table" => {
                use crate::generated_fbs::item_table_generated::*;
                let root = unsafe { root_as_clz_torappu_inventory_data_unchecked(data) };
                Ok(root.to_json())
            }
            "skin_table" => {
                use crate::generated_fbs::skin_table_generated::*;
                let root = unsafe { root_as_clz_torappu_skin_table_unchecked(data) };
                Ok(root.to_json())
            }
            "uniequip_table" => {
                use crate::generated_fbs::uniequip_table_generated::*;
                let root = unsafe { root_as_clz_torappu_uni_equip_table_unchecked(data) };
                Ok(root.to_json())
            }
            "battle_equip_table" => {
                use crate::generated_fbs::battle_equip_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_unchecked(data)
                };
                Ok(root.to_json())
            }
            "handbook_info_table" => {
                use crate::generated_fbs::handbook_info_table_generated::*;
                let root = unsafe { root_as_clz_torappu_handbook_info_table_unchecked(data) };
                Ok(root.to_json())
            }
            "handbook_team_table" => {
                use crate::generated_fbs::handbook_team_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_handbook_team_data_unchecked(
                        data,
                    )
                };
                Ok(root.to_json())
            }
            "gacha_table" => {
                use crate::generated_fbs::gacha_table_generated::*;
                let root = unsafe { root_as_clz_torappu_gacha_data_unchecked(data) };
                Ok(root.to_json())
            }
            "stage_table" => {
                use crate::generated_fbs::stage_table_generated::*;
                let root = unsafe { root_as_clz_torappu_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            "activity_table" => {
                use crate::generated_fbs::activity_table_generated::*;
                let root = unsafe { root_as_clz_torappu_activity_table_unchecked(data) };
                Ok(root.to_json())
            }
            "audio_data" => {
                use crate::generated_fbs::audio_data_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_audio_middleware_data_torappu_audio_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "building_data" => {
                use crate::generated_fbs::building_data_generated::*;
                let root = unsafe { root_as_clz_torappu_building_data_unchecked(data) };
                Ok(root.to_json())
            }
            "building_local_data" => {
                use crate::generated_fbs::building_local_data_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_building_data_building_local_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "campaign_table" => {
                use crate::generated_fbs::campaign_table_generated::*;
                let root = unsafe { root_as_clz_torappu_campaign_table_unchecked(data) };
                Ok(root.to_json())
            }
            "chapter_table" => {
                use crate::generated_fbs::chapter_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_chapter_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "charm_table" => {
                use crate::generated_fbs::charm_table_generated::*;
                let root = unsafe { root_as_clz_torappu_charm_data_unchecked(data) };
                Ok(root.to_json())
            }
            "checkin_table" => {
                use crate::generated_fbs::checkin_table_generated::*;
                let root = unsafe { root_as_clz_torappu_check_in_table_unchecked(data) };
                Ok(root.to_json())
            }
            "climb_tower_table" => {
                use crate::generated_fbs::climb_tower_table_generated::*;
                let root = unsafe { root_as_clz_torappu_climb_tower_table_unchecked(data) };
                Ok(root.to_json())
            }
            "clue_data" => {
                use crate::generated_fbs::clue_data_generated::*;
                let root = unsafe { root_as_clz_torappu_meeting_clue_data_unchecked(data) };
                Ok(root.to_json())
            }
            "crisis_table" => {
                use crate::generated_fbs::crisis_table_generated::*;
                let root = unsafe { root_as_clz_torappu_crisis_client_data_unchecked(data) };
                Ok(root.to_json())
            }
            "crisis_v2_table" => {
                use crate::generated_fbs::crisis_v2_table_generated::*;
                let root = unsafe { root_as_clz_torappu_crisis_v2_shared_data_unchecked(data) };
                Ok(root.to_json())
            }
            "display_meta_table" => {
                use crate::generated_fbs::display_meta_table_generated::*;
                let root = unsafe { root_as_clz_torappu_display_meta_data_unchecked(data) };
                Ok(root.to_json())
            }
            "favor_table" => {
                use crate::generated_fbs::favor_table_generated::*;
                let root = unsafe { root_as_clz_torappu_favor_table_unchecked(data) };
                Ok(root.to_json())
            }
            "gamedata_const" => {
                use crate::generated_fbs::gamedata_const_generated::*;
                let root = unsafe { root_as_clz_torappu_game_data_consts_unchecked(data) };
                Ok(root.to_json())
            }
            "hotupdate_meta_table" => {
                use crate::generated_fbs::hotupdate_meta_table_generated::*;
                let root = unsafe { root_as_clz_torappu_hot_update_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "medal_table" => {
                use crate::generated_fbs::medal_table_generated::*;
                let root = unsafe { root_as_clz_torappu_medal_data_unchecked(data) };
                Ok(root.to_json())
            }
            "meta_ui_table" => {
                use crate::generated_fbs::meta_ui_table_generated::*;
                let root = unsafe { root_as_clz_torappu_meta_uidisplay_table_unchecked(data) };
                Ok(root.to_json())
            }
            "mission_table" => {
                use crate::generated_fbs::mission_table_generated::*;
                let root = unsafe { root_as_clz_torappu_mission_table_unchecked(data) };
                Ok(root.to_json())
            }
            "open_server_table" => {
                use crate::generated_fbs::open_server_table_generated::*;
                let root = unsafe { root_as_clz_torappu_open_server_schedule_unchecked(data) };
                Ok(root.to_json())
            }
            "retro_table" => {
                use crate::generated_fbs::retro_table_generated::*;
                let root = unsafe { root_as_clz_torappu_retro_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            "roguelike_topic_table" => {
                use crate::generated_fbs::roguelike_topic_table_generated::*;
                let root = unsafe { root_as_clz_torappu_roguelike_topic_table_unchecked(data) };
                Ok(root.to_json())
            }
            "sandbox_perm_table" => {
                use crate::generated_fbs::sandbox_perm_table_generated::*;
                let root = unsafe { root_as_clz_torappu_sandbox_perm_table_unchecked(data) };
                Ok(root.to_json())
            }
            "sandbox_table" => {
                use crate::generated_fbs::sandbox_table_generated::*;
                let root = unsafe { root_as_clz_torappu_sandbox_table_unchecked(data) };
                Ok(root.to_json())
            }
            "shop_client_table" => {
                use crate::generated_fbs::shop_client_table_generated::*;
                let root = unsafe { root_as_clz_torappu_shop_client_data_unchecked(data) };
                Ok(root.to_json())
            }
            "special_operator_table" => {
                use crate::generated_fbs::special_operator_table_generated::*;
                let root = unsafe { root_as_clz_torappu_special_operator_table_unchecked(data) };
                Ok(root.to_json())
            }
            "story_review_meta_table" => {
                use crate::generated_fbs::story_review_meta_table_generated::*;
                let root = unsafe { root_as_clz_torappu_story_review_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "story_review_table" => {
                use crate::generated_fbs::story_review_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_story_review_group_client_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "story_table" => {
                use crate::generated_fbs::story_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_story_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "tip_table" => {
                use crate::generated_fbs::tip_table_generated::*;
                let root = unsafe { root_as_clz_torappu_tip_table_unchecked(data) };
                Ok(root.to_json())
            }
            "zone_table" => {
                use crate::generated_fbs::zone_table_generated::*;
                let root = unsafe { root_as_clz_torappu_zone_table_unchecked(data) };
                Ok(root.to_json())
            }
            "buff_table" => {
                use crate::generated_fbs::buff_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "cooperate_battle_table" => {
                use crate::generated_fbs::cooperate_battle_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_battle_cooperate_cooperate_mode_battle_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "language_data" => {
                use crate::generated_fbs::init_text_generated::*;
                let root = unsafe { root_as_clz_torappu_language_data_unchecked(data) };
                Ok(root.to_json())
            }
            "ep_breakbuff_table" => {
                use crate::generated_fbs::ep_breakbuff_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "extra_battlelog_table" => {
                use crate::generated_fbs::extra_battlelog_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_extra_battle_log_data_unchecked(
                        data,
                    )
                };
                Ok(root.to_json())
            }
            "replicate_table" => {
                use crate::generated_fbs::replicate_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_replicate_table_unchecked(data)
                };
                Ok(root.to_json())
            }
            "legion_mode_buff_table" => {
                use crate::generated_fbs::legion_mode_buff_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_battle_legion_legion_mode_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "token_table" => {
                use crate::generated_fbs::token_table_generated::*;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "level_data" => {
                use crate::generated_fbs::prts___levels_generated::*;
                let root = unsafe { root_as_clz_torappu_level_data_unchecked(data) };
                Ok(root.to_json())
            }
            _ => Err(format!("Unknown schema type: {}", schema_type)),
        }
    }));

    match decode_result {
        Ok(Ok(value)) => {
            if value.as_object().is_some_and(|o| o.is_empty()) {
                if has_yostar_schema(schema_type)
                    && let Ok(v) = decode_flatbuffer_yostar(data, schema_type)
                {
                    return Ok(v);
                }
                Err(format!(
                    "Schema mismatch for {} (empty result)",
                    schema_type
                ))
            } else {
                Ok(value)
            }
        }
        Ok(Err(e)) => {
            if has_yostar_schema(schema_type)
                && let Ok(v) = decode_flatbuffer_yostar(data, schema_type)
            {
                return Ok(v);
            }
            if schema_type == "unknown" {
                let strings = extract_strings(data);
                if !strings.is_empty() {
                    return Ok(json!({ "type": "unknown", "strings": strings }));
                }
            }
            Err(format!("Decode failed for {}: {}", schema_type, e))
        }
        Err(_) => {
            if has_yostar_schema(schema_type)
                && let Ok(v) = decode_flatbuffer_yostar(data, schema_type)
            {
                return Ok(v);
            }
            Err(format!("Decode panic for {}", schema_type))
        }
    }
}

/// Extract strings from FlatBuffer (fallback for unknown types)
pub fn extract_strings(data: &[u8]) -> Vec<String> {
    let mut strings = Vec::new();
    let mut i = 0;
    while i + 4 < data.len() {
        let len = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]) as usize;
        if len > 0
            && len < 1000
            && i + 4 + len <= data.len()
            && let Ok(s) = std::str::from_utf8(&data[i + 4..i + 4 + len])
            && s.len() >= 2
            && s.chars()
                .all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace() || !c.is_ascii())
        {
            strings.push(s.to_string());
        }
        i += 1;
    }
    strings
}
