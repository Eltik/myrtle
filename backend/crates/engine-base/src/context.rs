//! Roster profiles beside the data version's buff registry and morale drains

use std::{collections::HashMap, sync::Arc};

use crate::BaseData;
use crate::registry::{BuffResolutionStrategy, faction_tags_for};
use crate::types::OperatorBaseProfile;
use myrtle_types::roster::RosterEntry;

/// Roster-derived inputs to the clause engine
pub struct BaseContext {
    pub profiles: Vec<OperatorBaseProfile>,
    pub registry: Arc<HashMap<String, BuffResolutionStrategy>>,
    pub morale_drains: Arc<HashMap<String, f64>>,
}

impl BaseContext {
    #[must_use]
    pub fn build(roster: &[RosterEntry], game_data: &BaseData, ignore_promotion: bool) -> Self {
        let glossary = game_data.glossary();
        let profiles: Vec<OperatorBaseProfile> = roster
            .iter()
            .filter_map(|entry| {
                let bc = game_data.building().chars.get(&entry.operator_id)?;
                let static_op = game_data.operators().get(&entry.operator_id);
                let faction_tags = static_op
                    .map(|op| faction_tags_for(&entry.operator_id, op, glossary))
                    .unwrap_or_default();
                let rarity = static_op.map_or(0, |o| o.rarity.to_star_int());
                Some(OperatorBaseProfile::build(
                    entry,
                    bc,
                    faction_tags,
                    rarity,
                    game_data.building(),
                    ignore_promotion,
                ))
            })
            .collect();

        Self {
            profiles,
            registry: Arc::clone(game_data.registry()),
            morale_drains: Arc::clone(game_data.morale_drains()),
        }
    }

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
