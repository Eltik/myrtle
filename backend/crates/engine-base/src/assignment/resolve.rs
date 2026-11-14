use std::collections::{HashMap, HashSet};

use crate::{
    registry::BuffResolutionStrategy,
    types::{OperatorBaseProfile, UserBuilding},
};
use myrtle_gamedata_raw::tables::building::BuildingDataFile;

/// True when the roster owns an operator whose base-wide conditional could fire
pub fn base_wide_relevant(
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    let roster: HashSet<&str> = operators.iter().map(|o| o.char_id.as_str()).collect();
    operators.iter().any(|op| {
        op.available_buffs.iter().any(|b| {
            matches!(
                registry.get(b),
                Some(BuffResolutionStrategy::ConditionalOnBaseWide { required_char_ids, .. })
                    if required_char_ids.iter().any(|id| roster.contains(id.as_str()))
            )
        })
    })
}

/// A copy of `registry` with every base-wide conditional collapsed to a flat `DirectEfficiency`
#[must_use]
pub fn resolve_base_wide(
    registry: &HashMap<String, BuffResolutionStrategy>,
    working: &HashSet<String>,
    stationed_anywhere: &HashSet<String>,
) -> HashMap<String, BuffResolutionStrategy> {
    registry
        .iter()
        .map(|(id, strategy)| {
            let resolved = match strategy {
                BuffResolutionStrategy::ConditionalOnBaseWide {
                    required_char_ids,
                    base_efficiency,
                    bonus_efficiency,
                    anywhere,
                } => {
                    let present = if *anywhere {
                        stationed_anywhere
                    } else {
                        working
                    };
                    let bonus = if required_char_ids.iter().any(|c| present.contains(c)) {
                        *bonus_efficiency
                    } else {
                        0.0
                    };
                    BuffResolutionStrategy::DirectEfficiency {
                        value: base_efficiency + bonus,
                    }
                }
                other => other.clone(),
            };
            (id.clone(), resolved)
        })
        .collect()
}

pub fn dorm_residents(building: &UserBuilding) -> impl Iterator<Item = String> + '_ {
    building
        .rooms
        .iter()
        .filter(|r| r.room_type == "DORMITORY")
        .flat_map(|r| r.current_operators.iter().cloned())
}

/// True when the roster owns a room-presence gate that could fire
pub fn room_presence_relevant(
    operators: &[OperatorBaseProfile],
    registry: &HashMap<String, BuffResolutionStrategy>,
) -> bool {
    let roster: HashSet<&str> = operators.iter().map(|o| o.char_id.as_str()).collect();
    operators.iter().any(|op| {
        op.available_buffs.iter().any(|b| match registry.get(b) {
            Some(BuffResolutionStrategy::RoomPresenceGatedGlobal {
                required_faction,
                required_count,
                bonus_pct,
                ..
            }) if *bonus_pct != 0.0 => {
                operators
                    .iter()
                    .filter(|o| o.faction_tags.iter().any(|t| t == required_faction))
                    .count()
                    >= *required_count
            }
            Some(BuffResolutionStrategy::FacilityCountModifier { .. }) => {
                operators.iter().any(|o| {
                    o.available_buffs.iter().any(|b| {
                        matches!(
                            registry.get(b),
                            Some(BuffResolutionStrategy::FacilityCountScaling { .. })
                        )
                    })
                })
            }
            Some(BuffResolutionStrategy::BaseWideMatchCountScaling { token, .. }) => operators
                .iter()
                .any(|o| o.match_tags.iter().any(|t| t == token)),
            Some(BuffResolutionStrategy::NamedTargetRoomBoost { target_char_id, .. }) => {
                roster.contains(target_char_id.as_str())
            }
            Some(BuffResolutionStrategy::ConditionalOnRoomPresence {
                required_char_ids,
                required_faction,
                required_count,
                bonus_efficiency,
                ..
            }) if *bonus_efficiency != 0.0 => {
                required_char_ids
                    .iter()
                    .any(|id| roster.contains(id.as_str()))
                    || required_faction.as_ref().is_some_and(|tag| {
                        operators
                            .iter()
                            .filter(|o| o.faction_tags.iter().any(|t| t == tag))
                            .count()
                            >= *required_count
                    })
            }
            _ => false,
        })
    })
}

/// A copy of `registry` with every room-presence gate collapsed to a flat `DirectEfficiency`
#[must_use]
pub fn resolve_room_presence(
    registry: &HashMap<String, BuffResolutionStrategy>,
    deployed_rooms: &HashMap<String, String>,
    operators: &[OperatorBaseProfile],
) -> HashMap<String, BuffResolutionStrategy> {
    let faction_tags: HashMap<&str, &Vec<String>> = operators
        .iter()
        .map(|o| (o.char_id.as_str(), &o.faction_tags))
        .collect();
    let match_tags: HashMap<&str, &Vec<String>> = operators
        .iter()
        .map(|o| (o.char_id.as_str(), &o.match_tags))
        .collect();
    registry
        .iter()
        .map(|(id, strategy)| {
            let resolved = match strategy {
                BuffResolutionStrategy::ConditionalOnRoomPresence {
                    required_char_ids,
                    required_faction,
                    required_count,
                    room_type,
                    base_efficiency,
                    bonus_efficiency,
                } => {
                    let named_ok = required_char_ids
                        .iter()
                        .any(|c| deployed_rooms.get(c).is_some_and(|rt| rt == room_type));
                    let faction_ok = required_faction.as_ref().is_some_and(|tag| {
                        deployed_rooms
                            .iter()
                            .filter(|(char_id, rt)| {
                                *rt == room_type
                                    && faction_tags
                                        .get(char_id.as_str())
                                        .is_some_and(|tags| tags.iter().any(|t| t == tag))
                            })
                            .count()
                            >= *required_count
                    });
                    let bonus = if named_ok || faction_ok {
                        *bonus_efficiency
                    } else {
                        0.0
                    };
                    BuffResolutionStrategy::DirectEfficiency {
                        value: base_efficiency + bonus,
                    }
                }
                BuffResolutionStrategy::RoomPresenceGatedGlobal {
                    required_faction,
                    required_count,
                    room_type,
                    target_room,
                    bonus_pct,
                } => {
                    let seated = deployed_rooms
                        .iter()
                        .filter(|(char_id, rt)| {
                            *rt == room_type
                                && faction_tags
                                    .get(char_id.as_str())
                                    .is_some_and(|tags| tags.iter().any(|t| t == required_faction))
                        })
                        .count();
                    if seated >= *required_count {
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: target_room.clone(),
                            bonus_pct: *bonus_pct,
                        }
                    } else {
                        strategy.clone()
                    }
                }
                BuffResolutionStrategy::BaseWideMatchCountScaling {
                    token,
                    per_match_pct,
                    cap_count,
                } => {
                    let n = if deployed_rooms.is_empty() {
                        operators
                            .iter()
                            .filter(|o| o.match_tags.iter().any(|t| t == token))
                            .count()
                    } else {
                        deployed_rooms
                            .keys()
                            .filter(|id| {
                                match_tags
                                    .get(id.as_str())
                                    .is_some_and(|tags| tags.iter().any(|t| t == token))
                            })
                            .count()
                    };
                    #[allow(clippy::cast_precision_loss)]
                    let n = cap_count.map_or(n, |cap| n.min(cap)) as f64;
                    BuffResolutionStrategy::DirectEfficiency {
                        value: n * per_match_pct,
                    }
                }
                BuffResolutionStrategy::NamedTargetRoomBoost {
                    self_room,
                    target_char_id,
                    target_room,
                    bonus_pct,
                    ..
                } => {
                    let holder_seated = operators
                        .iter()
                        .filter(|o| o.available_buffs.iter().any(|b| b == id))
                        .any(|o| {
                            deployed_rooms
                                .get(&o.char_id)
                                .is_some_and(|rt| rt == self_room)
                        });
                    BuffResolutionStrategy::NamedTargetRoomBoost {
                        self_room: self_room.clone(),
                        target_char_id: target_char_id.clone(),
                        target_room: target_room.clone(),
                        bonus_pct: *bonus_pct,
                        active: holder_seated,
                    }
                }
                other => other.clone(),
            };
            (id.clone(), resolved)
        })
        .collect()
}

/// A copy of `registry` with every layout-counted branch collapsed to a plain `GlobalEffect`
#[must_use]
pub fn resolve_layout_branches(
    registry: &HashMap<String, BuffResolutionStrategy>,
    building: &UserBuilding,
    building_data: &BuildingDataFile,
    facility_counts: &HashMap<String, usize>,
) -> HashMap<String, BuffResolutionStrategy> {
    let count_of = |term: &str| -> usize {
        building_data.layout_terms.get(term).map_or(0, |rooms| {
            rooms
                .iter()
                .map(|rt| {
                    facility_counts.get(rt).copied().unwrap_or_else(|| {
                        building.rooms.iter().filter(|r| &r.room_type == rt).count()
                    })
                })
                .sum()
        })
    };
    registry
        .iter()
        .map(|(id, strategy)| {
            let resolved = match strategy {
                BuffResolutionStrategy::LayoutCountBranch {
                    a_term,
                    b_term,
                    ge_room,
                    ge_pct,
                    gt_room,
                    gt_pct,
                } if building_data.layout_terms.contains_key(a_term)
                    && building_data.layout_terms.contains_key(b_term) =>
                {
                    let (a, b) = (count_of(a_term), count_of(b_term));
                    if a >= b {
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: ge_room.clone(),
                            bonus_pct: *ge_pct,
                        }
                    } else {
                        BuffResolutionStrategy::GlobalEffect {
                            target_room: gt_room.clone(),
                            bonus_pct: *gt_pct,
                        }
                    }
                }
                other => other.clone(),
            };
            (id.clone(), resolved)
        })
        .collect()
}

/// A copy of `registry` with every pool-scaled CC global collapsed to a flat `GlobalEffect`
#[must_use]
pub fn resolve_global_pool(
    registry: &HashMap<String, BuffResolutionStrategy>,
    points: &HashMap<String, f64>,
) -> HashMap<String, BuffResolutionStrategy> {
    registry
        .iter()
        .map(|(id, strategy)| {
            let resolved = match strategy {
                BuffResolutionStrategy::GlobalPoolScaling {
                    target_room,
                    base_pct,
                    pct,
                    per,
                    resource,
                } if *per > 0.0 => {
                    let p = points.get(resource).copied().unwrap_or(0.0);
                    BuffResolutionStrategy::GlobalEffect {
                        target_room: target_room.clone(),
                        bonus_pct: base_pct + (p / per).floor() * pct,
                    }
                }
                other => other.clone(),
            };
            (id.clone(), resolved)
        })
        .collect()
}
