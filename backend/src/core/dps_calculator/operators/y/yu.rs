//! DPS calculations for Yu
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Yu operator implementation
pub struct Yu {
    pub unit: OperatorUnit,
}

impl Yu {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[];

    /// Creates a new Yu operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            0, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// newres = np.fmax(0,res-20)
    /// atkbuff = self.skill_params[1] if self.skill == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// dps = 0
    /// time_to_fallout = -1
    /// if self.talent_dmg and self.elite > 0:
    /// arts_scale = self.talent1_params[1]
    /// ele_scale = self.talent1_params[2]
    /// block = 5 if self.skill == 2 else 3
    /// artsdmg1 = np.fmax(final_atk * arts_scale * (1-res/100), final_atk * arts_scale * 0.05) * min(self.targets,block)
    /// artsdmg2 = np.fmax(final_atk * arts_scale * (1-newres/100), final_atk * arts_scale * 0.05) * min(self.targets,block)
    /// ele_gauge = 1000 if self.trait_dmg else 2000
    /// burn_dmg = final_atk * ele_scale
    /// time_to_fallout = ele_gauge / burn_dmg
    /// artsdmg = (artsdmg1 * time_to_fallout + artsdmg2 * 10)/(time_to_fallout + 10)
    /// artsdmg += 7000/(10+time_to_fallout) * min(self.targets,block)
    /// dps = artsdmg
    /// if self.skill == 0: hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// else:
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// if self.talent_dmg and self.elite > 0:
    /// hitdmg2 = np.fmax(final_atk * (1-newres/100), final_atk * 0.05)
    /// hitdmg = (hitdmg * time_to_fallout + hitdmg2 * 10)/(time_to_fallout + 10)
    /// dps += hitdmg/self.atk_interval * self.attack_speed/100
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens,
        clippy::if_same_then_else,
        clippy::nonminimal_bool,
        clippy::overly_complex_bool_expr,
        clippy::needless_return,
        clippy::collapsible_else_if,
        clippy::neg_multiply,
        clippy::assign_op_pattern,
        clippy::eq_op
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut ele_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;

        newres = ((0) as f64).max((res - 20.0) as f64);
        atkbuff = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        dps = 0.0;
        let mut time_to_fallout = -1.0;
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
            let mut arts_scale = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
            ele_scale = self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
            let mut block = if ((self.unit.skill_index as f64) as f64) == 2.0 {
                5.0
            } else {
                3.0
            };
            let mut artsdmg1 = ((final_atk * arts_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * arts_scale * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((block) as f64);
            let mut artsdmg2 = ((final_atk * arts_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * arts_scale * 0.05) as f64)
                * ((self.unit.targets as f64) as f64).min((block) as f64);
            let mut ele_gauge = if self.unit.trait_damage {
                1000.0
            } else {
                2000.0
            };
            let mut burn_dmg = final_atk * ele_scale;
            time_to_fallout = ele_gauge / burn_dmg;
            let mut artsdmg =
                (artsdmg1 * time_to_fallout + artsdmg2 * 10.0) / (time_to_fallout + 10.0);
            artsdmg += 7000.0 / (10.0 + time_to_fallout)
                * ((self.unit.targets as f64) as f64).min((block) as f64);
            dps = artsdmg;
        }
        if (self.unit.skill_index as f64) == 0.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        }
        // UNTRANSLATED ELSE (empty stack): else:
        hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
            hitdmg2 = ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            hitdmg = (hitdmg * time_to_fallout + hitdmg2 * 10.0) / (time_to_fallout + 10.0);
        }
        dps += hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        return dps;
    }
}

impl std::ops::Deref for Yu {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Yu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Yu {
    fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        Self::skill_dps(self, enemy)
    }

    fn unit(&self) -> &OperatorUnit {
        &self.unit
    }

    fn unit_mut(&mut self) -> &mut OperatorUnit {
        &mut self.unit
    }
}
