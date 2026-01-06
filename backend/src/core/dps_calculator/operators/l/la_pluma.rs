//! DPS calculations for LaPluma
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// LaPluma operator implementation
pub struct LaPluma {
    pub unit: OperatorUnit,
}

impl LaPluma {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new LaPluma operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
            1, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atkbuff = 0
    /// aspd = self.talent1_params[0] * self.talent1_params[1] if self.talent_dmg else 0
    /// if self.talent_dmg and self.module == 1 and self.module_lvl > 1: atkbuff = self.talent1_params[2]
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + 2 * skillhitdmg) / (sp_cost + 1) if self.skill == 1 else hitdmg
    /// dps = avgphys/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// if self.skill == 2:
    /// atk_interval = self.atk_interval * (1 + self.skill_params[3])
    /// atkbuff += self.skill_params[0]
    /// if self.skill_dmg:
    /// atkbuff += self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/atk_interval * (self.attack_speed+aspd)/100
    ///
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

        let mut aspd: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;

        atkbuff = 0.0;
        aspd = if self.unit.talent_damage {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if self.unit.talent_damage
            && (self.unit.module_index as f64) == 1.0
            && (self.unit.module_level as f64) > 1.0
        {
            atkbuff = self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            sp_cost = (self.unit.skill_cost as f64);
            avgphys = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                (sp_cost * hitdmg + 2.0 * skillhitdmg) / (sp_cost + 1.0)
            } else {
                hitdmg
            };
            dps = avgphys / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_interval = (self.unit.attack_interval as f64)
                * (1.0 + self.unit.skill_parameters.get(3).copied().unwrap_or(0.0));
            atkbuff += self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            if self.unit.skill_damage {
                atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            }
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / atk_interval * (self.unit.attack_speed + aspd) / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for LaPluma {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for LaPluma {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for LaPluma {
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
