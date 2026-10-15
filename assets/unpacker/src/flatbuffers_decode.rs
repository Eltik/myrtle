//! Auto-generated `FlatBuffer` decode dispatch
//! DO NOT EDIT - regenerate with: cargo run --bin generate-fbs

use crate::schema::opts::verifier_opts;
use serde_json::{Value, json};
use std::panic::{self, AssertUnwindSafe};

/// Check if data is likely a `FlatBuffer`
#[must_use]
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

/// Guess the root type (the store stem) from a gamedata filename.
///
/// Generated from the schema store: the longest stem contained in the
/// filename wins, then a short alias table, then the `level_` prefix.
fn guess_root_type(filename: &str) -> &'static str {
    const STEMS: &[&str] = &[
        "story_review_meta_table",
        "cooperate_battle_table",
        "legion_mode_buff_table",
        "special_operator_table",
        "extra_battlelog_table",
        "roguelike_topic_table",
        "enemy_handbook_table",
        "hotupdate_meta_table",
        "building_local_data",
        "handbook_info_table",
        "handbook_team_table",
        "battle_equip_table",
        "display_meta_table",
        "ep_breakbuff_table",
        "level_script_table",
        "sandbox_perm_table",
        "story_review_table",
        "char_master_table",
        "climb_tower_table",
        "open_server_table",
        "resource_manifest",
        "shop_client_table",
        "bake_muzzle_data",
        "char_patch_table",
        "char_meta_table",
        "character_table",
        "crisis_v2_table",
        "replicate_table",
        "activity_table",
        "campaign_table",
        "charword_table",
        "enemy_database",
        "gamedata_const",
        "uniequip_table",
        "arkvent_table",
        "building_data",
        "chapter_table",
        "checkin_table",
        "meta_ui_table",
        "mission_table",
        "prts___levels",
        "sandbox_table",
        "crisis_table",
        "charm_table",
        "favor_table",
        "gacha_table",
        "medal_table",
        "retro_table",
        "skill_table",
        "stage_table",
        "story_table",
        "token_table",
        "audio_data",
        "buff_table",
        "item_table",
        "skin_table",
        "zone_table",
        "clue_data",
        "init_text",
        "main_text",
        "tip_table",
    ];
    const ALIASES: &[(&str, &str)] = &[
        ("char_table", "character_table"),
        ("language_data", "init_text"),
    ];
    let lower = filename.to_lowercase();
    if let Some(stem) = STEMS.iter().find(|s| lower.contains(*s)) {
        return stem;
    }
    if let Some((_, stem)) = ALIASES.iter().find(|(n, _)| lower.contains(n)) {
        return stem;
    }
    // Level files carry no table name; this must stay the LAST rule
    // (`level_script_table` is matched above as a stem).
    if lower.starts_with("level_") {
        return "prts___levels";
    }
    "unknown"
}

/// Check if a schema type has a Yostar variant
fn has_yostar_schema(schema_type: &str) -> bool {
    matches!(
        schema_type,
        "activity_table"
            | "buff_table"
            | "building_data"
            | "display_meta_table"
            | "item_table"
            | "retro_table"
            | "roguelike_topic_table"
            | "stage_table"
    )
}

/// Which schema a buffer actually verifies against.
#[derive(Clone, Copy, PartialEq, Eq)]
enum SchemaChoice {
    /// The CN schema verified. Decode exactly as before.
    Cn,
    /// The CN schema did not verify but the Yostar (EN/Global) one did.
    Yostar,
    /// Neither verified — fall back to the legacy decode-then-inspect path.
    Neither,
}

/// The Yostar half of a schema-verification verdict.
///
/// A bare `Option<Option<String>>` collapses "no Yostar variant" and "Yostar
/// variant not run" into one `None`, which `clippy::option_option` flags —
/// and which reads ambiguously at every call site anyway.
pub enum YostarVerdict {
    /// The table has no Yostar variant, or the caller short-circuited before
    /// running it (`full = false` and the CN schema already verified).
    NotRun,
    /// The Yostar schema verified.
    Verified,
    /// The Yostar schema did not verify; the first line of the error.
    Failed(String),
}

impl From<Option<String>> for YostarVerdict {
    fn from(err: Option<String>) -> Self {
        match err {
            None => Self::Verified,
            Some(e) => Self::Failed(e),
        }
    }
}

/// One table's verification verdict, for the `unpacker verify` report.
pub struct TableVerdict {
    /// The schema type `guess_root_type` resolved the filename to.
    pub table: &'static str,
    /// `None` when the CN schema verified, else the first line of the error.
    pub cn: Option<String>,
    /// The Yostar schema's verdict, if it was run.
    pub yostar: YostarVerdict,
    /// What `decode_flatbuffer` will actually do with this buffer:
    /// `"CN"` (CN schema verified), `"Yostar"` (routed to the Yostar schema),
    /// or `"none"` (nothing verified — the table is skipped, no file written).
    pub chosen: &'static str,
}

/// `InvalidFlatbuffer` Displays as a multi-line trace; one line per table is
/// what every caller here wants.
fn first_line(e: &::flatbuffers::InvalidFlatbuffer) -> String {
    e.to_string().lines().next().unwrap_or_default().to_string()
}

/// Verify `data` against the schemas available for `schema_type`.
///
/// Returns `(cn_err, yostar_verdict)`, where a `None` `cn_err` means the CN
/// schema verified. `full = false` short-circuits: once the CN schema
/// verifies the routing decision is already made, so the Yostar verifier is
/// not run and `yostar_verdict` comes back `NotRun`. `full = true` (the
/// `verify` subcommand) always runs both.
///
/// The whole function returns `None` if a verifier panics — it is not supposed
/// to, but neither was the decoder, and this is the one place that can still
/// contain it.
fn verify_schemas(
    data: &[u8],
    schema_type: &str,
    full: bool,
) -> Option<(Option<String>, YostarVerdict)> {
    let opts = verifier_opts();
    panic::catch_unwind(AssertUnwindSafe(|| match schema_type {
        "activity_table" => {
            let cn_err = {
                use crate::generated_fbs::activity_table_generated::root_as_clz_torappu_activity_table_with_opts;
                root_as_clz_torappu_activity_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::activity_table_generated::root_as_clz_torappu_activity_table_with_opts;
                root_as_clz_torappu_activity_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "arkvent_table" => {
            let cn_err = {
                use crate::generated_fbs::arkvent_table_generated::root_as_clz_torappu_ark_odc_table_with_opts;
                root_as_clz_torappu_ark_odc_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "audio_data" => {
            let cn_err = {
                use crate::generated_fbs::audio_data_generated::root_as_clz_torappu_audio_middleware_data_torappu_audio_data_with_opts;
                root_as_clz_torappu_audio_middleware_data_torappu_audio_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "bake_muzzle_data" => {
            let cn_err = {
                use crate::generated_fbs::bake_muzzle_data_generated::root_as_clz_torappu_battle_baked_spine_data_with_opts;
                root_as_clz_torappu_battle_baked_spine_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "battle_equip_table" => {
            let cn_err = {
                use crate::generated_fbs::battle_equip_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "buff_table" => {
            let cn_err = {
                use crate::generated_fbs::buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "building_data" => {
            let cn_err = {
                use crate::generated_fbs::building_data_generated::root_as_clz_torappu_building_data_with_opts;
                root_as_clz_torappu_building_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::building_data_generated::root_as_clz_torappu_building_data_with_opts;
                root_as_clz_torappu_building_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "building_local_data" => {
            let cn_err = {
                use crate::generated_fbs::building_local_data_generated::root_as_clz_torappu_building_data_building_local_data_with_opts;
                root_as_clz_torappu_building_data_building_local_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "campaign_table" => {
            let cn_err = {
                use crate::generated_fbs::campaign_table_generated::root_as_clz_torappu_campaign_table_with_opts;
                root_as_clz_torappu_campaign_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "chapter_table" => {
            let cn_err = {
                use crate::generated_fbs::chapter_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_chapter_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_chapter_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "char_master_table" => {
            let cn_err = {
                use crate::generated_fbs::char_master_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_master_data_bundle_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_master_data_bundle_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "char_meta_table" => {
            let cn_err = {
                use crate::generated_fbs::char_meta_table_generated::root_as_clz_torappu_char_meta_table_with_opts;
                root_as_clz_torappu_char_meta_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "char_patch_table" => {
            let cn_err = {
                use crate::generated_fbs::char_patch_table_generated::root_as_clz_torappu_char_patch_data_with_opts;
                root_as_clz_torappu_char_patch_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "character_table" => {
            let cn_err = {
                use crate::generated_fbs::character_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "charm_table" => {
            let cn_err = {
                use crate::generated_fbs::charm_table_generated::root_as_clz_torappu_charm_data_with_opts;
                root_as_clz_torappu_charm_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "charword_table" => {
            let cn_err = {
                use crate::generated_fbs::charword_table_generated::root_as_clz_torappu_char_word_table_with_opts;
                root_as_clz_torappu_char_word_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "checkin_table" => {
            let cn_err = {
                use crate::generated_fbs::checkin_table_generated::root_as_clz_torappu_check_in_table_with_opts;
                root_as_clz_torappu_check_in_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "climb_tower_table" => {
            let cn_err = {
                use crate::generated_fbs::climb_tower_table_generated::root_as_clz_torappu_climb_tower_table_with_opts;
                root_as_clz_torappu_climb_tower_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "clue_data" => {
            let cn_err = {
                use crate::generated_fbs::clue_data_generated::root_as_clz_torappu_meeting_clue_data_with_opts;
                root_as_clz_torappu_meeting_clue_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "cooperate_battle_table" => {
            let cn_err = {
                use crate::generated_fbs::cooperate_battle_table_generated::root_as_clz_torappu_battle_cooperate_cooperate_mode_battle_data_with_opts;
                root_as_clz_torappu_battle_cooperate_cooperate_mode_battle_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "crisis_table" => {
            let cn_err = {
                use crate::generated_fbs::crisis_table_generated::root_as_clz_torappu_crisis_client_data_with_opts;
                root_as_clz_torappu_crisis_client_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "crisis_v2_table" => {
            let cn_err = {
                use crate::generated_fbs::crisis_v2_table_generated::root_as_clz_torappu_crisis_v2_shared_data_with_opts;
                root_as_clz_torappu_crisis_v2_shared_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "display_meta_table" => {
            let cn_err = {
                use crate::generated_fbs::display_meta_table_generated::root_as_clz_torappu_display_meta_data_with_opts;
                root_as_clz_torappu_display_meta_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::display_meta_table_generated::root_as_clz_torappu_display_meta_data_with_opts;
                root_as_clz_torappu_display_meta_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "enemy_database" => {
            let cn_err = {
                use crate::generated_fbs::enemy_database_generated::root_as_clz_torappu_enemy_database_with_opts;
                root_as_clz_torappu_enemy_database_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "enemy_handbook_table" => {
            let cn_err = {
                use crate::generated_fbs::enemy_handbook_table_generated::root_as_clz_torappu_enemy_hand_book_data_group_with_opts;
                root_as_clz_torappu_enemy_hand_book_data_group_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "ep_breakbuff_table" => {
            let cn_err = {
                use crate::generated_fbs::ep_breakbuff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "extra_battlelog_table" => {
            let cn_err = {
                use crate::generated_fbs::extra_battlelog_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_extra_battle_log_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_extra_battle_log_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "favor_table" => {
            let cn_err = {
                use crate::generated_fbs::favor_table_generated::root_as_clz_torappu_favor_table_with_opts;
                root_as_clz_torappu_favor_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "gacha_table" => {
            let cn_err = {
                use crate::generated_fbs::gacha_table_generated::root_as_clz_torappu_gacha_data_with_opts;
                root_as_clz_torappu_gacha_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "gamedata_const" => {
            let cn_err = {
                use crate::generated_fbs::gamedata_const_generated::root_as_clz_torappu_game_data_consts_with_opts;
                root_as_clz_torappu_game_data_consts_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "handbook_info_table" => {
            let cn_err = {
                use crate::generated_fbs::handbook_info_table_generated::root_as_clz_torappu_handbook_info_table_with_opts;
                root_as_clz_torappu_handbook_info_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "handbook_team_table" => {
            let cn_err = {
                use crate::generated_fbs::handbook_team_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_handbook_team_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_handbook_team_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "hotupdate_meta_table" => {
            let cn_err = {
                use crate::generated_fbs::hotupdate_meta_table_generated::root_as_clz_torappu_hot_update_meta_table_with_opts;
                root_as_clz_torappu_hot_update_meta_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "init_text" => {
            let cn_err = {
                use crate::generated_fbs::init_text_generated::root_as_clz_torappu_language_data_with_opts;
                root_as_clz_torappu_language_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "item_table" => {
            let cn_err = {
                use crate::generated_fbs::item_table_generated::root_as_clz_torappu_inventory_data_with_opts;
                root_as_clz_torappu_inventory_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::item_table_generated::root_as_clz_torappu_inventory_data_with_opts;
                root_as_clz_torappu_inventory_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "legion_mode_buff_table" => {
            let cn_err = {
                use crate::generated_fbs::legion_mode_buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_battle_legion_legion_mode_buff_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_battle_legion_legion_mode_buff_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "level_script_table" => {
            let cn_err = {
                use crate::generated_fbs::level_script_table_generated::root_as_clz_torappu_battle_level_script_data_map_with_opts;
                root_as_clz_torappu_battle_level_script_data_map_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "main_text" => {
            let cn_err = {
                use crate::generated_fbs::main_text_generated::root_as_clz_torappu_language_data_with_opts;
                root_as_clz_torappu_language_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "medal_table" => {
            let cn_err = {
                use crate::generated_fbs::medal_table_generated::root_as_clz_torappu_medal_data_with_opts;
                root_as_clz_torappu_medal_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "meta_ui_table" => {
            let cn_err = {
                use crate::generated_fbs::meta_ui_table_generated::root_as_clz_torappu_meta_uidisplay_table_with_opts;
                root_as_clz_torappu_meta_uidisplay_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "mission_table" => {
            let cn_err = {
                use crate::generated_fbs::mission_table_generated::root_as_clz_torappu_mission_table_with_opts;
                root_as_clz_torappu_mission_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "open_server_table" => {
            let cn_err = {
                use crate::generated_fbs::open_server_table_generated::root_as_clz_torappu_open_server_schedule_with_opts;
                root_as_clz_torappu_open_server_schedule_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "prts___levels" => {
            let cn_err = {
                use crate::generated_fbs::prts___levels_generated::root_as_clz_torappu_level_data_with_opts;
                root_as_clz_torappu_level_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "replicate_table" => {
            let cn_err = {
                use crate::generated_fbs::replicate_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_replicate_table_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_replicate_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "resource_manifest" => {
            let cn_err = {
                use crate::generated_fbs::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_with_opts;
                root_as_clz_torappu_resource_resource_manifest_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "retro_table" => {
            let cn_err = {
                use crate::generated_fbs::retro_table_generated::root_as_clz_torappu_retro_stage_table_with_opts;
                root_as_clz_torappu_retro_stage_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::retro_table_generated::root_as_clz_torappu_retro_stage_table_with_opts;
                root_as_clz_torappu_retro_stage_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "roguelike_topic_table" => {
            let cn_err = {
                use crate::generated_fbs::roguelike_topic_table_generated::root_as_clz_torappu_roguelike_topic_table_with_opts;
                root_as_clz_torappu_roguelike_topic_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::roguelike_topic_table_generated::root_as_clz_torappu_roguelike_topic_table_with_opts;
                root_as_clz_torappu_roguelike_topic_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "sandbox_perm_table" => {
            let cn_err = {
                use crate::generated_fbs::sandbox_perm_table_generated::root_as_clz_torappu_sandbox_perm_table_with_opts;
                root_as_clz_torappu_sandbox_perm_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "sandbox_table" => {
            let cn_err = {
                use crate::generated_fbs::sandbox_table_generated::root_as_clz_torappu_sandbox_table_with_opts;
                root_as_clz_torappu_sandbox_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "shop_client_table" => {
            let cn_err = {
                use crate::generated_fbs::shop_client_table_generated::root_as_clz_torappu_shop_client_data_with_opts;
                root_as_clz_torappu_shop_client_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "skill_table" => {
            let cn_err = {
                use crate::generated_fbs::skill_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_skill_data_bundle_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_skill_data_bundle_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "skin_table" => {
            let cn_err = {
                use crate::generated_fbs::skin_table_generated::root_as_clz_torappu_skin_table_with_opts;
                root_as_clz_torappu_skin_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "special_operator_table" => {
            let cn_err = {
                use crate::generated_fbs::special_operator_table_generated::root_as_clz_torappu_special_operator_table_with_opts;
                root_as_clz_torappu_special_operator_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "stage_table" => {
            let cn_err = {
                use crate::generated_fbs::stage_table_generated::root_as_clz_torappu_stage_table_with_opts;
                root_as_clz_torappu_stage_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            if cn_err.is_none() && !full {
                // The CN schema verified and the caller only wants
                // the routing decision: skip the second verify.
                return (None, YostarVerdict::NotRun);
            }
            let yostar_err = {
                use crate::generated_fbs_yostar::stage_table_generated::root_as_clz_torappu_stage_table_with_opts;
                root_as_clz_torappu_stage_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, yostar_err.into())
        }
        "story_review_meta_table" => {
            let cn_err = {
                use crate::generated_fbs::story_review_meta_table_generated::root_as_clz_torappu_story_review_meta_table_with_opts;
                root_as_clz_torappu_story_review_meta_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "story_review_table" => {
            let cn_err = {
                use crate::generated_fbs::story_review_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_story_review_group_client_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_story_review_group_client_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "story_table" => {
            let cn_err = {
                use crate::generated_fbs::story_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_story_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_story_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "tip_table" => {
            let cn_err = {
                use crate::generated_fbs::tip_table_generated::root_as_clz_torappu_tip_table_with_opts;
                root_as_clz_torappu_tip_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "token_table" => {
            let cn_err = {
                use crate::generated_fbs::token_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_with_opts;
                root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "uniequip_table" => {
            let cn_err = {
                use crate::generated_fbs::uniequip_table_generated::root_as_clz_torappu_uni_equip_table_with_opts;
                root_as_clz_torappu_uni_equip_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        "zone_table" => {
            let cn_err = {
                use crate::generated_fbs::zone_table_generated::root_as_clz_torappu_zone_table_with_opts;
                root_as_clz_torappu_zone_table_with_opts(&opts, data).err().map(|e| first_line(&e))
            };
            (cn_err, YostarVerdict::NotRun)
        }
        _ => (None, YostarVerdict::NotRun),
    }))
    .ok()
}

/// Pick CN or Yostar for `schema_type` by VERIFYING the buffer, before any
/// decode runs.
///
/// WHY: only a handful of tables have a Yostar (EN/Global) schema variant, and
/// the CN→Yostar fallback used to fire only when the CN decode came out EMPTY.
/// That catches a mismatch that nulls everything out. It does not catch a
/// mismatch that produces plausible-looking garbage. Measured on EN
/// 26-08-28-10-20-08_ea3678: `activity_table` under the CN schema writes
/// 5,211,117,589 bytes of JSON where the Yostar schema writes 13,143,425, and
/// CN's own `activity_table` is 15,867,159. Not empty, so nothing detected it,
/// and it takes peak RSS for one EN extraction from 431 MB to 8.56 GB — which
/// is the 10 GB OOM kill the unpacker took on the VPS on 2026-08-04.
///
/// Verification is the only signal that fires BEFORE the garbage is
/// materialised. Every emptiness heuristic runs on a fully built
/// `serde_json::Value`, i.e. after the memory has already been spent, whereas
/// `root_as_*_with_opts` only walks offsets and vtables: it allocates no
/// `Value` and serialises nothing.
///
/// A verifier panic (it is not supposed to, but a decode panic was not supposed
/// to happen either) is treated as `Neither`, which is the pre-existing path.
///
/// Silent on purpose: several callers ask the same question about the same
/// buffer (the pre-decode skip check, the decode itself, `verify`), so the
/// routing note is printed once by `decode_flatbuffer`, not here.
fn select_schema_by_verification(data: &[u8], schema_type: &str) -> SchemaChoice {
    match verify_schemas(data, schema_type, false) {
        None => SchemaChoice::Neither,
        Some((None, _)) => SchemaChoice::Cn,
        Some((Some(_), YostarVerdict::Verified)) => SchemaChoice::Yostar,
        Some((Some(_), _)) => SchemaChoice::Neither,
    }
}

/// The verification verdict for one gamedata buffer, for `unpacker verify`.
///
/// `chosen` comes from `select_schema_by_verification` itself, so the report
/// cannot drift from what `extract` does; the error strings come from the same
/// emitted match arms, run once more with `full = true` so the Yostar column is
/// filled in even when the CN schema verified.
#[must_use]
pub fn verify_table(data: &[u8], filename: &str) -> TableVerdict {
    let table = guess_root_type(filename);
    let (cn, yostar) = verify_schemas(data, table, true)
        .unwrap_or_else(|| (Some("verifier panicked".to_string()), YostarVerdict::NotRun));
    let routed_yostar = has_yostar_schema(table)
        && select_schema_by_verification(data, table) == SchemaChoice::Yostar;
    let chosen = if routed_yostar {
        "Yostar"
    } else if cn.is_none() {
        "CN"
    } else {
        "none"
    };
    TableVerdict {
        table,
        cn,
        yostar,
        chosen,
    }
}

/// `Some(table)` when NO schema verifies this buffer, i.e. the caller must SKIP
/// it: no decode, no file written. `None` when it will decode (CN or Yostar),
/// and also when `guess_root_type` has no schema for the filename at all —
/// there is nothing to verify against there, and that path only ever produces
/// the harmless `extract_strings` listing.
///
/// Callers use this to skip BEFORE `export_text_asset` runs, because the
/// fall-through inside it would otherwise write the raw payload as `.bytes`
/// over a perfectly good `.json` from the previous extraction.
#[must_use]
pub fn unverified_table(data: &[u8], filename: &str) -> Option<&'static str> {
    let schema_type = guess_root_type(filename);
    if schema_type == "unknown" {
        return None;
    }
    match select_schema_by_verification(data, schema_type) {
        SchemaChoice::Cn | SchemaChoice::Yostar => None,
        SchemaChoice::Neither => Some(schema_type),
    }
}

/// Try decoding with Yostar-specific schemas
fn decode_flatbuffer_yostar(data: &[u8], schema_type: &str) -> Result<Value, String> {
    use crate::fb_json_macros::FlatBufferToJson;
    let data_clone = data.to_vec();
    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {
        let data = &data_clone;
        match schema_type {
            "activity_table" => {
                use crate::generated_fbs_yostar::activity_table_generated::root_as_clz_torappu_activity_table_unchecked;
                let root = unsafe { root_as_clz_torappu_activity_table_unchecked(data) };
                Ok(root.to_json())
            }
            "buff_table" => {
                use crate::generated_fbs_yostar::buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "building_data" => {
                use crate::generated_fbs_yostar::building_data_generated::root_as_clz_torappu_building_data_unchecked;
                let root = unsafe { root_as_clz_torappu_building_data_unchecked(data) };
                Ok(root.to_json())
            }
            "display_meta_table" => {
                use crate::generated_fbs_yostar::display_meta_table_generated::root_as_clz_torappu_display_meta_data_unchecked;
                let root = unsafe { root_as_clz_torappu_display_meta_data_unchecked(data) };
                Ok(root.to_json())
            }
            "item_table" => {
                use crate::generated_fbs_yostar::item_table_generated::root_as_clz_torappu_inventory_data_unchecked;
                let root = unsafe { root_as_clz_torappu_inventory_data_unchecked(data) };
                Ok(root.to_json())
            }
            "retro_table" => {
                use crate::generated_fbs_yostar::retro_table_generated::root_as_clz_torappu_retro_stage_table_unchecked;
                let root = unsafe { root_as_clz_torappu_retro_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            "roguelike_topic_table" => {
                use crate::generated_fbs_yostar::roguelike_topic_table_generated::root_as_clz_torappu_roguelike_topic_table_unchecked;
                let root = unsafe { root_as_clz_torappu_roguelike_topic_table_unchecked(data) };
                Ok(root.to_json())
            }
            "stage_table" => {
                use crate::generated_fbs_yostar::stage_table_generated::root_as_clz_torappu_stage_table_unchecked;
                let root = unsafe { root_as_clz_torappu_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            _ => Err(format!("No Yostar schema for {schema_type}")),
        }
    }));
    match decode_result {
        Ok(Ok(value)) => {
            if value.as_object().is_some_and(serde_json::Map::is_empty) {
                Err("Yostar decode returned empty".to_string())
            } else {
                Ok(value)
            }
        }
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Yostar decode panic".to_string()),
    }
}

/// Decode `FlatBuffer` data to JSON using schema-based decoding
pub fn decode_flatbuffer(data: &[u8], filename: &str) -> Result<Value, String> {
    use crate::fb_json_macros::FlatBufferToJson;

    if !is_flatbuffer(data) {
        return Err("Data is not a valid FlatBuffer".to_string());
    }

    let schema_type = guess_root_type(filename);

    // DECODE ONLY WHAT VERIFIES. `guess_root_type` names a schema; the
    // buffer is verified against it (and against the Yostar variant, if the
    // table has one) BEFORE anything is decoded, and a buffer that verifies
    // under neither is skipped rather than decoded.
    //
    // WHY there is no unchecked fallback any more: `root_as_*_unchecked` on a
    // buffer the schema does not match has no termination guarantee. It reads
    // a scalar as a vector length and walks a multi-million-element phantom
    // vector, one caught panic per element. Measured twice: EN stage_table
    // 26-08-28 under the 2.7.71 CN schema never finished (>45 min on one
    // 5,022,624-byte buffer), and the committed March-2026 CN fixture under
    // the same schema burned 10 min at 97% CPU writing 3.7 GB of garbage
    // before it was killed (sampled: 129 of 129 frames in
    // stage_table_generated). Verification rejects both in microseconds.
    //
    // WHY skipping beats decoding anyway: a missing table is explicit and a
    // garbage table is not. The caller (`export_gamedata`) writes no file, so
    // the previous extraction's JSON stays on disk — the backend tolerates a
    // missing non-critical table and degrades a non-default server, where a
    // multi-GB garbage table takes the process out (a 10 GB RSS OOM kill on
    // the VPS on 2026-08-04) and a plausible-looking one is worse still.
    //
    // Verification also costs nothing to be wrong about in the safe direction:
    // it allocates no `Value` and serialises nothing, it only walks offsets
    // and vtables.
    if schema_type != "unknown" {
        match select_schema_by_verification(data, schema_type) {
            SchemaChoice::Yostar => {
                if let Some((Some(e), _)) = verify_schemas(data, schema_type, false) {
                    eprintln!("schema: {schema_type} CN verify failed ({e}), Yostar verified");
                }
                return decode_flatbuffer_yostar(data, schema_type);
            }
            SchemaChoice::Neither => {
                return Err(format!("No schema verifies {schema_type}"));
            }
            SchemaChoice::Cn => {}
        }
    }

    let data_clone = data.to_vec();

    let decode_result = panic::catch_unwind(AssertUnwindSafe(|| {
        let data = &data_clone;
        match schema_type {
            "activity_table" => {
                use crate::generated_fbs::activity_table_generated::root_as_clz_torappu_activity_table_unchecked;
                let root = unsafe { root_as_clz_torappu_activity_table_unchecked(data) };
                Ok(root.to_json())
            }
            "arkvent_table" => {
                use crate::generated_fbs::arkvent_table_generated::root_as_clz_torappu_ark_odc_table_unchecked;
                let root = unsafe { root_as_clz_torappu_ark_odc_table_unchecked(data) };
                Ok(root.to_json())
            }
            "audio_data" => {
                use crate::generated_fbs::audio_data_generated::root_as_clz_torappu_audio_middleware_data_torappu_audio_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_audio_middleware_data_torappu_audio_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "bake_muzzle_data" => {
                use crate::generated_fbs::bake_muzzle_data_generated::root_as_clz_torappu_battle_baked_spine_data_unchecked;
                let root = unsafe { root_as_clz_torappu_battle_baked_spine_data_unchecked(data) };
                Ok(root.to_json())
            }
            "battle_equip_table" => {
                use crate::generated_fbs::battle_equip_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_battle_equip_pack_unchecked(data)
                };
                Ok(root.to_json())
            }
            "buff_table" => {
                use crate::generated_fbs::buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "building_data" => {
                use crate::generated_fbs::building_data_generated::root_as_clz_torappu_building_data_unchecked;
                let root = unsafe { root_as_clz_torappu_building_data_unchecked(data) };
                Ok(root.to_json())
            }
            "building_local_data" => {
                use crate::generated_fbs::building_local_data_generated::root_as_clz_torappu_building_data_building_local_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_building_data_building_local_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "campaign_table" => {
                use crate::generated_fbs::campaign_table_generated::root_as_clz_torappu_campaign_table_unchecked;
                let root = unsafe { root_as_clz_torappu_campaign_table_unchecked(data) };
                Ok(root.to_json())
            }
            "chapter_table" => {
                use crate::generated_fbs::chapter_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_chapter_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_chapter_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "char_master_table" => {
                use crate::generated_fbs::char_master_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_master_data_bundle_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_master_data_bundle_unchecked(data)
                };
                Ok(root.to_json())
            }
            "char_meta_table" => {
                use crate::generated_fbs::char_meta_table_generated::root_as_clz_torappu_char_meta_table_unchecked;
                let root = unsafe { root_as_clz_torappu_char_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "char_patch_table" => {
                use crate::generated_fbs::char_patch_table_generated::root_as_clz_torappu_char_patch_data_unchecked;
                let root = unsafe { root_as_clz_torappu_char_patch_data_unchecked(data) };
                Ok(root.to_json())
            }
            "character_table" => {
                use crate::generated_fbs::character_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "charm_table" => {
                use crate::generated_fbs::charm_table_generated::root_as_clz_torappu_charm_data_unchecked;
                let root = unsafe { root_as_clz_torappu_charm_data_unchecked(data) };
                Ok(root.to_json())
            }
            "charword_table" => {
                use crate::generated_fbs::charword_table_generated::root_as_clz_torappu_char_word_table_unchecked;
                let root = unsafe { root_as_clz_torappu_char_word_table_unchecked(data) };
                Ok(root.to_json())
            }
            "checkin_table" => {
                use crate::generated_fbs::checkin_table_generated::root_as_clz_torappu_check_in_table_unchecked;
                let root = unsafe { root_as_clz_torappu_check_in_table_unchecked(data) };
                Ok(root.to_json())
            }
            "climb_tower_table" => {
                use crate::generated_fbs::climb_tower_table_generated::root_as_clz_torappu_climb_tower_table_unchecked;
                let root = unsafe { root_as_clz_torappu_climb_tower_table_unchecked(data) };
                Ok(root.to_json())
            }
            "clue_data" => {
                use crate::generated_fbs::clue_data_generated::root_as_clz_torappu_meeting_clue_data_unchecked;
                let root = unsafe { root_as_clz_torappu_meeting_clue_data_unchecked(data) };
                Ok(root.to_json())
            }
            "cooperate_battle_table" => {
                use crate::generated_fbs::cooperate_battle_table_generated::root_as_clz_torappu_battle_cooperate_cooperate_mode_battle_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_battle_cooperate_cooperate_mode_battle_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "crisis_table" => {
                use crate::generated_fbs::crisis_table_generated::root_as_clz_torappu_crisis_client_data_unchecked;
                let root = unsafe { root_as_clz_torappu_crisis_client_data_unchecked(data) };
                Ok(root.to_json())
            }
            "crisis_v2_table" => {
                use crate::generated_fbs::crisis_v2_table_generated::root_as_clz_torappu_crisis_v2_shared_data_unchecked;
                let root = unsafe { root_as_clz_torappu_crisis_v2_shared_data_unchecked(data) };
                Ok(root.to_json())
            }
            "display_meta_table" => {
                use crate::generated_fbs::display_meta_table_generated::root_as_clz_torappu_display_meta_data_unchecked;
                let root = unsafe { root_as_clz_torappu_display_meta_data_unchecked(data) };
                Ok(root.to_json())
            }
            "enemy_database" => {
                use crate::generated_fbs::enemy_database_generated::root_as_clz_torappu_enemy_database_unchecked;
                let root = unsafe { root_as_clz_torappu_enemy_database_unchecked(data) };
                Ok(root.to_json())
            }
            "enemy_handbook_table" => {
                use crate::generated_fbs::enemy_handbook_table_generated::root_as_clz_torappu_enemy_hand_book_data_group_unchecked;
                let root =
                    unsafe { root_as_clz_torappu_enemy_hand_book_data_group_unchecked(data) };
                Ok(root.to_json())
            }
            "ep_breakbuff_table" => {
                use crate::generated_fbs::ep_breakbuff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_epbreak_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "extra_battlelog_table" => {
                use crate::generated_fbs::extra_battlelog_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_extra_battle_log_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_extra_battle_log_data_unchecked(
                        data,
                    )
                };
                Ok(root.to_json())
            }
            "favor_table" => {
                use crate::generated_fbs::favor_table_generated::root_as_clz_torappu_favor_table_unchecked;
                let root = unsafe { root_as_clz_torappu_favor_table_unchecked(data) };
                Ok(root.to_json())
            }
            "gacha_table" => {
                use crate::generated_fbs::gacha_table_generated::root_as_clz_torappu_gacha_data_unchecked;
                let root = unsafe { root_as_clz_torappu_gacha_data_unchecked(data) };
                Ok(root.to_json())
            }
            "gamedata_const" => {
                use crate::generated_fbs::gamedata_const_generated::root_as_clz_torappu_game_data_consts_unchecked;
                let root = unsafe { root_as_clz_torappu_game_data_consts_unchecked(data) };
                Ok(root.to_json())
            }
            "handbook_info_table" => {
                use crate::generated_fbs::handbook_info_table_generated::root_as_clz_torappu_handbook_info_table_unchecked;
                let root = unsafe { root_as_clz_torappu_handbook_info_table_unchecked(data) };
                Ok(root.to_json())
            }
            "handbook_team_table" => {
                use crate::generated_fbs::handbook_team_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_handbook_team_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_handbook_team_data_unchecked(
                        data,
                    )
                };
                Ok(root.to_json())
            }
            "hotupdate_meta_table" => {
                use crate::generated_fbs::hotupdate_meta_table_generated::root_as_clz_torappu_hot_update_meta_table_unchecked;
                let root = unsafe { root_as_clz_torappu_hot_update_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "init_text" => {
                use crate::generated_fbs::init_text_generated::root_as_clz_torappu_language_data_unchecked;
                let root = unsafe { root_as_clz_torappu_language_data_unchecked(data) };
                Ok(root.to_json())
            }
            "item_table" => {
                use crate::generated_fbs::item_table_generated::root_as_clz_torappu_inventory_data_unchecked;
                let root = unsafe { root_as_clz_torappu_inventory_data_unchecked(data) };
                Ok(root.to_json())
            }
            "legion_mode_buff_table" => {
                use crate::generated_fbs::legion_mode_buff_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_battle_legion_legion_mode_buff_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_battle_legion_legion_mode_buff_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "level_script_table" => {
                use crate::generated_fbs::level_script_table_generated::root_as_clz_torappu_battle_level_script_data_map_unchecked;
                let root =
                    unsafe { root_as_clz_torappu_battle_level_script_data_map_unchecked(data) };
                Ok(root.to_json())
            }
            "main_text" => {
                use crate::generated_fbs::main_text_generated::root_as_clz_torappu_language_data_unchecked;
                let root = unsafe { root_as_clz_torappu_language_data_unchecked(data) };
                Ok(root.to_json())
            }
            "medal_table" => {
                use crate::generated_fbs::medal_table_generated::root_as_clz_torappu_medal_data_unchecked;
                let root = unsafe { root_as_clz_torappu_medal_data_unchecked(data) };
                Ok(root.to_json())
            }
            "meta_ui_table" => {
                use crate::generated_fbs::meta_ui_table_generated::root_as_clz_torappu_meta_uidisplay_table_unchecked;
                let root = unsafe { root_as_clz_torappu_meta_uidisplay_table_unchecked(data) };
                Ok(root.to_json())
            }
            "mission_table" => {
                use crate::generated_fbs::mission_table_generated::root_as_clz_torappu_mission_table_unchecked;
                let root = unsafe { root_as_clz_torappu_mission_table_unchecked(data) };
                Ok(root.to_json())
            }
            "open_server_table" => {
                use crate::generated_fbs::open_server_table_generated::root_as_clz_torappu_open_server_schedule_unchecked;
                let root = unsafe { root_as_clz_torappu_open_server_schedule_unchecked(data) };
                Ok(root.to_json())
            }
            "prts___levels" => {
                use crate::generated_fbs::prts___levels_generated::root_as_clz_torappu_level_data_unchecked;
                let root = unsafe { root_as_clz_torappu_level_data_unchecked(data) };
                Ok(root.to_json())
            }
            "replicate_table" => {
                use crate::generated_fbs::replicate_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_replicate_table_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_replicate_table_unchecked(data)
                };
                Ok(root.to_json())
            }
            "resource_manifest" => {
                use crate::generated_fbs::resource_manifest_generated::root_as_clz_torappu_resource_resource_manifest_unchecked;
                let root =
                    unsafe { root_as_clz_torappu_resource_resource_manifest_unchecked(data) };
                Ok(root.to_json())
            }
            "retro_table" => {
                use crate::generated_fbs::retro_table_generated::root_as_clz_torappu_retro_stage_table_unchecked;
                let root = unsafe { root_as_clz_torappu_retro_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            "roguelike_topic_table" => {
                use crate::generated_fbs::roguelike_topic_table_generated::root_as_clz_torappu_roguelike_topic_table_unchecked;
                let root = unsafe { root_as_clz_torappu_roguelike_topic_table_unchecked(data) };
                Ok(root.to_json())
            }
            "sandbox_perm_table" => {
                use crate::generated_fbs::sandbox_perm_table_generated::root_as_clz_torappu_sandbox_perm_table_unchecked;
                let root = unsafe { root_as_clz_torappu_sandbox_perm_table_unchecked(data) };
                Ok(root.to_json())
            }
            "sandbox_table" => {
                use crate::generated_fbs::sandbox_table_generated::root_as_clz_torappu_sandbox_table_unchecked;
                let root = unsafe { root_as_clz_torappu_sandbox_table_unchecked(data) };
                Ok(root.to_json())
            }
            "shop_client_table" => {
                use crate::generated_fbs::shop_client_table_generated::root_as_clz_torappu_shop_client_data_unchecked;
                let root = unsafe { root_as_clz_torappu_shop_client_data_unchecked(data) };
                Ok(root.to_json())
            }
            "skill_table" => {
                use crate::generated_fbs::skill_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_skill_data_bundle_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_skill_data_bundle_unchecked(data)
                };
                Ok(root.to_json())
            }
            "skin_table" => {
                use crate::generated_fbs::skin_table_generated::root_as_clz_torappu_skin_table_unchecked;
                let root = unsafe { root_as_clz_torappu_skin_table_unchecked(data) };
                Ok(root.to_json())
            }
            "special_operator_table" => {
                use crate::generated_fbs::special_operator_table_generated::root_as_clz_torappu_special_operator_table_unchecked;
                let root = unsafe { root_as_clz_torappu_special_operator_table_unchecked(data) };
                Ok(root.to_json())
            }
            "stage_table" => {
                use crate::generated_fbs::stage_table_generated::root_as_clz_torappu_stage_table_unchecked;
                let root = unsafe { root_as_clz_torappu_stage_table_unchecked(data) };
                Ok(root.to_json())
            }
            "story_review_meta_table" => {
                use crate::generated_fbs::story_review_meta_table_generated::root_as_clz_torappu_story_review_meta_table_unchecked;
                let root = unsafe { root_as_clz_torappu_story_review_meta_table_unchecked(data) };
                Ok(root.to_json())
            }
            "story_review_table" => {
                use crate::generated_fbs::story_review_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_story_review_group_client_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_story_review_group_client_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "story_table" => {
                use crate::generated_fbs::story_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_story_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_story_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "tip_table" => {
                use crate::generated_fbs::tip_table_generated::root_as_clz_torappu_tip_table_unchecked;
                let root = unsafe { root_as_clz_torappu_tip_table_unchecked(data) };
                Ok(root.to_json())
            }
            "token_table" => {
                use crate::generated_fbs::token_table_generated::root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked;
                let root = unsafe {
                    root_as_clz_torappu_simple_kvtable_clz_torappu_character_data_unchecked(data)
                };
                Ok(root.to_json())
            }
            "uniequip_table" => {
                use crate::generated_fbs::uniequip_table_generated::root_as_clz_torappu_uni_equip_table_unchecked;
                let root = unsafe { root_as_clz_torappu_uni_equip_table_unchecked(data) };
                Ok(root.to_json())
            }
            "zone_table" => {
                use crate::generated_fbs::zone_table_generated::root_as_clz_torappu_zone_table_unchecked;
                let root = unsafe { root_as_clz_torappu_zone_table_unchecked(data) };
                Ok(root.to_json())
            }
            _ => Err(format!("Unknown schema type: {schema_type}")),
        }
    }));

    match decode_result {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(e)) => {
            // `guess_root_type` found no schema for this filename, so nothing
            // was verified and nothing was decoded. The string scavenger is the
            // only thing left, and it is bounds-checked and UTF-8-checked.
            if schema_type == "unknown" {
                let strings = extract_strings(data);
                if !strings.is_empty() {
                    return Ok(json!({ "type": "unknown", "strings": strings }));
                }
            }
            Err(format!("Decode failed for {schema_type}: {e}"))
        }
        // The buffer verified under this exact schema, so a panic here is a bug
        // in the emitted `to_json`, not a schema mismatch: report it, do not
        // silently retry under a schema that just failed verification.
        Err(_) => Err(format!("Decode panic for {schema_type}")),
    }
}

/// Extract strings from `FlatBuffer` (fallback for unknown types)
#[must_use]
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
