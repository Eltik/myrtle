use crate::BaseData;
use crate::assignment::{compute_live_assignment, sustained_assignment_value};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;
use myrtle_types::roster::RosterEntry;

use super::{
    registry::{BuffResolutionStrategy, faction_tags_of},
    types::{OperatorBaseProfile, UserBuilding},
};
use std::collections::HashMap;

const UTILIZATION_WEIGHT: f64 = 0.75;
const INFRASTRUCTURE_WEIGHT: f64 = 0.25;

/// Base grade plus its two log-curved [0, 1] components
#[derive(Debug, Clone, Copy, Default)]
pub struct BaseGrade {
    pub score: f64,
    /// Actual vs achievable on the built rooms
    pub utilization: f64,
    /// Achievable as built vs at max level
    pub infrastructure: f64,
}

impl BaseGrade {
    pub(crate) fn blend(utilization: f64, infrastructure: f64) -> Self {
        Self {
            score: INFRASTRUCTURE_WEIGHT.mul_add(infrastructure, UTILIZATION_WEIGHT * utilization),
            utilization,
            infrastructure,
        }
    }
}

/// Two log-curved ratios of sustained LMD-equivalent daily yield, blended 75/25
pub fn grade_base(
    roster: &[RosterEntry],
    building_json: Option<&serde_json::Value>,
    game_data: &BaseData,
) -> BaseGrade {
    let building_data = game_data.building();
    if building_data.buffs.is_empty() {
        return BaseGrade::default();
    }
    let registry = game_data.registry().as_ref();
    let morale_drains = game_data.morale_drains().as_ref();

    let user_building = match building_json {
        Some(json) => UserBuilding::from_json(json),
        None => return BaseGrade::default(),
    };
    if user_building.is_empty() {
        return BaseGrade::default();
    }
    let profiles = build_operator_profiles(roster, game_data);

    let achievable = best_yield(
        &profiles,
        &user_building,
        building_data,
        registry,
        morale_drains,
    );
    if achievable <= 0.0 {
        return BaseGrade::default();
    }

    let live_morale = building_json.map_or_else(
        HashMap::new,
        crate::rotation::sustain_sim::synced_live_morale,
    );
    let value_of = |shift: Option<usize>| {
        let stationed = compute_live_assignment(
            &profiles,
            &user_building,
            building_data,
            registry,
            morale_drains,
            shift,
            &live_morale,
        );
        sustained_assignment_value(
            &stationed,
            &profiles,
            &user_building,
            building_data,
            registry,
            morale_drains,
        )
    };
    let usable_shifts = crate::rotation::value::usable_preset_shifts(&user_building);
    let actual = if usable_shifts.len() >= 2 {
        #[allow(clippy::cast_precision_loss)]
        let n = usable_shifts.len() as f64;
        usable_shifts
            .iter()
            .map(|&i| value_of(Some(i)))
            .sum::<f64>()
            / n
    } else {
        value_of(None)
    };
    let utilization = log_curve_ratio((actual / achievable).clamp(0.0, 1.0));

    let maxed_building = max_level_building(&user_building, building_data);
    let ceiling = best_yield(
        &profiles,
        &maxed_building,
        building_data,
        registry,
        morale_drains,
    );
    let infrastructure = if ceiling > 0.0 {
        log_curve_ratio((achievable / ceiling).clamp(0.0, 1.0))
    } else {
        1.0
    };

    BaseGrade::blend(utilization, infrastructure)
}

fn best_yield(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    let economy =
        crate::scoring::pools::search_economy(profiles, building, building_data, registry);
    let accepted = crate::scoring::pools::optimal_with_bundles(
        profiles,
        building,
        building_data,
        registry,
        &economy.registry,
        morale_drains,
        &economy.pins,
    );
    sustained_assignment_value(
        &accepted.optimal,
        profiles,
        building,
        building_data,
        registry,
        morale_drains,
    )
}

pub(crate) fn max_level_building(
    building: &UserBuilding,
    building_data: &BuildingDataFile,
) -> UserBuilding {
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
    game_data: &BaseData,
) -> Vec<OperatorBaseProfile> {
    let mut profiles = Vec::new();
    for entry in roster {
        let building_char = game_data.building().chars.get(&entry.operator_id);

        if let Some(bc) = building_char {
            let static_op = game_data.operators().get(&entry.operator_id);
            let faction_tags = static_op.map(faction_tags_of).unwrap_or_default();
            let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
            profiles.push(OperatorBaseProfile::build(
                entry,
                bc,
                faction_tags,
                rarity,
                game_data.building(),
                false,
            ));
        }
    }
    profiles
}

pub(crate) fn log_curve_ratio(t: f64) -> f64 {
    t.ln_1p() / 2.0_f64.ln()
}
