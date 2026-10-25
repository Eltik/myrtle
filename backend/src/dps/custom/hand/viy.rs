//! Viy (`char_4226_veen`), Caster / Mystic, base attack time 3.0 s, Arts.
//! Hand-written: `ArknightsDpsCompare` has no class for her. Written from the EN game
//! data (`character_table`, `skill_table`, `battle_equip_table`), checked 2026-10-25.
//!
//! Game data behind each number:
//! - Trait (`Trait_` blackboard `times` 9, `merge_cnt` 3): when no target is found,
//!   attacks are stored (up to 3 charges) and fired all at once.
//! - Talent 1 "Before the Sword's Swing", `talent1_parameters`: `[0]` `atk` (E2 0.10,
//!   P5 0.12), `[1]` `attack_speed` (E2 15). It has no promotion gate: its first
//!   candidate unlocks at `PHASE_0` (ATK +2%, ASPD +5, EN and CN `character_table`; PRTS
//!   lists 精英0 and 精英2 rows, https://prts.wiki/w/维伊), so the formula reads it at
//!   every elite. 3 stored charges merge into 1 Transposed
//!   Energy (TE); ATK +`[0]` while she holds TE, ASPD +`[1]` while she does not.
//! - Talent 2 "Arts of War" (E2), `talent2_parameters`: `[0]` `attack@duration` 5,
//!   `[1]` `attack@value` 90, `[2]` `attack@max_stack_cnt` 3. A stored-charge attack
//!   puts a 90 Arts/s DoT on the target for 5 s (TE inflicts 3 stacks), and she does
//!   not attack a target under it. Module MSC-Y (`uniequip_002_veen`) stage 2 / 3
//!   overrides it to 100 / 120 per s and 4 stacks (TALENT_DATA_ONLY, talent index 1).
//! - Module MSC-Y: ATK +85 / +112 / +120 (attribute blackboard, applied by the engine)
//!   and "ASPD +30 when there are stored attacks" (hidden TALENT part `10_root`,
//!   `attack_speed` 30, no name, so `OperatorData` does not file it under a talent; read
//!   from the unit's module data with `util::module_talent_value`).
//! - S1 `skchr_veen_1`, `skill_parameters`: `[0]` `attack_speed` 100. 18 s.
//! - S2 `skchr_veen_2`: `[0]` `base_attack_time` -0.5, `[1]` stack `atk` 0.15, `[2]`
//!   stack `attack_speed` 8, `[3]` `max_stack_cnt` 9. 28 s.
//! - S3 `skchr_veen_3`: `[0]` `attack_speed` 180, `[1]` `attack@bounce_atk_scale` 1.65,
//!   `[2]` `attack@delay` 0.4, `[3]` `attack@trigger_time` 27 (ammo), `[4]`
//!   `merge_hit_cnt` 3, `[5]` `attack@base_atk_scale` 1.0. Ammo, no duration.
//!
//! Conditionals:
//! - trait `storedCharges` (ON): every attack is fired as a stored charge. This is her
//!   E2 single-target steady state: a release puts Arts of War on the target, she cannot
//!   attack it for 5 s, so she stores until it expires and releases again (PRTS Mystic
//!   note: with no valid target, each attack check stores one charge, at the attack
//!   interval; the next real attack releases everything). Throughput is one charge per
//!   attack interval. OFF: plain attacks on a target she can always hit
//!   (no stored charges: no TE, no DoT, no S3 bounce, no module ASPD).
//! - talent `transposedEnergy` (ON, only with `storedCharges`): charges are released as
//!   Transposed Energy: ATK +talent `[0]`, no talent ASPD, 3 DoT stacks per release.
//!   OFF: the talent ASPD +`[1]` applies and each release is a single charge (1 stack).
//! - talent2 `artsOfWarDot` (ON, E2 and `storedCharges`): the Arts of War DoT.
//! - skill `maxStacks` (ON, S2): 9 stacks (ATK +135%, ASPD +72) for the whole window,
//!   the upstream `maxStacks` convention. OFF: no stacks.
//!
//! ASSUMPTIONS and simplifications:
//! - A Transposed Energy is the damage of 3 stored charges (it counts 3 ammo, 3 S2
//!   stacks, 3 DoT stacks and 3 bounces); every stored charge hits at
//!   `base_atk_scale` (100% ATK), the same as a normal attack.
//! - Charge storage is lossless: every stored charge is released, and the 9-charge cap
//!   is never reached (at her fastest, S3 with the module and the talent ASPD, ASPD
//!   325 and a 3.0 s base interval, 5.4 attacks fit in a 5 s DoT window).
//! - The DoT is on the primary target with 100% uptime (it is reapplied by the release
//!   that its own expiry allows). A release carries the charges stored during the
//!   previous 5 s lockout, `duration / effective interval` (interval / ASPD, taken as
//!   continuous), so stacks = min(cap, charges per release, that count). At S0 (3.0 s,
//!   ASPD 100) only 1.67 charges fit; S1 to S3 store 3 or more.
//! - NOT modelled: PRTS notes that during S3 a released charge deals no damage of its
//!   own and only adds its bounce (https://prts.wiki/w/维伊, S3 notes); the formula
//!   keeps charge + bounce. The CN talent 2 text ("攻击和储存的能量") may also apply
//!   the DoT on normal attacks; the formula follows the EN text (stored charges only).
//! - S3: each stored charge bounces once for `bounce_atk_scale`; with one enemy the
//!   bounce returns to it ("prioritizing different targets"), so the total does not
//!   depend on the target count. The 0.4 s bounce delay is ignored.
//! - S3 duration: the game data has none (ammo, `Duration` -1). With `storedCharges`
//!   ON every attack interval releases one charge and spends 1 ammo (a TE is 3 charges
//!   and 3 ammo), so `skill_duration` reports `trigger_time` attack intervals at the S3
//!   attack speed: 27 x 3.0 / 2.80 s with the talent and no module. With it OFF no
//!   charge is released, no ammo is spent and the skill does not end: the game-data
//!   -1 (unlimited) stays, and `total_damage` and `average_dps` equal `skill_dps`.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_talent_value;

/// Trait `merge_cnt`: stored charges per Transposed Energy, also its DoT stacks.
const MERGE_CNT: f64 = 3.0;

/// ASPD from talent 1 (when she holds no Transposed Energy) and module MSC-Y (when
/// she has stored attacks), before any skill's.
fn talent_and_module_aspd(unit: &OperatorUnit) -> f64 {
    let stored = unit.trait_damage;
    let transposed = stored && unit.talent_damage;
    let mut aspd: f64 = 0.0;
    if !transposed {
        aspd += unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
    }
    if stored && unit.module_index == 1 {
        aspd += module_talent_value(unit, "10_root", "attack_speed").unwrap_or(0.0);
    }
    aspd
}

/// S3 with stored charges: `trigger_time` ammo, one spent per attack interval.
/// Otherwise the game-data duration (S3 without stored charges never ends).
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 3 || !unit.trait_damage {
        return None;
    }
    let ammo = unit.skill_parameters.get(3).copied().unwrap_or(0.0);
    let aspd = talent_and_module_aspd(unit) + unit.skill_parameters.first().copied().unwrap_or(0.0);
    Some(ammo * f64::from(unit.attack_interval) / ((unit.attack_speed + aspd) / 100.0))
}

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let res = enemy.res;
    let sp = |i: usize, d: f64| unit.skill_parameters.get(i).copied().unwrap_or(d);
    let t1 = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
    let t2 = |i: usize| unit.talent2_parameters.get(i).copied().unwrap_or(0.0);

    let stored = unit.trait_damage;
    let transposed = stored && unit.talent_damage;

    let mut atk_interval = f64::from(unit.attack_interval);
    let mut atkbuff: f64 = 0.0;
    let mut aspd = talent_and_module_aspd(unit);
    let mut atk_scale: f64 = 1.0;
    let mut bounce_scale: f64 = 0.0;

    if transposed {
        atkbuff += t1(0);
    }

    if skill == 1 {
        aspd += sp(0, 0.0);
    }
    if skill == 2 {
        atk_interval += sp(0, 0.0);
        if unit.skill_damage {
            let stacks = sp(3, 0.0);
            atkbuff += sp(1, 0.0) * stacks;
            aspd += sp(2, 0.0) * stacks;
        }
    }
    if skill == 3 {
        aspd += sp(0, 0.0);
        if stored {
            atk_scale = sp(5, 1.0);
            bounce_scale = sp(1, 0.0);
        }
    }

    let final_atk = unit.atk * (1.0 + atkbuff + unit.buff_atk) + unit.buff_atk_flat;
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let hitdmg = arts(atk_scale);
    let bouncedmg = if bounce_scale > 0.0 {
        arts(bounce_scale)
    } else {
        0.0
    };

    let mut dps = (hitdmg + bouncedmg) / atk_interval * (unit.attack_speed + aspd) / 100.0;

    if stored && unit.talent2_damage && unit.elite == 2 {
        let per_release = if transposed { MERGE_CNT } else { 1.0 };
        // Charges stored while the DoT (`[0]` duration) locks the target out.
        let effective_interval = atk_interval / ((unit.attack_speed + aspd) / 100.0);
        let stored_in_lockout = t2(0) / effective_interval;
        let stacks = t2(2).min(per_release).min(stored_in_lockout);
        let value = t2(1);
        dps += (value * (1.0 - res / 100.0)).max(value * 0.05) * stacks;
    }

    Some(dps)
}
