//! Astgenne the Lightchaser (`char_1047_halo2`): `ArknightsDpsCompare` has no class
//! for her. Talent 1 is read at full stacks; S1 assumes every jump finds a target.

use super::super::super::operator_unit::{EnemyStats, OperatorUnit};

#[allow(clippy::unnecessary_wraps)]
pub fn dps(unit: &OperatorUnit, enemy: &EnemyStats) -> Option<f64> {
    let skill = unit.skill_index;
    let res = enemy.res;
    let sp = |i: usize| unit.skill_parameters.get(i).copied().unwrap_or(0.0);
    let mut atk_interval = f64::from(unit.attack_interval);
    let mut atkbuff: f64 = 0.0;

    let mut aspd: f64 = 0.0;
    if unit.talent_damage && unit.elite > 0 {
        aspd = unit.talent1_parameters.first().copied().unwrap_or(0.0)
            * unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        // The module's hidden level 3 talent part: +ATK at full stacks.
        if unit.module_index == 1 && unit.module_level == 3 {
            atkbuff += unit
                .data
                .talent1_module_extra
                .iter()
                .find(|e| e.required_module_level == 3)
                .and_then(|e| e.talent_data.first().copied())
                .unwrap_or(0.0);
        }
    }

    let mut fragile: f64 = 0.0;
    if unit.elite == 2 {
        let idx = if unit.talent2_damage { 3 } else { 0 };
        fragile = unit.talent2_parameters.get(idx).copied().unwrap_or(1.0) - 1.0;
    }
    // Her Fragile does not stack with external Fragile: the larger applies, and the
    // engine's own factor is divided back out at the end.
    fragile = fragile.max(unit.buff_fragile);

    let targets = f64::from(unit.targets);
    let mut skill_atk_scale: f64 = 1.0;
    let mut link_scale: f64 = 1.0;
    let mut hits: f64 = 1.0;
    let mut share: f64 = 0.0;
    if skill == 1 {
        atkbuff += sp(0);
        if targets >= 2.0 {
            hits = 1.0 + sp(1);
        }
    }
    if skill == 2 {
        skill_atk_scale = sp(0);
        link_scale = sp(5);
    }
    if skill == 3 {
        atkbuff += sp(0);
        atk_interval += sp(1);
        hits = targets.min(sp(2));
        share = sp(3);
    }

    let final_atk = unit.atk.mul_add(1.0 + atkbuff + unit.buff_atk, unit.buff_atk_flat);
    let arts = |scale: f64| (final_atk * scale * (1.0 - res / 100.0)).max(final_atk * scale * 0.05);
    let hitdmg = arts(1.0);
    let mut avghit = hitdmg * hits * share.mul_add(hits - 1.0, 1.0);

    if skill == 2 {
        let primary = targets.min(sp(6));
        let linked = (targets - primary).max(0.0).min(sp(6) * sp(2));
        let skilldmg = primary * arts(skill_atk_scale) + linked * arts(link_scale);
        let sp_cost = f64::from(unit.skill_cost);
        avghit = sp_cost.mul_add(hitdmg, skilldmg) / (sp_cost + 1.0);
    }

    let dps = avghit / atk_interval * (unit.attack_speed + aspd) / 100.0;
    Some(dps * (1.0 + fragile) / (1.0 + unit.buff_fragile))
}
