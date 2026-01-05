use rayon::prelude::*;
use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    gamedata::{
        handbook::get_handbook_and_profile, modules::get_operator_modules, skills::enrich_skills,
        skins::get_artists,
    },
    types::{
        handbook::Handbook,
        material::Materials,
        module::{BattleEquip, RawModules},
        operator::{
            Drone, EvolveCost, LevelUpCostItem, Operator, OperatorProfession, Phase, RawOperator,
        },
        skill::Skill,
        skin::SkinData,
    },
};

/// Pre-enriches all operators once at startup (instead of per-request)
#[allow(clippy::too_many_arguments)]
pub fn enrich_all_operators(
    raw_operators: &HashMap<String, RawOperator>,
    drones: &HashMap<String, Drone>,
    skills: &HashMap<String, Skill>,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    handbook: &Handbook,
    skins: &SkinData,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> HashMap<String, Operator> {
    raw_operators
        .par_iter()
        .filter(|(id, _)| id.starts_with("char_"))
        .map(|(id, raw)| {
            let enriched = enrich_operator(
                id,
                raw,
                drones,
                skills,
                modules,
                battle_equip,
                handbook,
                skins,
                materials,
                asset_mappings,
            );
            (id.clone(), enriched)
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn enrich_operator(
    id: &str,
    raw: &RawOperator,
    drones: &HashMap<String, Drone>,
    skills: &HashMap<String, Skill>,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    handbook: &Handbook,
    skins: &SkinData,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Operator {
    let enriched_skills = enrich_skills(&raw.skills, skills, materials);
    let operator_modules =
        get_operator_modules(id, modules, battle_equip, materials, asset_mappings);
    let (handbook_item, profile) = get_handbook_and_profile(id, handbook);
    let artists = get_artists(id, skins);
    let portrait = asset_mappings.get_portrait_path(id);
    let skin = asset_mappings.get_charart_path(id);

    // Enrich phases with item icon paths
    let enriched_phases = enrich_phases(&raw.phases, materials, asset_mappings);

    // Enrich skill level up costs
    let enriched_all_skill_level_up =
        enrich_all_skill_level_up(&raw.all_skill_lvlup, materials, asset_mappings);

    // Resolve drones from display_token_dict, skills' overrideTokenKey, or by ID pattern
    // First, collect from display_token_dict (e.g., Magallan, Ling in CN data)
    let mut operator_drones: Vec<Drone> = raw
        .display_token_dict
        .as_ref()
        .map(|dict| {
            dict.keys()
                .filter_map(|key| drones.get(key).cloned())
                .collect()
        })
        .unwrap_or_default();

    // Also collect from skills' overrideTokenKey (e.g., Kazemaru's shadow doll)
    // These are skill-specific summons not in displayTokenDict
    for skill in &raw.skills {
        if let Some(ref token_key) = skill.override_token_key {
            if let Some(drone) = drones.get(token_key) {
                // Only add if not already in the list
                if !operator_drones.iter().any(|d| d.id == drone.id) {
                    operator_drones.push(drone.clone());
                }
            }
        }
    }

    // Fallback: find drones by ID pattern matching (EN data doesn't have displayTokenDict)
    // Pattern: char_248_mgllan -> token_*_mgllan_*
    if operator_drones.is_empty() {
        // Extract the name part from operator ID (e.g., "mgllan" from "char_248_mgllan")
        let parts: Vec<&str> = id.split('_').collect();
        if parts.len() >= 3 && parts[0] == "char" {
            let name_part = parts[2]; // e.g., "mgllan", "kazema", "ling"
            let pattern = format!("_{name_part}_");

            // Find all tokens matching this operator's name
            let mut matched_drones: Vec<Drone> = drones
                .iter()
                .filter(|(key, _)| key.starts_with("token_") && key.contains(&pattern))
                .map(|(_, drone)| drone.clone())
                .collect();

            // Sort by key to ensure consistent ordering (drone1, drone2, drone3)
            matched_drones.sort_by(|a, b| {
                let a_id = a.id.as_deref().unwrap_or("");
                let b_id = b.id.as_deref().unwrap_or("");
                a_id.cmp(b_id)
            });

            operator_drones = matched_drones;
        }
    }

    // Sort drones by ID to ensure consistent ordering (drone1, drone2, drone3)
    // This is important because skill->drone mapping depends on the order
    operator_drones.sort_by(|a, b| {
        let a_id = a.id.as_deref().unwrap_or("");
        let b_id = b.id.as_deref().unwrap_or("");
        a_id.cmp(b_id)
    });

    Operator {
        id: Some(id.to_string()),
        name: raw.name.clone(),
        description: raw.description.clone().unwrap_or_default(),
        can_use_general_potential_item: raw.can_use_general_potential_item,
        can_use_activity_potential_item: raw.can_use_activity_potential_item,
        potential_item_id: raw.potential_item_id.clone().unwrap_or_default(),
        activity_potential_item_id: raw.activity_potential_item_id.clone(),
        classic_potential_item_id: raw.classic_potential_item_id.clone(),
        nation_id: raw.nation_id.clone().unwrap_or_default(),
        group_id: raw.group_id.clone(),
        team_id: raw.team_id.clone(),
        display_number: raw.display_number.clone().unwrap_or_default(),
        appellation: raw.appellation.clone(),
        position: raw.position.clone(),
        tag_list: raw.tag_list.clone().unwrap_or_default(),
        item_usage: raw.item_usage.clone().unwrap_or_default(),
        item_desc: raw.item_desc.clone().unwrap_or_default(),
        item_obtain_approach: raw.item_obtain_approach.clone().unwrap_or_default(),
        is_not_obtainable: raw.is_not_obtainable,
        is_sp_char: raw.is_sp_char,
        max_potential_level: raw.max_potential_level,
        rarity: raw.rarity.clone(),
        profession: raw.profession.clone(),
        sub_profession_id: raw.sub_profession_id.clone().unwrap_or_default(),
        trait_data: raw.trait_data.clone(),
        phases: enriched_phases,
        skills: enriched_skills,
        display_token_dict: raw.display_token_dict.clone(),
        drones: operator_drones,
        talents: raw.talents.clone().unwrap_or_default(),
        potential_ranks: raw.potential_ranks.clone(),
        favor_key_frames: raw.favor_key_frames.clone().unwrap_or_default(),
        all_skill_level_up: enriched_all_skill_level_up,
        modules: operator_modules,
        handbook: handbook_item,
        profile,
        artists,
        portrait,
        skin,
    }
}

/// Enrich phases with item icon paths for evolve costs
fn enrich_phases(
    phases: &[Phase],
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Vec<Phase> {
    phases
        .iter()
        .map(|phase| {
            let enriched_evolve_cost = phase.evolve_cost.as_ref().map(|costs| {
                costs
                    .iter()
                    .map(|cost| enrich_evolve_cost(cost, materials, asset_mappings))
                    .collect()
            });

            Phase {
                character_prefab_key: phase.character_prefab_key.clone(),
                range_id: phase.range_id.clone(),
                max_level: phase.max_level,
                attributes_key_frames: phase.attributes_key_frames.clone(),
                evolve_cost: enriched_evolve_cost,
            }
        })
        .collect()
}

/// Enrich a single evolve cost item with icon_id and image path
fn enrich_evolve_cost(
    cost: &EvolveCost,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> EvolveCost {
    let (icon_id, image) = get_item_icon_info(&cost.id, materials, asset_mappings);

    EvolveCost {
        id: cost.id.clone(),
        count: cost.count,
        item_type: cost.item_type.clone(),
        icon_id,
        image,
    }
}

/// Enrich all skill level up costs
fn enrich_all_skill_level_up(
    all_skill_lvlup: &[crate::core::local::types::operator::AllSkillLevelUp],
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Vec<crate::core::local::types::operator::AllSkillLevelUp> {
    all_skill_lvlup
        .iter()
        .map(
            |lvlup| crate::core::local::types::operator::AllSkillLevelUp {
                unlock_cond: lvlup.unlock_cond.clone(),
                lvl_up_cost: lvlup
                    .lvl_up_cost
                    .iter()
                    .map(|cost| enrich_level_up_cost_item(cost, materials, asset_mappings))
                    .collect(),
            },
        )
        .collect()
}

/// Enrich a level up cost item with icon_id and image path
fn enrich_level_up_cost_item(
    cost: &LevelUpCostItem,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> LevelUpCostItem {
    let (icon_id, image) = get_item_icon_info(&cost.id, materials, asset_mappings);

    LevelUpCostItem {
        id: cost.id.clone(),
        count: cost.count,
        item_type: cost.item_type.clone(),
        icon_id,
        image,
    }
}

/// Get icon_id and image path for an item ID
fn get_item_icon_info(
    item_id: &str,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> (Option<String>, Option<String>) {
    // Look up in items table
    if let Some(item) = materials.items.get(item_id) {
        let icon_id = Some(item.icon_id.clone());
        let image = Some(asset_mappings.get_item_icon_path(&item.icon_id));
        return (icon_id, image);
    }

    // Look up in exp_items table - exp items use their id as the icon name
    if materials.exp_items.contains_key(item_id) {
        let icon_id = Some(item_id.to_string());
        let image = Some(asset_mappings.get_item_icon_path(item_id));
        return (icon_id, image);
    }

    // Fallback: use the item_id as icon_id
    (
        Some(item_id.to_string()),
        Some(asset_mappings.get_item_icon_path(item_id)),
    )
}

/// Convert OperatorProfession enum to its serialized string representation
fn profession_to_string(profession: &OperatorProfession) -> String {
    match profession {
        OperatorProfession::Medic => "MEDIC".to_string(),
        OperatorProfession::Caster => "CASTER".to_string(),
        OperatorProfession::Guard => "WARRIOR".to_string(),
        OperatorProfession::Vanguard => "PIONEER".to_string(),
        OperatorProfession::Sniper => "SNIPER".to_string(),
        OperatorProfession::Specialist => "SPECIAL".to_string(),
        OperatorProfession::Supporter => "SUPPORT".to_string(),
        OperatorProfession::Defender => "TANK".to_string(),
        OperatorProfession::Token => "TOKEN".to_string(),
        OperatorProfession::Trap => "TRAP".to_string(),
    }
}

/// Extract all drones/tokens from the character table
/// Drones are entries that don't start with "char_" (e.g., "token_10000_silent")
pub fn extract_all_drones(raw_operators: &HashMap<String, RawOperator>) -> HashMap<String, Drone> {
    raw_operators
        .par_iter()
        .filter(|(id, _)| !id.starts_with("char_"))
        .map(|(id, raw)| {
            let drone = Drone {
                id: Some(id.clone()),
                name: raw.name.clone(),
                description: raw.description.clone().unwrap_or_default(),
                can_use_general_potential_item: raw.can_use_general_potential_item,
                can_use_activity_potential_item: raw.can_use_activity_potential_item,
                potential_item_id: raw.potential_item_id.clone(),
                activity_potential_item_id: raw.activity_potential_item_id.clone(),
                classic_potential_item_id: raw.classic_potential_item_id.clone(),
                nation_id: raw.nation_id.clone(),
                group_id: raw.group_id.clone(),
                team_id: raw.team_id.clone(),
                display_number: raw.display_number.clone(),
                appellation: raw.appellation.clone(),
                position: raw.position.clone(),
                tag_list: raw.tag_list.clone().unwrap_or_default(),
                item_usage: raw.item_usage.clone(),
                item_desc: raw.item_desc.clone(),
                item_obtain_approach: raw.item_obtain_approach.clone(),
                is_not_obtainable: raw.is_not_obtainable,
                is_sp_char: raw.is_sp_char,
                max_potential_level: raw.max_potential_level,
                rarity: raw.rarity.clone(),
                profession: profession_to_string(&raw.profession),
                sub_profession_id: raw.sub_profession_id.clone().unwrap_or_default(),
                trait_data: raw.trait_data.clone(),
                phases: raw.phases.clone(),
                skills: raw.skills.clone(),
                display_token_dict: raw.display_token_dict.clone(),
                talents: raw.talents.clone().unwrap_or_default(),
                potential_ranks: raw.potential_ranks.clone(),
                favor_key_frames: None,
                all_skill_lvlup: raw.all_skill_lvlup.clone(),
                modules: Vec::new(),
            };
            (id.clone(), drone)
        })
        .collect()
}
