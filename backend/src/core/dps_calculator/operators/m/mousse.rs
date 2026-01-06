//! DPS calculations for Mousse
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Mousse operator implementation
pub struct Mousse {
    pub unit: OperatorUnit,
}

impl Mousse {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Mousse operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// crate = self.talent1_params[0]
    /// atkbuff = self.skill_params[0] * min(self.skill,1)
    /// aspd = 8 if self.module == 1 and self.module_dmg else 0
    ///
    /// if self.skill < 2:
    /// sp_cost = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// final_atk2 = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg2 = np.fmax(final_atk2 * (1-res/100), final_atk2 * 0.05)
    /// avgdmg = (hitdmg * sp_cost + hitdmg2) / (sp_cost + 1)
    /// dps = avgdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * (1+crate)
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * (1+crate)
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

        let mut sp_cost: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        let mut crit_rate = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            * ((self.unit.skill_index as f64) as f64).min((1) as f64);
        aspd = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            8.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            sp_cost = (self.unit.skill_cost as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut final_atk2 =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            let mut hitdmg2 =
                ((final_atk2 * (1.0 - res / 100.0)) as f64).max((final_atk2 * 0.05) as f64);
            avgdmg = (hitdmg * sp_cost + hitdmg2) / (sp_cost + 1.0);
            dps = avgdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * (1.0 + crit_rate);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * (1.0 + crit_rate);
        }
        return dps;
    }
}

impl std::ops::Deref for Mousse {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Mousse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Mousse {
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
