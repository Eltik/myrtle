//! Manticore (`char_215_mantic`), Specialist / Ambusher, base attack time 3.5 s.
//! Hand-written over the transpiled formula, which counts her module's talent upgrade
//! twice.
//!
//! The reference (`damage_formulas.py`, `class Manticore`) adds `0.05 * (module_lvl - 1)`
//! to talent 1's ATK by hand, because its `JsonReader` files only `prefabKey` 1 / 2
//! candidates and skips the module's `TALENT_DATA_ONLY` override. `OperatorUnit` applies
//! that override, so `talent1_parameters` already holds the module value, and the
//! transpiled `+= 0.05 * (module_level - 1)` counted it again.
//!
//! Game data (EN `battle_equip_table` `uniequip_002_mantic`, read 2026-11-05): talent 1
//! (`delay`, `atk`) is overridden at stage 2 to `atk` 0.55 (P5 0.59) and at stage 3 to
//! 0.60 (P5 0.64), from the base 0.50. With the double count, S2 at module stage 3 read
//! ATK +0.70 and ran 4.0% high at DEF 0 (478.00 against the reference's 459.62), up to
//! 6.9% at high DEF.
//!
//! Everything else is the transpiled formula unchanged: S2 fixes the attack interval at
//! 5.2 s, talent 1's ATK applies only when the interval (over ASPD) is at least its
//! `delay`, and S2 adds `skill_parameters[1]` ATK.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

// The registry signature is `DpsFn`, which returns an Option.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = if skill == 2 {
        5.2
    } else {
        f64::from(unit.attack_interval)
    };
    let mut atkbuff_talent = if unit.elite > 0 {
        unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
    } else {
        0.0
    };
    if unit.elite > 0
        && atk_interval / unit.attack_speed * 100.0
            < unit.talent1_parameters.first().copied().unwrap_or(0.0)
    {
        atkbuff_talent = 0.0;
    }
    let atkbuff = if skill == 2 {
        unit.skill_parameters.get(1).copied().unwrap_or(0.0)
    } else {
        0.0
    };
    let final_atk =
        unit.atk.mul_add(1.0 + atkbuff + atkbuff_talent + unit.buff_atk, unit.buff_atk_flat);
    let hitdmg = (final_atk - defense).max(final_atk * 0.05);
    Some(hitdmg / atk_interval * unit.attack_speed / 100.0 * f64::from(unit.targets))
}
