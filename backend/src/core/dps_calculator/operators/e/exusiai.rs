//! DPS calculations for Exusiai
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Exusiai operator implementation
pub struct Exusiai {
    pub unit: OperatorUnit,
}

impl Exusiai {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Exusiai operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            1, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = min(self.talent2_params) #they changed the order in the module ffs
    /// aspd = self.talent1_params[0]
    /// if self.module == 2 and self.module_dmg: aspd += 8
    /// newdef = np.fmax(defense - self.talent1_params[1]*self.talent1_params[2],0) if self.module == 2 and self.module_lvl > 1 and self.talent_dmg else defense
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// final_atk = self.atk * (1+atkbuff+self.buff_atk) + self.buff_atk_flat
    /// skill_scale = self.skill_params[0] if self.skill > 0 else 1
    /// if self.skill < 2:
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// skillhitdmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk* atk_scale * skill_scale * 0.05) * 3
    /// avgphys = (self.skill_cost * hitdmg + skillhitdmg) / (self.skill_cost + 1)
    /// if self.skill == 0: avgphys = hitdmg
    /// dps = avgphys/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// elif self.skill == 2:
    /// hitdmg = np.fmax(final_atk *atk_scale * skill_scale - newdef, final_atk* atk_scale* skill_scale * 0.05)
    /// dps = 4*hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// elif self.skill == 3:
    /// atk_interval = self.atk_interval + 2 * self.skill_params[2]
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk* atk_scale* skill_scale * 0.05)
    /// dps = 5*hitdmg/(atk_interval/((self.attack_speed+aspd)/100))
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

        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut avgphys: f64 = 0.0;

        atkbuff = self
            .unit
            .talent2_parameters
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        aspd = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
            aspd += 8.0;
        }
        let mut newdef = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.talent_damage
        {
            ((defense
                - self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
                    * self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0))
                as f64)
                .max((0) as f64)
        } else {
            defense
        };
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        skill_scale = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64)
                * 3.0;
            avgphys = ((self.unit.skill_cost as f64) * hitdmg + skillhitdmg)
                / ((self.unit.skill_cost as f64) + 1.0);
            if (self.unit.skill_index as f64) == 0.0 {
                avgphys = hitdmg;
            }
            dps = avgphys
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        } else if (self.unit.skill_index as f64) == 2.0 {
            hitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = 4.0 * hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        } else if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64)
                + 2.0 * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = 5.0 * hitdmg / (atk_interval / ((self.unit.attack_speed + aspd) / 100.0));
        }
        return dps;
    }
}

impl std::ops::Deref for Exusiai {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Exusiai {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Exusiai {
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
