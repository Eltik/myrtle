//! Ch'en the Dawnstreak (`char_1050_chen3`), Guard / Arts Fighter (`artsfghter`), base
//! interval 1.25 s, blocks 1, deals Arts damage (trait). Hand-written:
//! `ArknightsDpsCompare` has no class for her. Written from the EN game data
//! (`character_table`, `skill_table`, `battle_equip_table`, `gamedata_const`), checked
//! 2026-10-25.
//!
//! `skill_parameters` are the raw blackboard order:
//! - S1 `skchr_chen3_1` (18 s): `[0]` `atk` 1.2; "attacks hit twice".
//! - S2 `skchr_chen3_2` (6 s): `[0]` `atk_scale` 4.8 (10 slashes, Arts, nearest enemy),
//!   `[1]` `chen3_s2[respawn_buff].atk` 3.0 (ATK +300% afterward).
//! - S3 `skchr_chen3_3` (20 s): `[1]` `projectile_min_atk_scale` 5.8 (sword wave floor),
//!   `[2]` `attack@atk_scale` 2.1 (3 instances per attack), `[3]` `attack@max_target` 4.
//!
//! `talent1_parameters` (All-Seeing Eye, E1): `[0]` `atk` 0.13, `[1]` `attack_speed` 13
//! (E2, potential 1), and "attacks deal Weakness damage". `gamedata_const`
//! `ba.weaknessatk`: "When dealing Physical or Arts damage, changes damage type
//! depending on the target's DEF and RES".
//! Talent 2 (To Know Time's Flow) is a self-heal and a Dodge: not modelled (no HPS
//! formula; self-heal-only operators have none in the generated table either).
//!
//! Module 1 (`uniequip_002_chen3`, AFT-X): trait `attack_speed` +8 "when not blocking
//! an enemy", read from the module's trait blackboard and gated by the `NotBlocking`
//! module conditional (Vina Victoria's label). Its talent part only changes talent 2.
//!
//! ASSUMPTIONS:
//! - Weakness damage takes whichever of Physical (ATK - DEF) and Arts (ATK x (1 -
//!   RES)) deals more. This is the game's rule, not an upper bound: PRTS
//!   (https://prts.wiki/w/赤刃明霄陈 talent notes, and 术语释义 弱点伤害) says each hit
//!   computes a theoretical Physical and Arts damage from ATK, the ATK multiplier,
//!   DEF / RES and their penetration only, and takes the higher type (a tie keeps the
//!   original Arts); damage multipliers then apply to the chosen type. The engine's
//!   multipliers are type-blind, so `max` is that rule. Whether the 5% floor enters the
//!   comparison is not stated; it changes nothing unless both values are below it. It
//!   applies to her attacks (S0, S1, S3's attack instances, S2's
//!   attacks after the slashes); S2's slashes and S3's sword wave, which the text calls
//!   Arts damage and are not attacks, stay Arts. Below E1 the talent is locked: Arts.
//! - S2: the 10 slashes all hit one target that survives them (no retarget, no extra
//!   slashes); the slash sequence takes no time, and the 6 s skill duration is the
//!   ATK +300% window. skill_dps = slashes / 6 s + buffed attack DPS, so total_damage is
//!   the slashes plus 6 s of buffed attacks.
//! - S3: the sword wave deals its floor, 580% ATK (the 6%-of-current-HP part is not
//!   modelled), to every target once per activation, spread over the 20 s duration.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

/// Slashes in S2 ("Slashes the nearest enemy 10 times").
const S2_SLASHES: f64 = 10.0;
/// Damage instances per S3 attack ("each attack deals 3 instances").
const S3_INSTANCES: f64 = 3.0;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let res = enemy.res;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);

    let weakness = unit.elite > 0;
    let (mut atkbuff, mut aspd) = if unit.elite > 0 {
        (
            unit.talent1_parameters.first().copied().unwrap_or(0.0),
            unit.talent1_parameters.get(1).copied().unwrap_or(0.0),
        )
    } else {
        (0.0, 0.0)
    };
    if unit.module_index == 1 && unit.module_damage {
        aspd += module_trait_value(unit, "attack_speed").unwrap_or(0.0);
    }

    let arts = |dmg: f64| (dmg * (1.0 - res / 100.0)).max(dmg * 0.05);
    // One attack hit of `dmg` raw damage: Weakness once the talent is unlocked.
    let attack_hit = |dmg: f64| {
        if weakness {
            arts(dmg).max((dmg - defense).max(dmg * 0.05))
        } else {
            arts(dmg)
        }
    };
    let duration = unit.skill_duration;

    let dps = match skill {
        1 => {
            atkbuff += param(0);
            let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
            2.0 * attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
        2 => {
            let slash_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
            let slashes = S2_SLASHES * arts(slash_atk * param(0));
            let final_atk =
                unit.atk * (1.0 + atkbuff + param(1) + unit.buff_atk) + unit.buff_atk_flat;
            let attacks = attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0;
            if duration > 0.0 {
                attacks + slashes / duration
            } else {
                attacks
            }
        }
        3 => {
            let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
            let targets = f64::from(unit.targets);
            let attacks = S3_INSTANCES * attack_hit(final_atk * param(2)) / atk_interval
                * (unit.attack_speed + aspd)
                / 100.0
                * targets.min(param(3));
            let wave = arts(final_atk * param(1)) * targets;
            if duration > 0.0 {
                attacks + wave / duration
            } else {
                attacks
            }
        }
        _ => {
            let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
            attack_hit(final_atk) / atk_interval * (unit.attack_speed + aspd) / 100.0
        }
    };

    Some(dps)
}
