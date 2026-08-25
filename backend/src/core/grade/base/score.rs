use crate::core::gamedata::types::GameData;
use crate::core::grade::base::assignment::{
    compute_current_assignment, compute_sustained_assignment, sustained_assignment_value,
};
use crate::database::models::roster::RosterEntry;

use super::{
    buff_registry::{build_name_to_char, build_registry, faction_tags_of},
    types::{OperatorBaseProfile, UserBuilding},
};

/// The base grade answers ONE question: does the player have the best base
/// THEY could have? Both sides of the ratio use their own roster on their own
/// built rooms, valued as sustained LMD-equivalent daily yield (morale
/// rotation and factory buffer stalls included, gold->trade coupled):
///
///   - ACTUAL: the base exactly as stationed in the sync;
///   - ACHIEVABLE: the optimizer's best staffing of the same rooms.
///
/// The ratio is log-curved so early improvements move the grade more than the
/// final few percent. Roster size, room levels and account progress do NOT
/// enter directly - upgrading a room or recruiting an operator raises both
/// sides - so the grade measures how well the player USES what they have,
/// not how much they have.
pub fn grade_base(
    roster: &[RosterEntry],
    building_json: Option<&serde_json::Value>,
    game_data: &GameData,
) -> f64 {
    let building_data = &game_data.building;
    if building_data.buffs.is_empty() {
        return 0.0;
    }
    let name_to_char = build_name_to_char(&game_data.operators);
    let (registry, morale_drains) = build_registry(&building_data.buffs, &name_to_char);

    let user_building = match building_json {
        Some(json) => UserBuilding::from_json(json),
        None => return 0.0, // No building data synced
    };
    if user_building.is_empty() {
        return 0.0;
    }
    let profiles = build_operator_profiles(roster, game_data);

    // The base as the player actually stationed it.
    let current = compute_current_assignment(
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
        None,
    );
    let actual = sustained_assignment_value(
        &current,
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
    );

    // The best base they could have: the same roster, the same rooms,
    // optimizer-staffed (cap-aware, so sustainable teams are preferred).
    let optimal = compute_sustained_assignment(
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
    );
    let achievable = sustained_assignment_value(
        &optimal.main,
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
    );
    if achievable <= 0.0 {
        return 0.0;
    }
    log_curve_ratio((actual / achievable).clamp(0.0, 1.0))
}

fn build_operator_profiles(
    roster: &[RosterEntry],
    game_data: &GameData,
) -> Vec<OperatorBaseProfile> {
    let mut profiles = Vec::new();
    for entry in roster {
        let building_char = game_data.building.chars.get(&entry.operator_id);

        if let Some(bc) = building_char {
            let static_op = game_data.operators.get(&entry.operator_id);
            let faction_tags = static_op.map(faction_tags_of).unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            profiles.push(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                &game_data.building,
                false,
            ));
        }
    }
    profiles
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}
