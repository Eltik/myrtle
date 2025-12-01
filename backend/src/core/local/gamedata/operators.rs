use std::collections::HashMap;

use crate::core::local::types::{operator::{EnrichedOperator, RawOperator}, skill::{EnrichedSkill, RawSkill}};

/// Pre-enriches all operators once at startup (instead of per-request)
pub fn enrich_all_operators(
    characters: &HashMap<String, RawOperator>,
    skills: &HashMap<String, RawSkill>,
    modules: &ModuleData,
    battle_equip: &HashMap<String, BattleEquip>,
    handbook: &HashMap<String, HandbookItem>,
    voices: &VoiceData,
    skins: &SkinData,
) -> HashMap<String, EnrichedOperatorr> {
    characters
        .par_iter()  // Parallel iteration with rayon
        .filter(|(id, _)| id.starts_with("char_"))
        .map(|(id, op)| {
            let enriched = enrich_operator(
                id, op, skills, modules, battle_equip, handbook, voices, skins
            );
            (id.clone(), enriched)
        })
        .collect()
}

fn enrich_operator(
    id: &str,
    op: &RawOperator,
    skills: &HashMap<String, RawSkill>,
    // ... other tables
) -> EnrichedOperator {
    // Skills: O(1) lookup per skill
    let enriched_skills: Vec<EnrichedSkill> = op.skills
        .iter()
        .filter_map(|s| {
            skills.get(&s.skill_id).map(|static_data| {
                EnrichedSkill {
                    base: s.clone(),
                    static_data: static_data.clone(),
                }
            })
        })
        .collect();

    // Modules: Pre-indexed by char_id
    let char_modules = modules.equip_dict
        .values()
        .filter(|m| m.char_id == id)
        .map(|m| {
            let details = battle_equip.get(&m.uni_equip_id);
            EnrichedModule { base: m.clone(), data: details.cloned() }
        })
        .collect();

    // Profile parsing
    let profile = handbook.get(id)
        .and_then(|h| parse_operator_profile(&h.story_text_audio).ok());

    EnrichedOperator {
        id: id.to_string(),
        base: op.clone(),
        skills: enriched_skills,
        modules: char_modules,
        profile,
        // ...
    }
}
