//! Helpers shared by the hand formulas. Each reads the unit the way the per-file
//! copies did, so moving a formula onto one changes no output bit.

use super::super::super::operator_unit::OperatorUnit;

/// `skill_table` `Duration` for a skill with unlimited duration (and for an ammo skill,
/// whose length the game data does not give in seconds). Return it from a hand
/// formula's `skill_duration` to report a skill as unlimited: the engine then sets
/// `total_damage` and `average_dps` to `skill_dps` and files it as Sustained.
pub const UNLIMITED_DURATION: f64 = -1.0;

/// A key from the trait part of the equipped module at its current level
/// (the `OverrideTraitDataBundle` of any part; the first candidate that
/// carries the key).
pub fn module_trait_value(unit: &OperatorUnit, key: &str) -> Option<f64> {
    let module = unit.operator_module.as_ref()?;
    module
        .data
        .phases
        .iter()
        .find(|phase| phase.equip_level == unit.module_level)?
        .parts
        .iter()
        .filter_map(|part| part.override_trait_data_bundle.candidates.as_ref())
        .flatten()
        .flat_map(|candidate| candidate.blackboard.iter())
        .find(|entry| entry.key == key)
        .map(|entry| entry.value)
}

/// A key from a talent part of the equipped module at its current level
/// (`AddOrOverrideTalentDataBundle`), picked by the part's `PrefabKey` (`"1"`, `"2"`
/// for the two talents, `"10_root"` and similar for hidden ones), at the highest
/// `RequiredPotentialRank` the unit's potential reaches.
///
/// `OperatorData` only keeps talent candidates it can file under talent 1 or 2 by
/// index or name; an unnamed hidden part (`TalentIndex` -1, no `Name`, e.g. Viy's
/// MSC-Y `10_root` ASPD) is dropped there, but the unit's own module data still holds
/// it, which is what this reads.
pub fn module_talent_value(unit: &OperatorUnit, prefab_key: &str, key: &str) -> Option<f64> {
    let module = unit.operator_module.as_ref()?;
    module
        .data
        .phases
        .iter()
        .find(|phase| phase.equip_level == unit.module_level)?
        .parts
        .iter()
        .filter_map(|part| part.add_or_override_talent_data_bundle.candidates.as_ref())
        .flatten()
        .filter(|candidate| {
            candidate.prefab_key.as_deref() == Some(prefab_key)
                && unit.potential > candidate.required_potential_rank
        })
        .max_by_key(|candidate| candidate.required_potential_rank)?
        .blackboard
        .iter()
        .find(|entry| entry.key == key)
        .map(|entry| entry.value)
}
