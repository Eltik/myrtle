//! Helpers shared by the hand formulas.

use super::super::super::operator_unit::OperatorUnit;

/// `skill_table` `Duration` of an unlimited skill; the engine then files it as
/// Sustained with `total_damage` and `average_dps` equal to `skill_dps`.
pub const UNLIMITED_DURATION: f64 = -1.0;

/// A key from the equipped module's trait blackboard at its current level.
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

/// A key from the equipped module's talent part `prefab_key` (`"1"`, `"2"`, hidden
/// `"10_root"`) at the unit's potential. Read from the module itself because
/// `OperatorData` drops unnamed hidden parts.
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
