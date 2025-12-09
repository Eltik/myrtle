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
        module::{BattleEquip, RawModules},
        operator::{Operator, RawOperator},
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
    asset_mappings: &AssetMappings,
) -> Operator {
    let enriched_skills = enrich_skills(&raw.skills, skills);
    let operator_modules = get_operator_modules(id, modules, battle_equip, asset_mappings);
    let (handbook_item, profile) = get_handbook_and_profile(id, handbook);
    let artists = get_artists(id, skins);
    let portrait = asset_mappings.get_portrait_path(id);

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
        phases: raw.phases.clone(),
        skills: enriched_skills,
        display_token_dict: raw.display_token_dict.clone(),
        talents: raw.talents.clone().unwrap_or_default(),
        potential_ranks: raw.potential_ranks.clone(),
        favor_key_frames: raw.favor_key_frames.clone().unwrap_or_default(),
        all_skill_level_up: raw.all_skill_lvlup.clone(),
        modules: operator_modules,
        handbook: handbook_item,
        profile,
        artists,
        portrait,
    }
}
