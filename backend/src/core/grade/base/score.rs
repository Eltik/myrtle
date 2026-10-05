use crate::core::gamedata::types::{GameData, building::BuildingDataFile};
use crate::core::grade::base::assignment::{compute_live_assignment, sustained_assignment_value};
use crate::database::models::roster::RosterEntry;

use super::{
    buff_registry::{BuffResolutionStrategy, build_name_to_char, build_registry, faction_tags_of},
    types::{OperatorBaseProfile, UserBuilding},
};
use std::collections::HashMap;

/// Stationing utilization: synced stationing vs the optimizer's best staffing
/// of the same rooms. The dominant term.
const UTILIZATION_WEIGHT: f64 = 0.75;
/// Infrastructure completeness: built rooms vs the same rooms at max level.
/// Smaller on purpose: progress matters less than using what you have.
const INFRASTRUCTURE_WEIGHT: f64 = 0.25;

/// Base grade plus its two log-curved [0, 1] components. `score` is the blend
/// the profile stores; the components let the frontend show which one drags.
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

/// Does the player have the best base THEY could have? Two log-curved ratios of
/// sustained LMD-equivalent daily yield (morale rotation, factory buffer stalls,
/// gold->trade coupling), blended 75/25:
///
///   - UTILIZATION (75%): the synced stationing vs the optimizer's best staffing
///     of the same roster on the same rooms. Roster size and room levels cancel.
///   - INFRASTRUCTURE (25%): the rooms as built vs at max level (same layout and
///     roster; furniture ambience left as synced, it's decor, not a level).
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
        None => return BaseGrade::default(),
    };
    if user_building.is_empty() {
        return BaseGrade::default();
    }
    let profiles = build_operator_profiles(roster, game_data);

    // Achievable on the rooms as built (utilization denominator, infrastructure
    // numerator).
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

    // Preset-shift players are graded on the average of their shifts, not the one
    // on duty at sync: each shift is weaker than the sustained optimum on purpose
    // (one is the rest shift). One shift of a three-shift plan read 88% on 00980819
    // where the whole plan read 94%.
    let live_morale =
        building_json.map_or_else(HashMap::new, super::sustain_sim::synced_live_morale);
    let value_of = |shift: Option<usize>| {
        let stationed = compute_live_assignment(
            &profiles,
            &user_building,
            building_data,
            &registry,
            &morale_drains,
            shift,
            &live_morale,
        );
        sustained_assignment_value(
            &stationed,
            &profiles,
            &user_building,
            building_data,
            &registry,
            &morale_drains,
        )
    };
    // A preset shift counts only if every production room with presets has a crew
    // in it. An unset third preset is stored as `-1` seats and would read as dark
    // rooms (00980819: two posts and a factory dark in "shift 3", average fell to
    // 78% of a base running at 98%).
    let usable_shifts: Vec<usize> = {
        let with_presets: Vec<&super::types::UserRoom> = user_building
            .rooms
            .iter()
            .filter(|r| {
                super::util::is_production_room(&r.room_type) && !r.preset_shifts.is_empty()
            })
            .collect();
        let most = with_presets
            .iter()
            .map(|r| r.preset_shifts.len())
            .max()
            .unwrap_or(0);
        (0..most)
            .filter(|&i| {
                with_presets
                    .iter()
                    .all(|r| r.preset_shifts.get(i).is_some_and(|crew| !crew.is_empty()))
            })
            .collect()
    };
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
        &registry,
        &morale_drains,
    );
    let infrastructure = if ceiling > 0.0 {
        log_curve_ratio((achievable / ceiling).clamp(0.0, 1.0))
    } else {
        // Nothing producible even fully upgraded: nothing to hold against them.
        1.0
    };

    BaseGrade::blend(utilization, infrastructure)
}

/// Best sustained LMD-equivalent daily yield for this roster on this building.
fn best_yield(
    profiles: &[OperatorBaseProfile],
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    registry: &HashMap<String, BuffResolutionStrategy>,
    morale_drains: &HashMap<String, f64>,
) -> f64 {
    // The bar is the Optimizer tab's plan (bundle-searched optimal), valued
    // sustained, so copying it reads 100%. Against the cap-aware sustained selection
    // (3% higher on 00980819, never shown) a perfect copy read 96% and the player
    // asked why (2026-09-22). The bundle search is the expensive part of a grade (a
    // few seconds in release); it runs on the blocking pool.
    let economy = super::pools::search_economy(profiles, building, building_data, registry);
    let accepted = super::pools::optimal_with_bundles(
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

/// Every room at max level (`phases` count from the catalogue). Composition is
/// untouched: which slot hosts what is the floor plan, not an upgrade.
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
