//! Closure (`char_4228_closur`), Vanguard / Tactician, base attack time 1.0 s,
//! Physical, ranged. Hand-written: `ArknightsDpsCompare` has no class for her. Written
//! from the EN game data (`character_table`, `skill_table`, `battle_equip_table`),
//! checked 2026-10-25, in the style of the upstream tactician formula (Vigil).
//!
//! She deals real damage: ATK +80% on two targets on S2, and 250% ATK every 0.5 s on a
//! growing target count on S3, both x1.5 against enemies blocked by her Reinforcements.
//!
//! Game data behind each number:
//! - Trait (`Trait_` blackboard `atk_scale` 1.5): ATK x1.5 against enemies blocked by
//!   her Reinforcements (her Command Center, `token_10066_closur_ourbase`, "does not
//!   attack; blocks 2 enemies", and allies inside the Tactical Point range).
//! - Talent 2 "Scheduler, Max Settings!" (E2), `talent2_parameters`: `[0]` `cost`,
//!   `[1]` `atk` 0.04 (module TAC-X `uniequip_002_closur` stage 2 / 3: 0.06 / 0.08),
//!   "[Rhodes Island] Operator ATK +4%"; she is Rhodes Island (`NationId` `rhodes`), so
//!   it always applies to her at E2.
//! - Module TAC-X: ATK +20 / +25 / +30 (attribute blackboard, applied by the engine);
//!   the trait addition (Reinforcements take 15% less damage) is defensive.
//! - S1 `skchr_closur_1`: DP and a Shield only; her damage is her normal attacks.
//! - S2 `skchr_closur_2` (30 s), `skill_parameters`: `[4]` `atk` 0.8, `[5]`
//!   `attack@max_target` 2.
//! - S3 `skchr_closur_3` (30 s): `[2]` `attack_trigger_cnt` 9, `[3]` `max_trigger_cnt`
//!   6, `[4]` `attack@atk_scale` 2.5, `[5]` `base_attack_time` -0.5. Target count +1
//!   after every 9 attacks, at most 6 times (1 to 7 targets).
//!
//! Conditionals:
//! - trait `vsBlocked` (ON): the target is blocked by her Reinforcements (ATK x1.5).
//!   UPPER BOUND on S2 and S3 with more than one target: ON applies x1.5 to EVERY
//!   target. PRTS (https://prts.wiki/w/可露希尔) says S3 attacks any enemy in her, the
//!   Tactical Point's and her Reinforcements' ranges, not only blocked ones, and the
//!   trait's x1.5 is per enemy (blocked by a summon or a marked ally); her Command
//!   Center blocks 2.
//!
//! ASSUMPTIONS and simplifications:
//! - S3 target ramp: attack k (from 0) can hit 1 + min(6, floor(k / 9)) targets, capped
//!   at the enemies present. `skill_dps` averages that over the attacks that fit in the
//!   window (duration x attack rate, taken as continuous), so with one enemy it is a
//!   plain single-target rate.
//! - S2's "can attack enemies blocked by her own Reinforcements" is read as enabling
//!   the trait bonus on S2 the same way as on the other skills; it changes nothing here.
//! - NOT modelled: the S3 Sluggish (no damage), DP generation.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// Trait blackboard `atk_scale`: ATK x1.5 against enemies her Reinforcements block.
const TRAIT_ATK_SCALE: f64 = 1.5;

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);

    let atk_scale = if unit.trait_damage {
        TRAIT_ATK_SCALE
    } else {
        1.0
    };
    let mut atkbuff = if unit.elite == 2 {
        unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
    } else {
        0.0
    };
    let mut atk_interval = f64::from(unit.attack_interval);
    let mut skill_scale: f64 = 1.0;
    let mut targets: f64 = 1.0;

    if skill == 2 {
        atkbuff += sp(4, 0.0);
        targets = f64::from(unit.targets).min(sp(5, 1.0));
    }
    if skill == 3 {
        atk_interval += sp(5, 0.0);
        skill_scale = sp(4, 1.0);
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    let scale = atk_scale * skill_scale;
    let hitdmg = (final_atk * scale - defense).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;

    if skill == 3 {
        targets = s3_average_targets(
            attack_rate * unit.skill_duration,
            f64::from(unit.targets),
            sp(2, 1.0),
            sp(3, 0.0),
        );
    }

    Some(hitdmg * attack_rate * targets)
}

/// Mean target count over `attacks` attacks (continuous) when attack k hits
/// 1 + min(`max_trigger`, floor(k / `per_trigger`)) targets, capped at `enemies`.
fn s3_average_targets(attacks: f64, enemies: f64, per_trigger: f64, max_trigger: f64) -> f64 {
    if attacks <= 0.0 || per_trigger <= 0.0 {
        return enemies.min(1.0 + max_trigger).max(1.0);
    }
    let mut sum = 0.0;
    let mut step = 0.0;
    while step <= max_trigger {
        let start = step * per_trigger;
        if start >= attacks {
            break;
        }
        let end = if step < max_trigger {
            ((step + 1.0) * per_trigger).min(attacks)
        } else {
            attacks
        };
        sum += (end - start) * enemies.min(1.0 + step);
        step += 1.0;
    }
    sum / attacks
}
