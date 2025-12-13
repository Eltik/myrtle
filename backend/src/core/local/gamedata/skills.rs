use std::collections::HashMap;

use crate::core::local::{
    asset_mapping::AssetMappings,
    types::{
        material::Materials,
        operator::{
            EnrichedSkill, LevelUpCostCond, LevelUpCostItem, OperatorSkillRef, SkillStatic,
        },
        skill::{RawSkill, Skill},
    },
};

/// Transform raw skills to enriched skills (add id and image URL)
pub fn enrich_all_skills(
    raw_skills: HashMap<String, RawSkill>,
    asset_mappings: &AssetMappings,
) -> HashMap<String, Skill> {
    raw_skills
        .into_iter()
        .map(|(id, raw)| {
            let skill = Skill {
                id: Some(id.clone()),
                skill_id: raw.skill_id,
                icon_id: raw.icon_id,
                image: Some(asset_mappings.get_skill_icon_path(&id)),
                hidden: raw.hidden,
                levels: raw.levels,
            };
            (id, skill)
        })
        .collect()
}

pub fn enrich_skills(
    raw_skills: &Vec<OperatorSkillRef>,
    skill_table: &HashMap<String, Skill>,
    materials: &Materials,
) -> Vec<EnrichedSkill> {
    raw_skills
        .iter()
        .filter_map(|skill_ref| {
            // Skip skills without skill_id
            let skill_id = skill_ref.skill_id.as_ref()?;

            let static_data = skill_table.get(skill_id).map(|skill| SkillStatic {
                levels: skill.levels.clone(),
                skill_id: skill.skill_id.clone(),
                icon_id: skill.icon_id.clone(),
                hidden: skill.hidden,
                image: skill.image.clone(),
            });

            // Enrich level up cost conditions with item icon paths
            let enriched_level_up_cost_cond =
                enrich_level_up_cost_cond(&skill_ref.level_up_cost_cond, materials);

            Some(EnrichedSkill {
                skill_id: skill_id.clone(),
                override_prefab_key: skill_ref.override_prefab_key.clone(),
                override_token_key: skill_ref.override_token_key.clone(),
                level_up_cost_cond: enriched_level_up_cost_cond,
                static_data,
            })
        })
        .collect()
}

/// Enrich level up cost conditions with item icon paths
fn enrich_level_up_cost_cond(
    level_up_cost_cond: &[LevelUpCostCond],
    materials: &Materials,
) -> Vec<LevelUpCostCond> {
    level_up_cost_cond
        .iter()
        .map(|cond| LevelUpCostCond {
            unlock_cond: cond.unlock_cond.clone(),
            lvl_up_time: cond.lvl_up_time,
            level_up_cost: cond
                .level_up_cost
                .iter()
                .map(|cost| enrich_mastery_cost_item(cost, materials))
                .collect(),
        })
        .collect()
}

/// Enrich a mastery cost item with icon_id and image path
fn enrich_mastery_cost_item(cost: &LevelUpCostItem, materials: &Materials) -> LevelUpCostItem {
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
