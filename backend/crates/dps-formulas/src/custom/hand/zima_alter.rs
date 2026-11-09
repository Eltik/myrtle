//! Zima the Raging Tide (`char_1051_headb2`), Guard / Earthshaker (`hammer`), base
//! interval 1.8 s, blocks 2. Hand-written: `ArknightsDpsCompare` has no class for her.
//! Written from the EN game data (`character_table`, `skill_table`,
//! `battle_equip_table`), checked 2026-10-25.
//!
//! Trait: "Attacks deal 50% ATK as AOE Physical damage to enemies around the target".
//! Modelled as Odda and Pepe are: the target takes the full hit, each of the other
//! `targets - 1` enemies takes the 50% splash; skill multipliers scale both.
//!
//! `skill_parameters` are the raw blackboard order:
//! - S1 `skchr_headb2_1` (30 s): `[0]` `atk` 0.5, `[1]` `attack_speed` 50.
//! - S2 `skchr_headb2_2` (16 s, auto): `[0]` `atk` 0.9, `[2]` `headb2_s_2[second].atk`
//!   1.8 (second activation onward, unlimited duration).
//! - S3 `skchr_headb2_3` (instant, manual): `[0]` `atk_scale` 2.4, `[1]` `atk_step` 0.3,
//!   `[4]` `atk_base` 1.0. Five blows.
//!
//! `talent1_parameters` (Roiling Rage, E1): `[0]` `damage_scale` 1.24 (splash Physical
//! damage x1.24; the module raises it to 1.32 / 1.40 at levels 2 / 3 through the
//! engine's talent resolution).
//! `talent2_parameters` (Human Tide, E2): `[0]` `atk` 0.14 (0.18 from potential 3),
//! `[2]` `scale_bonus` 2.0 for Ursus Student Self-Government Group operators.
//! Zima the Raging Tide is in that group herself (`TeamId` `student`), so she gains
//! `atk * scale_bonus` (+28%) while any of her skills is active.
//!
//! Module 1 (`uniequip_002_headb2`, HAM-X): trait `atk_scale_e` 1.15 ("when there are
//! 3 or more enemies within the splash area, ATK of the current instance of attack
//! increases to 115%"), read from the module's trait blackboard and gated by the
//! `3inRange` module conditional, as Odda's and Pepe's are.
//!
//! ASSUMPTIONS:
//! - Talent 1's high-ground effect (`attack@splash_atk_scale` 24% ATK to enemies next
//!   to every high-ground tile the splash touches), S3's amplification of it
//!   (`splash_atk_scale_bonus` 3.5) and S2's SP gain per trigger are NOT modelled: they
//!   depend on the map's tiles.
//! - S2: `2ndActivation` ON is the second activation onward (ATK +180%), OFF the first
//!   (ATK +90%). The first activation keeps its 16 s game-data duration; from the
//!   second on the skill is unlimited ("this skill has unlimited duration"), so
//!   `skill_duration` reports it as unlimited and the engine sets `total_damage` and
//!   `average_dps` to `skill_dps`.
//! - S3 returns the TOTAL damage of the five blows (Degenbrecher's and Ch'en's S3
//!   convention, "totalDMG"): the skill has no duration, so the engine reports that
//!   total as `skill_dps`, `total_damage` and `average_dps` alike. Blow k (k = 0..4) has
//!   ATK +(100% + 30% * k): the +30% is read as granted after each blow, so the first
//!   blow has none of it. A third-party test supports this reading: blows of 5533 /
//!   6538 / 7542 / 8547 / 9551 on a 2100 DEF target fit 240% ATK at ATK x(2.28 +
//!   0.30 k), i.e. +100% + 28% (talent 2) + 30% per EARLIER blow (Zhihu
//!   <https://zhuanlan.zhihu.com/p/2025549626309133730>, seen through a search summary).
//!   The alternative, each blow's +30% applying to that blow (ATK +(130% + 30% * k)),
//!   gives +10.4% total at DEF 0 with the default talent 2 (sum of ATK factors 15.9
//!   against 14.4).
//! - S3 splash: PRTS (<https://prts.wiki/w/怒潮凛冬>) says each blow is a splash centred
//!   one tile ahead with a 1.5 radius and normally no main target; whether the other
//!   enemies take the full 240% or the trait's 50% is not stated anywhere found. The
//!   formula keeps the trait model (the target full, the others 50% x talent 1).

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::{UNLIMITED_DURATION, module_trait_value};

/// Trait splash, fraction of ATK (character trait text; the module keeps it as
/// `attack@atk_scale_2` 0.5).
const SPLASH_SCALE: f64 = 0.5;
/// Blows in S3 ("a chain of five hammer blows").
const S3_BLOWS: i32 = 5;

/// S2 from its second activation on (`2ndActivation` ON) has unlimited duration;
/// every other skill keeps its game-data duration.
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    (unit.skill_index == 2 && unit.skill_damage).then_some(UNLIMITED_DURATION)
}

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let others = f64::from(unit.targets - 1);

    let atk_scale = if unit.module_index == 1 && unit.module_damage {
        module_trait_value(unit, "atk_scale_e").unwrap_or(1.0)
    } else {
        1.0
    };
    let splash_dmg_scale = if unit.elite > 0 {
        unit.talent1_parameters.first().copied().unwrap_or(1.0)
    } else {
        1.0
    };
    // Human Tide applies while a skill is active, doubled for her own group.
    let mut atkbuff = if unit.elite == 2 && skill > 0 {
        unit.talent2_parameters.first().copied().unwrap_or(0.0)
            * unit.talent2_parameters.get(2).copied().unwrap_or(1.0)
    } else {
        0.0
    };

    // One attack at `final_atk` with skill multiplier `scale`: the target plus the
    // splash on every other enemy.
    let attack = |final_atk: f64, scale: f64| {
        let main = final_atk * atk_scale * scale;
        let splash = SPLASH_SCALE * main;
        (main - defense).max(main * 0.05)
            + (splash - defense).max(splash * 0.05) * splash_dmg_scale * others
    };

    let mut aspd = 0.0;
    match skill {
        1 => {
            atkbuff += param(0);
            aspd += param(1);
        }
        2 => {
            atkbuff += if unit.skill_damage {
                param(2)
            } else {
                param(0)
            }
        }
        3 => {
            let mut total = 0.0;
            for k in 0..S3_BLOWS {
                let blow_buff = atkbuff + param(4) + f64::from(k) * param(1);
                let final_atk = unit.atk * (1.0 + blow_buff + unit.buff_atk) + unit.buff_atk_flat;
                total += attack(final_atk, param(0));
            }
            return Some(total);
        }
        _ => {}
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    Some(attack(final_atk, 1.0) / atk_interval * (unit.attack_speed + aspd) / 100.0)
}
