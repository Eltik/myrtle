use crate::core::gamedata::types::{GameData, building::BuildingDataFile};
use crate::core::grade::base::assignment::{
    compute_live_assignment, compute_sustained_assignment, sustained_assignment_value,
};
use crate::database::models::roster::RosterEntry;

use super::{
    buff_registry::{BuffResolutionStrategy, build_name_to_char, build_registry, faction_tags_of},
    types::{OperatorBaseProfile, UserBuilding},
};
use std::collections::HashMap;

/// Weight of stationing UTILIZATION: how close the synced stationing comes to
/// the optimizer's best staffing of the same rooms. The dominant term - the
/// grade is first and foremost "do you have the best base you could have".
const UTILIZATION_WEIGHT: f64 = 0.75;
/// Weight of infrastructure COMPLETENESS: what the built rooms can achieve
/// versus the same rooms at max level. Deliberately the smaller share -
/// account progress matters, but less than using what you have well.
const INFRASTRUCTURE_WEIGHT: f64 = 0.25;

/// The base grade with its two components, each already log-curved to [0, 1].
/// `score` is the weighted blend the profile stores; the components are kept
/// so the frontend can show WHICH term drags the grade.
#[derive(Debug, Clone, Copy, Default)]
pub struct BaseGrade {
    pub score: f64,
    /// Stationing utilization: actual vs achievable on the built rooms.
    pub utilization: f64,
    /// Infrastructure completeness: achievable as built vs at max level.
    pub infrastructure: f64,
}

impl BaseGrade {
    fn blend(utilization: f64, infrastructure: f64) -> Self {
        Self {
            score: UTILIZATION_WEIGHT * utilization + INFRASTRUCTURE_WEIGHT * infrastructure,
            utilization,
            infrastructure,
        }
    }
}

/// The base grade answers: does the player have the best base THEY could
/// have? Two log-curved ratios, both valued as sustained LMD-equivalent
/// daily yield (morale rotation and factory buffer stalls included,
/// gold->trade coupled), blended 75/25:
///
///   - UTILIZATION (75%): the base as stationed in the sync, versus the
///     optimizer's best staffing of the same roster on the same rooms.
///     Roster size and room levels cancel out of this ratio - it measures
///     how well the player USES what they have.
///   - INFRASTRUCTURE (25%): what those rooms can achieve as built, versus
///     the same rooms upgraded to max level (same layout, same roster;
///     furniture ambience is left as synced - it is decor, not a room
///     level). This is the "keep upgrading" pressure, kept deliberately
///     below the stationing term.
pub fn grade_base(
    roster: &[RosterEntry],
    building_json: Option<&serde_json::Value>,
    game_data: &GameData,
) -> BaseGrade {
    let building_data = &game_data.building;
    if building_data.buffs.is_empty() {
        return BaseGrade::default();
    }
    let name_to_char = build_name_to_char(&game_data.operators);
    let (registry, morale_drains) = build_registry(&building_data.buffs, &name_to_char);

    let user_building = match building_json {
        Some(json) => UserBuilding::from_json(json),
        None => return BaseGrade::default(), // No building data synced
    };
    if user_building.is_empty() {
        return BaseGrade::default();
    }
    let profiles = build_operator_profiles(roster, game_data);

    // Achievable on the rooms as BUILT: the optimizer's cap-aware best
    // staffing of the player's own roster (denominator of utilization,
    // numerator of infrastructure).
    let achievable = best_yield(
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
    );
    if achievable <= 0.0 {
        return BaseGrade::default();
    }

    // The base as the player actually stationed it, with each bar as the
    // game last wrote it.
    let live_morale =
        building_json.map_or_else(HashMap::new, super::sustain_sim::synced_live_morale);
    let current = compute_live_assignment(
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
        None,
        &live_morale,
    );
    let actual = sustained_assignment_value(
        &current,
        &profiles,
        &user_building,
        building_data,
        &registry,
        &morale_drains,
    );
    let utilization = log_curve_ratio((actual / achievable).clamp(0.0, 1.0));

    // The same rooms at max level: what finishing the upgrades would unlock.
    let maxed_building = max_level_building(&user_building, building_data);
    let ceiling = best_yield(
        &profiles,
        &maxed_building,
        building_data,
        &registry,
        &morale_drains,
    );
    let infrastructure = if ceiling > 0.0 {
        log_curve_ratio((achievable / ceiling).clamp(0.0, 1.0))
    } else {
        // No production possible even fully upgraded: nothing to hold
        // against the player's upgrade state.
        1.0
    };

    BaseGrade::blend(utilization, infrastructure)
}

/// The optimizer's best sustained LMD-equivalent daily yield for this roster
/// on this building (cap-aware staffing, then the same sustained valuation
/// the utilization numerator uses).
fn best_yield(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let optimal =
        compute_sustained_assignment(profiles, building, building_data, registry, morale_drains);
    sustained_assignment_value(
        &optimal.main,
        profiles,
        building,
        building_data,
        registry,
        morale_drains,
    )
}

/// The same layout with every room upgraded to its max level (`phases` count
/// from the room catalogue). Composition is untouched - which slot hosts
/// which facility is the player's floor plan, not an upgrade.
fn max_level_building(building: &UserBuilding, building_data: &BuildingDataFile) -> UserBuilding {
    let mut maxed = building.clone();
    for room in &mut maxed.rooms {
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let max_level = building_data
            .rooms
            .get(&room.room_type)
            .map_or(room.level, |def| def.phases.len() as i32);
        room.level = room.level.max(max_level);
    }
    maxed
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
