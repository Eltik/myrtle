use std::collections::HashMap;

use crate::core::gamedata::{
    assets::AssetIndex,
    types::{
        material::Materials,
        operator::{
            EnrichedSkill, LevelUpCostCond, LevelUpCostItem, OperatorSkillRef, SkillStatic,
        },
        skill::{RawSkill, Skill},
    },
};

use super::resolve_item_icon;

pub fn enrich_all_skills(
    raw_skills: HashMap<String, RawSkill>,
    assets: &AssetIndex,
) -> HashMap<String, Skill> {
    raw_skills
        .into_iter()
        .map(|(id, raw)| {
            let image = assets.skill_icon_path(&id).map(str::to_owned);
            let skill = Skill {
                id: Some(id.clone()),
                skill_id: raw.skill_id,
                icon_id: raw.icon_id,
                image,
                hidden: raw.hidden,
                levels: raw.levels,
            };
            (id, skill)
        })
        .collect()
}

pub fn enrich_operator_skills(
    skill_refs: &[OperatorSkillRef],
    skill_table: &HashMap<String, Skill>,
    materials: &Materials,
    assets: &AssetIndex,
) -> Vec<EnrichedSkill> {
    skill_refs
        .iter()
        .filter_map(|skill_ref| {
            let skill_id = skill_ref.skill_id.as_ref()?;

            let static_data = skill_table.get(skill_id).map(|skill| SkillStatic {
                levels: skill.levels.clone(),
                skill_id: skill.skill_id.clone(),
                icon_id: skill.icon_id.clone(),
                hidden: skill.hidden,
                image: skill.image.clone(),
            });

            let level_up_cost_cond =
                enrich_mastery_costs(&skill_ref.level_up_cost_cond, materials, assets);

            Some(EnrichedSkill {
                skill_id: skill_id.clone(),
                override_prefab_key: skill_ref.override_prefab_key.clone(),
                override_token_key: skill_ref.override_token_key.clone(),
                level_up_cost_cond,
                static_data,
            })
        })
        .collect()
}

fn enrich_mastery_costs(
    costs: &[LevelUpCostCond],
    materials: &Materials,
    assets: &AssetIndex,
) -> Vec<LevelUpCostCond> {
    costs
        .iter()
        .map(|cond| LevelUpCostCond {
            unlock_cond: cond.unlock_cond.clone(),
            lvl_up_time: cond.lvl_up_time,
            level_up_cost: cond
                .level_up_cost
                .iter()
                .map(|cost| {
                    let (icon_id, image) = resolve_item_icon(&cost.id, materials, assets);
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
