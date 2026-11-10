use std::collections::HashMap;

use myrtle_gamedata_raw::tables::operator::Operator;

use super::parse::plain_text;
use myrtle_gamedata_raw::tables::consts::GameDataConst;

/// Lowercased display name -> `char_id`
pub fn build_name_to_char(operators: &HashMap<String, Operator>) -> HashMap<String, String> {
    operators
        .iter()
        .map(|(char_id, op)| (op.name.to_lowercase(), char_id.clone()))
        .collect()
}

/// Lowercased group/nation/team ids, the match tags for count scalers
pub fn faction_tags_of(op: &Operator) -> Vec<String> {
    let mut tags = Vec::new();
    let mut push = |s: &str| {
        if !s.is_empty() {
            tags.push(s.to_lowercase());
        }
    };
    push(&op.nation_id);
    if let Some(g) = &op.group_id {
        push(g);
    }
    if let Some(t) = &op.team_id {
        push(t);
    }
    if op.tag_list.iter().any(|s| s == "Robot") {
        push("robot");
    }
    if let Some(race) = op
        .profile
        .as_ref()
        .and_then(|p| serde_json::to_value(&p.basic_info.race).ok())
        .and_then(|v| v.as_str().map(str::to_lowercase))
    {
        push(&race);
    }
    let powers = op.main_power.iter().chain(op.sub_power.iter().flatten());
    for nation in powers.filter_map(|p| p.nation_id.as_ref()) {
        let lower = nation.to_lowercase();
        if !lower.is_empty() && !tags.contains(&lower) {
            tags.push(lower);
        }
    }
    tags
}

pub fn build_faction_map(operators: &HashMap<String, Operator>) -> HashMap<String, Vec<String>> {
    operators
        .iter()
        .map(|(char_id, op)| (char_id.clone(), faction_tags_of(op)))
        .collect()
}

/// `char_id` -> glossary tags
pub fn glossary_tags(
    consts: &GameDataConst,
    name_to_char: &HashMap<String, String>,
) -> HashMap<String, Vec<String>> {
    let mut out: HashMap<String, Vec<String>> = HashMap::new();
    for entry in &consts.term_description_dict {
        let Some(tag) = entry.key.strip_prefix("cc.tag.") else {
            continue;
        };
        let desc = plain_text(&entry.value.description);
        let Some((_, names)) = desc.split_once("following") else {
            continue;
        };
        for name in names.split([',', '\n']) {
            let name = name
                .trim()
                .trim_matches(|c| c == '\'' || c == '"' || c == ':')
                .trim()
                .to_lowercase();
            if let Some(id) = name_to_char.get(&name) {
                let tags = out.entry(id.clone()).or_default();
                let tag = tag.to_lowercase();
                if !tags.contains(&tag) {
                    tags.push(tag);
                }
            }
        }
    }
    out
}

pub fn faction_tags_for(
    char_id: &str,
    op: &Operator,
    glossary: &HashMap<String, Vec<String>>,
) -> Vec<String> {
    let mut tags = faction_tags_of(op);
    for tag in glossary.get(char_id).into_iter().flatten() {
        if !tags.contains(tag) {
            tags.push(tag.clone());
        }
    }
    tags
}
