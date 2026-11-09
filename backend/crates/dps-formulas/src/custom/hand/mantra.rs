//! Mantra (`char_4204_mantra`): `ArknightsDpsCompare` has no class for her. The
//! Nervous Impairment cycle follows the upstream Tragodia / Gracebearer model.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};
use super::util::module_trait_value;

/// Upstream Nervous Impairment burst (Tragodia): total damage and duration.
const NERV_BURST_DAMAGE: f64 = 6000.0;
const NERV_BURST_DURATION: f64 = 10.0;
/// Paralysis stacks a Nervous Impairment burst inflicts (`ba.dt.neural2`).
const NERV_BURST_PARALYSIS: f64 = 3.0;

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

    let final_atk = unit
        .atk
        .mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let attack_rate = unit.attack_speed / 100.0 / atk_interval;
    let hitdmg = arts(1.0);

    // `other_dps` (S2 jumps) stays outside the burst model: see the module doc.
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
        // Module trait: damage to enemies under a burst.
        let burst_scale = if unit.module_index == 1 {
            module_trait_value(unit, "damage_scale").unwrap_or(1.0)
        } else {
            1.0
        };
        let time_to_burst = gauge / nerv_rate;
        let burst_dps = (primary_dps + burst_ele_dps) * burst_scale;
        // Talent 1: each of the burst's Paralysis stacks interrupts once, kept with `prob`.
        let paralysis = if unit.talent_damage && unit.elite > 0 {
            let t1 = |i: usize| unit.talent1_parameters.get(i).copied().unwrap_or(0.0);
            NERV_BURST_PARALYSIS / (1.0 - t1(1)) * t1(0) * final_atk
        } else {
            0.0
        };
        dps = (primary_dps.mul_add(time_to_burst, burst_dps * NERV_BURST_DURATION)
            + NERV_BURST_DAMAGE
            + paralysis)
            / (time_to_burst + NERV_BURST_DURATION);
    }

    Some(dps + other_dps)
}
