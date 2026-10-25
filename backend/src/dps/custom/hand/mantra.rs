//! Mantra (`char_4204_mantra`), Caster / Primal, base attack time 1.6 s, Arts.
//! Hand-written: `ArknightsDpsCompare` has no class for her. Written from the EN game
//! data (`character_table`, `skill_table`, `battle_equip_table`), checked 2026-10-25,
//! in the style of the upstream Nervous Impairment formulas (Tragodia, Gracebearer).
//!
//! Game data behind each number:
//! - Trait: Arts attacks that can inflict Elemental Damage (no blackboard).
//! - Module PRI-X (`uniequip_002_mantra`): ATK +43 / +69 / +87 (attribute blackboard,
//!   applied by the engine); trait override `damage_scale` 1.1, "damage dealt to enemies
//!   under an Elemental Burst effect increased to 110%", read from the module's trait
//!   blackboard (`util::module_trait_value`). Stage 2 / 3 raise talent 1 (155% / 10%,
//!   160% / 15% at P1) through the engine's talent resolution.
//! - Talent 1 "Quiet Zone" (E1), `talent1_parameters`: `[0]` `atk_scale` (E2 1.35),
//!   `[1]` `prob` (E2 0.10). "Enemies take 135% of Mantra's ATK as Elemental Damage
//!   when they trigger Paralysis; ... a 10% chance to not consume a Paralysis stack".
//! - `gamedata_const` `ba.dt.neural2` (Nervous Impairment - Ally): the burst deals 6000
//!   Elemental damage "and inflict[s] 3 stacks of Paralysis"; `ba.palsy`: "Each stack of
//!   Paralysis can interrupt an enemy when it attempts to perform a normal attack".
//! - S1 `skchr_mantra_1` (2 SP, gained per attack), `skill_parameters`: `[0]`
//!   `atk_scale` 3.25, `[1]` `ep_damage_ratio` 0.30, `[2]` `element_atk_scale` 2.0.
//! - S2 `skchr_mantra_2` (25 s): `[0]` `base_attack_time` -0.8, `[1]` `attack@atk_scale`
//!   2.4, `[2]` `attack@chain.atk_scale` 1.2, `[3]` `attack@ep_damage_ratio` 0.18,
//!   `[4]` `attack@times` 4, `[5]` `chain_times` 3, `[6]` `attack@element_atk_scale` 0.25.
//! - S3 `skchr_mantra_3` (40 s): `[0]` `atk` 2.75, `[1]` `attack@max_target` 2, `[2]`
//!   `atk_scale` 1.85, `[3]` `max_target` 2, `[4]` `chain_times` 1, `[5]`
//!   `max_trigger_cnt` 3, `[6]` `interval_projectile_trigger` 1.5, `[7]` `per_active` 2.
//!
//! Conditionals:
//! - skill `avgNerv` (ON, S1 and S2): average over the Nervous Impairment cycle: the
//!   gauge fills from the skill's Nervous Impairment, then a 10 s burst deals 6000
//!   Elemental damage, her hits add `element_atk_scale` ATK as Elemental damage, and
//!   (module PRI-X) her damage is x1.1. OFF: no Nervous Impairment.
//! - trait `vsBoss` (OFF = the label): the gauge is 2000 instead of 1000.
//! - talent `targetAttacks` (ON, with `avgNerv`, E1): the target attacks often enough to
//!   spend every Paralysis stack of each burst before the next one. Each burst's 3
//!   stacks are each spent on an interrupted attack and kept with chance `prob`, so a
//!   burst yields 3 / (1 - `prob`) interrupts on average, each dealing talent 1's
//!   `atk_scale` x ATK Elemental damage. OFF: the target never attacks (no interrupts).
//!   Sources: PRTS https://prts.wiki/w/真言 (the talent triggers when Paralysis
//!   interrupts an attack, not when the stack is gained; a burst's "3 stacks" are three
//!   1-stack gains), arknights.wiki.gg/wiki/Elemental_damage (Elemental damage ignores
//!   DEF and RES).
//!
//! ASSUMPTIONS and simplifications:
//! - Nervous Impairment burst constants are the upstream ones (Tragodia, Gracebearer):
//!   6000 Elemental damage over a 10 s burst, gauge 1000 (2000 vs a boss), no
//!   accumulation during the burst, and no reduction of Elemental damage by RES.
//! - "x% of damage as Nervous Impairment" is x% of the Arts damage dealt (after RES).
//! - S1 fires every third attack (2 SP per attack, cost + 1 attacks a cycle, as
//!   Tragodia S1); SP boost does not apply to an attack-charged skill.
//! - S2 jumps hit min(targets - 1, `chain_times`) other enemies; the burst cycle is
//!   modelled on the primary target only, so jump damage gets no burst bonus.
//! - The 1.1 module bonus applies to Mantra's own damage during the burst, not to the
//!   6000 burst damage itself.
//! - Paralysis: only the stacks of her own bursts (S1, S2) are modelled, on the primary
//!   target, at her ATK of that skill, without the module's x1.1 (the interrupts are
//!   not tied to the burst window). Stacks from Total Grasp spawns and from allies' and
//!   S3's skill activations, and S3's Elemental jumps per Paralysis stack above its
//!   limit of 2, depend on things the engine cannot see: S3 is its ATK +275% two-target
//!   attacks only, S0 plain attacks.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

/// Upstream Nervous Impairment burst: total damage and duration (Tragodia).
const NERV_BURST_DAMAGE: f64 = 6000.0;
const NERV_BURST_DURATION: f64 = 10.0;
/// Paralysis stacks a Nervous Impairment burst inflicts (`ba.dt.neural2`).
const NERV_BURST_PARALYSIS: f64 = 3.0;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let res = enemy.res;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);

    let mut atk_interval = f64::from(unit.attack_interval);
    let atkbuff = if skill == 3 { sp(0, 0.0) } else { 0.0 };
    if skill == 2 {
        atk_interval += sp(0, 0.0);
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;
    let hitdmg = arts(1.0);

    // Primary-target DPS, its Nervous Impairment per second, the Elemental damage per
    // second her hits add during a burst, and DPS outside the burst model.
    let (primary_dps, nerv_rate, burst_ele_dps, other_dps) = match skill {
        1 => {
            let cycle = f64::from(unit.skill_cost) + 1.0;
            let skilldmg = arts(sp(0, 1.0));
            let avg = (skilldmg + hitdmg * (cycle - 1.0)) / cycle;
            (
                avg * attack_rate,
                skilldmg * sp(1, 0.0) * attack_rate / cycle,
                final_atk * sp(2, 0.0) * attack_rate / cycle,
                0.0,
            )
        }
        2 => {
            let main = arts(sp(1, 1.0));
            let jumps = (f64::from(unit.targets) - 1.0).clamp(0.0, sp(5, 0.0));
            (
                main * attack_rate,
                main * sp(3, 0.0) * attack_rate,
                final_atk * sp(6, 0.0) * attack_rate,
                arts(sp(2, 0.0)) * jumps * attack_rate,
            )
        }
        3 => (
            hitdmg * attack_rate * f64::from(unit.targets).min(sp(1, 1.0)),
            0.0,
            0.0,
            0.0,
        ),
        _ => (hitdmg * attack_rate, 0.0, 0.0, 0.0),
    };

    let mut dps = primary_dps;
    if unit.skill_damage && nerv_rate > 0.0 {
        let gauge = if unit.trait_damage { 1000.0 } else { 2000.0 };
        // Module PRI-X trait override `damage_scale`: damage to enemies under a burst.
        let burst_scale = if unit.module_index == 1 {
            module_trait_value(unit, "damage_scale").unwrap_or(1.0)
        } else {
            1.0
        };
        let time_to_burst = gauge / nerv_rate;
        let burst_dps = (primary_dps + burst_ele_dps) * burst_scale;
        // Talent 1: Elemental damage per Paralysis interrupt, not reduced by RES.
        let paralysis = if unit.talent_damage && unit.elite > 0 {
            let t1 = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
            NERV_BURST_PARALYSIS / (1.0 - t1(1)) * t1(0) * final_atk
        } else {
            0.0
        };
        dps = (primary_dps * time_to_burst
            + burst_dps * NERV_BURST_DURATION
            + NERV_BURST_DAMAGE
            + paralysis)
            / (time_to_burst + NERV_BURST_DURATION);
    }

    Some(dps + other_dps)
}
