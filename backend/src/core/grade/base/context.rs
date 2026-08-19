//! The shared setup every base computation needs: the roster turned into
//! base-skill profiles, the buff registry, and the morale drain table.
//!
//! Both the read-only improvements plan and the interactive planner endpoints
//! start from exactly this state, so it lives in one place - a second copy
//! would be a second chance for the two to disagree about what a roster means.

use std::collections::HashMap;

use crate::core::gamedata::types::GameData;
use crate::core::grade::base::buff_registry::{
    BuffResolutionStrategy, build_name_to_char, build_registry, faction_tags_of,
};
use crate::core::grade::base::types::OperatorBaseProfile;
use crate::database::models::roster::RosterEntry;

/// Roster-derived inputs to the clause engine, built once per request.
pub struct BaseContext {
    /// One profile per roster operator that has a `building_data.chars` entry.
    pub profiles: Vec<OperatorBaseProfile>,
    pub registry: HashMap<String, BuffResolutionStrategy>,
    pub morale_drains: HashMap<String, f64>,
}

impl BaseContext {
    /// Roster -> base-skill profiles. Drops operators with no entry in
    /// `building_data.chars` (e.g. tokens, drones).
    /// `ignore_promotion` plans with every operator's highest base skills,
    /// whether or not the player has promoted them that far.
    pub fn build(roster: &[RosterEntry], game_data: &GameData, ignore_promotion: bool) -> Self {
        let profiles: Vec<OperatorBaseProfile> = roster
            .iter()
            .filter_map(|entry| {
                let bc = game_data.building.chars.get(&entry.operator_id)?;
                let static_op = game_data.operators.get(&entry.operator_id);
                let faction_tags = static_op.map(faction_tags_of).unwrap_or_default();
                let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
                Some(OperatorBaseProfile::build(
                    entry,
                    bc,
                    faction_tags,
                    rarity,
                    &game_data.building,
                    ignore_promotion,
                ))
            })
            .collect();

        let name_to_char = build_name_to_char(&game_data.operators);
        let (registry, morale_drains) = build_registry(&game_data.building.buffs, &name_to_char);

        Self {
            profiles,
            registry,
            morale_drains,
        }
    }

    /// The subset of `profiles` the planner may seat, given an exclusion list.
    /// Excluding is cheaper than re-deriving profiles, and keeps the candidate
    /// pool the single thing a caller has to reason about.
    pub fn profiles_excluding(&self, excluded: &[String]) -> Vec<OperatorBaseProfile> {
        if excluded.is_empty() {
            return self.profiles.clone();
        }
        let drop: std::collections::HashSet<&str> = excluded.iter().map(String::as_str).collect();
        self.profiles
            .iter()
            .filter(|p| !drop.contains(p.char_id.as_str()))
            .cloned()
            .collect()
    }
}
