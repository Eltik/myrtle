use std::{collections::HashMap, sync::LazyLock};

use myrtle_gamedata_raw::tables::consts::GameDataConst;
use regex::Regex;

fn room_type_from_label(label: &str) -> Option<&'static str> {
    Some(match label {
        "Factory" => "MANUFACTURE",
        "Trading Post" => "TRADING",
        "Control Center" => "CONTROL",
        "Power Plant" => "POWER",
        "Training Room" => "TRAINING",
        "Dormitory" => "DORMITORY",
        "Reception Room" => "MEETING",
        "Office" => "HIRE",
        "Workshop" => "WORKSHOP",
        _ => return None,
    })
}

#[must_use]
pub fn layout_term_rooms(consts: &GameDataConst) -> HashMap<String, Vec<String>> {
    static RE_FOR_EVERY: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"For every ([A-Za-z ,]+?), [A-Za-z ]+ \+1").expect("valid regex")
    });
    consts
        .term_description_dict
        .iter()
        .filter_map(|e| {
            let id = e.value.term_id.strip_prefix("cc.")?.to_string();
            let c = RE_FOR_EVERY.captures(&e.value.description)?;
            let rooms: Vec<String> = c[1]
                .split(" and ")
                .flat_map(|part| part.split(", "))
                .filter_map(|label| room_type_from_label(label.trim()))
                .map(str::to_string)
                .collect();
            (!rooms.is_empty()).then_some((id, rooms))
        })
        .collect()
}
