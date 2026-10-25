//! Nasti (`char_4212_nasti`), Supporter / Artificer, melee physical, base attack
//! time 1.5 s, Block 2 at every promotion. Hand-written: `ArknightsDpsCompare` has no
//! class for her.
//!
//! Game data (EN `character_table` / `skill_table` / `battle_equip_table`, read
//! 2026-10-25):
//!
//! - Trait (`craftsman`): blocks 2, uses Support Devices. Normal attacks hit the one
//!   target, physical.
//! - Talent 1 "Construction Ahead" (`cnt`, device count) and talent 2 "Safety First"
//!   (`nasti_t2[res].damage_resistance`, `[res_plus]`, `[add_sp].interval`: Sanctuary
//!   and 1 SP every 6 s) change no damage number. Potential 3 (ATK +25) and module
//!   `uniequip_002_nasti` ATK (+26 / +32 / +40) are applied by `OperatorUnit`; the
//!   module's talent parts only change device count and cost.
//! - S1 `skchr_nasti_1`: `[0]` `talent@def` (device), `[1]` `atk` 0.8, `[2]` `def`;
//!   "simultaneously attacks all blocked enemies".
//! - S2 `skchr_nasti_2`: barrier only; "Stops attacking".
//! - S3 `skchr_nasti_3`: `[0]` `atk` 1.6, `[1]` `def`.
//!
//! ASSUMPTIONS and simplifications:
//! - S1 hits `min(targets, 2)`: every target is assumed blocked by her (Block 2 from
//!   `character_table` `BlockCnt`; the device's +1 Block goes to the operator in front).
//!   PRTS (https://prts.wiki/w/娜斯提): device effects are an aura on the units in the
//!   device's range, so she could block 3 inside her own device's range; not modelled.
//!   The device has 0 ATK (https://prts.wiki/w/质检专员).
//! - S2 deals no damage for its duration (0).
//! - Devices are not modelled (they deal no damage), nor is the talent 2 SP gain,
//!   which depends on a ranged enemy having appeared.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// `character_table` `BlockCnt` for `char_4212_nasti` (2 at E0, E1 and E2).
const BLOCK: f64 = 2.0;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    if skill == 2 {
        return Some(0.0);
    }

    let mut atkbuff: f64 = 0.0;
    let mut targets: f64 = 1.0;
    if skill == 1 {
        atkbuff = sp(1);
        targets = f64::from(unit.targets).min(BLOCK);
    }
    if skill == 3 {
        atkbuff = sp(0);
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    let hitdmg = (final_atk - defense).max(final_atk * 0.05);
    Some(hitdmg / atk_interval * unit.attack_speed / 100.0 * targets)
}
