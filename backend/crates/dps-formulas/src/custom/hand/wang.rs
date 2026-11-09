//! Wang (`char_2027_wang`), Specialist / Trapper (`traper`), base interval 0.85 s,
//! Physical attacks. Hand-written: `ArknightsDpsCompare` has no class for him. Written
//! from the EN game data (`character_table`, `skill_table`, `battle_equip_table`, the
//! stone token `token_10064_wang_stone1` and its skills `sktok_wang_1..3`), checked
//! 2026-10-25; mechanics from PRTS (<https://prts.wiki/w/望>, <https://prts.wiki/w/棋子>)
//! and arknights.wiki.gg (<https://arknights.wiki.gg/wiki/Wang/Overview>).
//!
//! His damage comes from weiqi stones (traps). Talent 1 (Mold Stones) places them;
//! the stone's damage is the passive of the equipped skill. With no skill (S0) a stone
//! has no damage effect, so S0 is his own attacks only.
//!
//! `skill_parameters` are the raw blackboard order:
//! - S1 `skchr_wang_1` (auto, gains stones): `[0]` `cnt` 2 (stones per activation),
//!   `[1]` `attack@atk_scale` 1.35 (Arts per second on the triggering enemy), `[2]`
//!   `attack@sluggish` 6.5 (seconds).
//! - S2 `skchr_wang_2` (auto, gains stones): `[0]` `cnt` 2, `[1]` `attack@atk_scale` 5.8
//!   (Arts to all enemies within 3 tiles along the stone's links).
//! - S3 `skchr_wang_3` (manual, ammo, stops attacking): `[0]` `cnt` 8 (stones granted),
//!   `[1]` `trigger_time` 20 (ammo, spent on talent 1's extra stones), `[2]` `atk_scale`
//!   3.8 (Arts, expanded damage range).
//!
//! `talent1_parameters` (Mold Stones): `[3]` `attack@max_spawn_cnt`, the cap on talent
//! 1's extra stones: 0 at E0, 8 at E1 P1, 9 at E1 P3 and E2 P1, 10 at E2 P3. From E1,
//! "when manually deploying a stone, Wang will deploy an additional stone on a tile
//! next to it".
//! `talent2_parameters` (Every Move Anticipated, E2): `[0]` `attack@per_atk_scale` 0.1,
//! `[1]` `attack@per_magic_resist_penetrate_fixed` 9, `[2]` `attack@max_trigger_cnt` 3:
//! per stone in the same straight chain, the chain's stones deal +10% damage and ignore
//! 9 RES, up to 3 stacks. The module raises it (0.13 / 12 at level 3) through the
//! engine's talent resolution.
//!
//! Module 1 (`uniequip_002_wang`, TRP-X): ATK +60 at level 3 (attribute), a display-only
//! trait (lower trap DP cost, higher deploy limit) and the talent 2 upgrade. No
//! conditional of its own.
//!
//! What the sources settle (PRTS notes on the Wang and stone pages, wiki.gg Overview):
//! - S1 / S2 put their 2 stones in his hand (hold cap `MaxDeckStackCnt` 7 at E2), not on
//!   the field, and their SP is frozen while the hand is full; S3 ends when the stones
//!   run out. A stone is armed by an orthogonal link to another stone, and triggers
//!   when an enemy enters its tile.
//! - Talent 1's extra stone (a "跟子") comes only from a MANUAL placement; it triggers,
//!   links and deals damage like a stone. S3's overflow stones are auto-placed 跟子.
//! - Each S1 trigger adds its own damage-over-time instance; instances stack.
//!
//! ASSUMPTIONS (no source states these; review them before relying on the numbers):
//! - Every stone is triggered exactly once (a mine-like stone, as Dorothy's mines are
//!   modelled): the sources show the hand stock is spent by placing stones, not that a
//!   placed stone is used up by its trigger. No chain triggering: linked stones only
//!   arm each other (S2 and S3 reach the line through their area damage). Stones arrive
//!   only from the skill: `cnt` per activation, each placed by hand, so doubled from E1
//!   by talent 1's extra stone, one activation per `sp_cost / (1 + sp_boost)` seconds
//!   (Dorothy's "1MinePerSPcost"). The starting stones are not modelled.
//! - S1: the damage-over-time ticks once per second from 1 s after the trigger, so a
//!   6.5 s effect deals 6 whole ticks (`floor(6.5)`) on one enemy.
//! - S2 and S3 stones hit every target (`targets`), S1's hits one.
//! - `maxTalent2` ON: every stone sits in a chain of 3 or more (max stacks); OFF: no
//!   stacks.
//! - S3: `cnt + trigger_time` = 28 stone activations (8 granted stones, and 20 extra
//!   stones paid with the 20 ammo), as a rate over the skill's duration. The skill has
//!   no timer (ammo, `Duration` -1); it is paced by manual placement. Each manual
//!   placement carries up to `S3_EXTRA_PER_PLACEMENT` (3, skill text) ammo-paid extra
//!   stones, so the ammo lasts `ceil(trigger_time / 3)` = 7 placements, one per stone
//!   redeploy time (`STONE_REDEPLOY_E2` 2 s): `skill_duration` reports 7 x 2 = 14 s.
//!   Every stone is assumed to trigger as soon as it is placed. His own attacks stop
//!   during S3.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

/// S3 text: "Talent 1 deploys up to 3 additional stones at the cost of an equal amount
/// of ammo" per manual placement (no blackboard key).
const S3_EXTRA_PER_PLACEMENT: f64 = 3.0;
/// `token_10064_wang_stone1` `RespawnTime` at E2 (S3 needs E2): the stone card's
/// redeploy cooldown after each placement, in seconds.
const STONE_REDEPLOY_E2: f64 = 2.0;

/// S3: one manual placement per stone redeploy time until the ammo is spent, each
/// placement paying `S3_EXTRA_PER_PLACEMENT` ammo. Every other skill keeps its
/// game-data duration (unlimited for S1 / S2).
pub fn skill_duration(unit: &OperatorUnit) -> Option<f64> {
    if unit.skill_index != 3 {
        return None;
    }
    let ammo = unit.skill_parameters.get(1).copied().unwrap_or(0.0);
    Some((ammo / S3_EXTRA_PER_PLACEMENT).ceil() * STONE_REDEPLOY_E2)
}

// The `Option` is the registry signature every DPS formula shares.
#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let defense = enemy.defense;
    let res = enemy.res;
    let atk_interval = f64::from(unit.attack_interval);
    let param = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let talent2 = |i: usize| unit.talent2_parameters.get(i).copied().unwrap_or(0.0);
    let targets = f64::from(unit.targets);

    let final_atk = unit.atk * (1.0 + unit.buff_atk) + unit.buff_atk_flat;
    let attack_dps =
        (final_atk - defense).max(final_atk * 0.05) / atk_interval * unit.attack_speed / 100.0;

    // Talent 2 chain stacks, E2 only (below it the slots hold placeholder defaults).
    let stacks = if unit.elite == 2 && unit.talent2_damage {
        talent2(2)
    } else {
        0.0
    };
    let stone_scale = 1.0 + stacks * talent2(0);
    let stone_res = (res - stacks * talent2(1)).max(0.0);
    let stone_hit = |scale: f64| {
        (final_atk * scale * (1.0 - stone_res / 100.0)).max(final_atk * scale * 0.05) * stone_scale
    };

    // Stones placed per skill activation, and activations per second.
    let extra_stone = if unit.talent1_parameters.get(3).copied().unwrap_or(0.0) > 0.0 {
        2.0
    } else {
        1.0
    };
    let cycle = f64::from(unit.skill_cost) / (1.0 + f64::from(unit.sp_boost));
    let stones_per_second = if cycle > 0.0 {
        param(0) * extra_stone / cycle
    } else {
        0.0
    };

    let dps = match skill {
        1 => attack_dps + stone_hit(param(1)) * param(2).floor() * stones_per_second,
        2 => attack_dps + stone_hit(param(1)) * targets * stones_per_second,
        3 => {
            let duration = unit.skill_duration;
            let total = (param(0) + param(1)) * stone_hit(param(2)) * targets;
            if duration > 0.0 {
                total / duration
            } else {
                total
            }
        }
        _ => attack_dps,
    };

    Some(dps)
}
