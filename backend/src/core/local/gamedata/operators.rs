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
        operator::{EvolveCost, LevelUpCostItem, Operator, Phase, RawOperator},
        skill::Skill,
        skin::SkinData,
    },
};

/// Pre-enriches all operators once at startup (instead of per-request)
pub fn enrich_all_operators(
    raw_operators: &HashMap<String, RawOperator>,
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

fn enrich_operator(
    id: &str,
    raw: &RawOperator,
    skills: &HashMap<String, Skill>,
    modules: &RawModules,
    battle_equip: &BattleEquip,
    handbook: &Handbook,
    skins: &SkinData,
    materials: &Materials,
    asset_mappings: &AssetMappings,
) -> Operator {
    let enriched_skills = enrich_skills(&raw.skills, skills, materials);
    let operator_modules = get_operator_modules(id, modules, battle_equip, asset_mappings);
    let (handbook_item, profile) = get_handbook_and_profile(id, handbook);
    let artists = get_artists(id, skins);
    let portrait = asset_mappings.get_portrait_path(id);
    let skin = asset_mappings.get_charart_path(id);

    // Enrich phases with item icon paths
    let enriched_phases = enrich_phases(&raw.phases, materials);

    // Enrich skill level up costs
    let enriched_all_skill_level_up = enrich_all_skill_level_up(&raw.all_skill_lvlup, materials);

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
fn enrich_phases(phases: &[Phase], materials: &Materials) -> Vec<Phase> {
    phases
        .iter()
        .map(|phase| {
            let enriched_evolve_cost = phase.evolve_cost.as_ref().map(|costs| {
                costs
                    .iter()
                    .map(|cost| enrich_evolve_cost(cost, materials))
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
fn enrich_evolve_cost(cost: &EvolveCost, materials: &Materials) -> EvolveCost {
    let (icon_id, image) = get_item_icon_info(&cost.id, materials);

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
) -> Vec<crate::core::local::types::operator::AllSkillLevelUp> {
    all_skill_lvlup
        .iter()
        .map(
            |lvlup| crate::core::local::types::operator::AllSkillLevelUp {
                unlock_cond: lvlup.unlock_cond.clone(),
                lvl_up_cost: lvlup
                    .lvl_up_cost
                    .iter()
                    .map(|cost| enrich_level_up_cost_item(cost, materials))
                    .collect(),
            },
        )
        .collect()
}

/// Enrich a level up cost item with icon_id and image path
fn enrich_level_up_cost_item(cost: &LevelUpCostItem, materials: &Materials) -> LevelUpCostItem {
    let (icon_id, image) = get_item_icon_info(&cost.id, materials);

    LevelUpCostItem {
        id: cost.id.clone(),
        count: cost.count,
        item_type: cost.item_type.clone(),
        icon_id,
        image,
    }
}

/// Get icon_id and image path for an item ID
fn get_item_icon_info(item_id: &str, materials: &Materials) -> (Option<String>, Option<String>) {
    // Look up in items table
    if let Some(item) = materials.items.get(item_id) {
        let icon_id = Some(item.icon_id.clone());
        let image = Some(format!("/upk/arts/items/icons/{}.png", item.icon_id));
        return (icon_id, image);
    }

    // Look up in exp_items table - exp items use their id as the icon name
    if materials.exp_items.contains_key(item_id) {
        let icon_id = Some(item_id.to_string());
        let image = Some(format!("/upk/arts/items/icons/{}.png", item_id));
        return (icon_id, image);
    }

    // Fallback: use the item_id as icon_id
    (
        Some(item_id.to_string()),
        Some(format!("/upk/arts/items/icons/{}.png", item_id)),
    )
}
