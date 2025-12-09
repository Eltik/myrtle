use regex::Regex;
use std::collections::HashMap;

use crate::core::local::types::{
    operator::{EnrichedSkill, OperatorSkillRef, SkillStatic},
    skill::{RawSkill, Skill},
};

/// Transform raw skills to enriched skills (add id and image URL)
pub fn enrich_all_skills(raw_skills: HashMap<String, RawSkill>) -> HashMap<String, Skill> {
    raw_skills
        .into_iter()
        .map(|(id, raw)| {
            let num = extract(&id)
                .map(|(_, n)| n)
                .unwrap_or_else(|| "unknown".to_string());

            let skill = Skill {
                id: Some(id.clone()),
                skill_id: raw.skill_id,
                icon_id: raw.icon_id,
                image: Some(format!(
                    "/upk/spritepack/skill_icons_{}/skill_icon_{}.png",
                    num, id
                )),
                hidden: raw.hidden,
                levels: raw.levels,
            };
            (id, skill)
        })
        .collect()
}

fn extract(s: &str) -> Option<(String, String)> {
    let re = Regex::new(r"^(.*?)(?:_|\[)?(\d+)\]?$").unwrap();

    let caps = re.captures(s)?;
    let prefix = caps.get(1)?.as_str().to_string();
    let num = caps.get(2)?.as_str().to_string();

    Some((prefix, num))
}

pub fn enrich_skills(
    raw_skills: &Vec<OperatorSkillRef>,
    skill_table: &HashMap<String, Skill>,
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

            Some(EnrichedSkill {
                skill_id: skill_id.clone(),
                override_prefab_key: skill_ref.override_prefab_key.clone(),
                override_token_key: skill_ref.override_token_key.clone(),
                level_up_cost_cond: skill_ref.level_up_cost_cond.clone(),
                static_data,
            })
        })
        .collect()
}
