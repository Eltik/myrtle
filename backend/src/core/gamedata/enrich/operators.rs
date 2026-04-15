use rayon::prelude::*;
use std::collections::HashMap;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        building::BuildingDataFile,
        handbook::Handbook,
        material::Materials,
        module::{BattleEquip, RawModules},
        operator::{
            AllSkillLevelUp, Drone, EvolveCost, LevelUpCostItem, Operator, OperatorBaseSkill,
            Phase, RawOperator,
        },
        skill::Skill,
        skin::SkinData,
    },
};

use super::handbook::get_handbook_and_profile;
use super::modules::get_operator_modules;
use super::resolve_item_icon;
use super::skills::enrich_operator_skills;
use super::skins::get_artists;

pub struct EnrichCtx<'a> {
    pub skills: &'a HashMap<String, Skill>,
    pub modules: &'a RawModules,
    pub battle_equip: &'a BattleEquip,
    pub handbook: &'a Handbook,
    pub skins: &'a SkinData,
    pub materials: &'a Materials,
    pub assets: &'a AssetIndex,
    pub drones: &'a HashMap<String, Drone>,
    pub building: &'a BuildingDataFile,
}

pub fn enrich_all_operators(
    raw_operators: &HashMap<String, RawOperator>,
    ctx: &EnrichCtx,
) -> HashMap<String, Operator> {
    raw_operators
        .par_iter()
        .filter(|(id, _)| id.starts_with("char_"))
        .map(|(id, raw)| {
            let enriched = enrich_operator(id, raw, ctx);
            (id.clone(), enriched)
        })
        .collect()
}

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
                profession: raw.profession.to_raw_str().to_owned(),
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

fn enrich_operator(id: &str, raw: &RawOperator, ctx: &EnrichCtx) -> Operator {
    let enriched_phases = enrich_phases(&raw.phases, ctx);
    let enriched_all_skill_level_up = enrich_all_skill_level_up(&raw.all_skill_lvlup, ctx);
    let operator_drones = resolve_drones(id, raw, ctx.drones);

    let enriched_skills =
        enrich_operator_skills(&raw.skills, ctx.skills, ctx.materials, ctx.assets);
    let operator_modules =
        get_operator_modules(id, ctx.modules, ctx.battle_equip, ctx.materials, ctx.assets);
    let (handbook_item, profile) = get_handbook_and_profile(id, ctx.handbook);
    let artists = get_artists(id, ctx.skins);

    let portrait = ctx.assets.portrait_path(id).map(str::to_owned);
    let skin = ctx.assets.charart_path(id);

    let base_skills = get_operator_base_skills(id, ctx.building);

    Operator {
        id: Some(id.to_owned()),
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
        base_skills,
        portrait,
        skin,
    }
}

fn resolve_drones(id: &str, raw: &RawOperator, drones: &HashMap<String, Drone>) -> Vec<Drone> {
    let mut result: Vec<Drone> = Vec::new();

    // Strategy 1: display_token_dict
    if let Some(dict) = &raw.display_token_dict {
        for key in dict.keys() {
            if let Some(drone) = drones.get(key) {
                result.push(drone.clone());
            }
        }
    }

    // Strategy 2: skill override_token_key
    for skill in &raw.skills {
        if let Some(ref token_key) = skill.override_token_key
            && let Some(drone) = drones.get(token_key)
            && !result.iter().any(|d| d.id == drone.id)
        {
            result.push(drone.clone());
        }
    }

    // Strategy 3: ID pattern fallback (if nothing found yet)
    if result.is_empty()
        && let Some(name_part) = id.split('_').nth(2)
    {
        let pattern = format!("_{name_part}_");
        result = drones
            .iter()
            .filter(|(key, _)| key.starts_with("token_") && key.contains(&pattern))
            .map(|(_, drone)| drone.clone())
            .collect();
    }

    result.sort_by(|a, b| {
        let a_id = a.id.as_deref().unwrap_or("");
        let b_id = b.id.as_deref().unwrap_or("");
        a_id.cmp(b_id)
    });

    result
}

fn enrich_phases(phases: &[Phase], ctx: &EnrichCtx) -> Vec<Phase> {
    phases
        .iter()
        .map(|phase| Phase {
            character_prefab_key: phase.character_prefab_key.clone(),
            range_id: phase.range_id.clone(),
            max_level: phase.max_level,
            attributes_key_frames: phase.attributes_key_frames.clone(),
            evolve_cost: phase
                .evolve_cost
                .as_ref()
                .map(|costs| costs.iter().map(|c| enrich_evolve_cost(c, ctx)).collect()),
        })
        .collect()
}

fn enrich_evolve_cost(cost: &EvolveCost, ctx: &EnrichCtx) -> EvolveCost {
    let (icon_id, image) = resolve_item_icon(&cost.id, ctx.materials, ctx.assets);
    EvolveCost {
        id: cost.id.clone(),
        count: cost.count,
        item_type: cost.item_type.clone(),
        icon_id,
        image,
    }
}

fn enrich_all_skill_level_up(all: &[AllSkillLevelUp], ctx: &EnrichCtx) -> Vec<AllSkillLevelUp> {
    all.iter()
        .map(|lvlup| AllSkillLevelUp {
            unlock_cond: lvlup.unlock_cond.clone(),
            lvl_up_cost: lvlup
                .lvl_up_cost
                .iter()
                .map(|cost| {
                    let (icon_id, image) = resolve_item_icon(&cost.id, ctx.materials, ctx.assets);
                    LevelUpCostItem {
                        id: cost.id.clone(),
                        count: cost.count,
                        item_type: cost.item_type.clone(),
                        icon_id,
                        image,
                    }
                })
                .collect(),
        })
        .collect()
}

fn get_operator_base_skills(char_id: &str, building: &BuildingDataFile) -> Vec<OperatorBaseSkill> {
    let Some(building_char) = building.chars.get(char_id) else {
        return Vec::new();
    };

    let mut skills = Vec::new();
    for slot in &building_char.buff_char {
        for entry in &slot.buff_data {
            if let Some(buff) = building.buffs.get(&entry.buff_id) {
                skills.push(OperatorBaseSkill {
                    buff_id: entry.buff_id.clone(),
                    buff_name: buff.buff_name.clone(),
                    description: buff.description.clone(),
                    room_type: buff.room_type.clone(),
                    efficiency: buff.efficiency,
                    targets: buff.targets.clone(),
                    unlock_elite: entry.cond.elite(),
                    unlock_level: entry.cond.level,
                });
            }
        }
    }
    skills
}
