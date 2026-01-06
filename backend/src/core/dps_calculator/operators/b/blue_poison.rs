//! DPS calculations for BluePoison
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// BluePoison operator implementation
pub struct BluePoison {
    pub unit: OperatorUnit,
}

impl BluePoison {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [(
        &'static str,
        &'static str,
        bool,
        &'static [i32],
        &'static [i32],
        i32,
        i32,
    )] = &[("module", "GroundTargets", false, &[], &[2], 0, 0)];

    /// Creates a new BluePoison operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// aspd = 8 if self.module_dmg and self.module == 2 else 0
    /// artsdmg = self.talent1_params[1]
    /// artsdps = np.fmax(artsdmg * (1 - res/100), artsdmg * 0.05) if self.elite > 0 else 0
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// skillhitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05) * min(2,self.targets)
    /// if self.skill == 0: skillhitdmg = hitdmg
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1)
    /// dps = avgphys/(self.atk_interval/((self.attack_speed + aspd)/100)) + artsdps * min(1 + self.skill,self.targets)
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = self.skill_params[1] * hitdmg/(self.atk_interval/((self.attack_speed+ aspd)/100)) + hitdmg/(self.atk_interval/((self.attack_speed+ aspd)/100)) * min(2,self.targets-1) + artsdps * min(3, self.targets)
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

        let mut avgphys: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        aspd = if self.unit.module_damage && ((self.unit.module_index as f64) as f64) == 2.0 {
            8.0
        } else {
            0.0
        };
        let mut artsdmg = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        let mut artsdps = if ((self.unit.elite as f64) as f64) > 0.0 {
            ((artsdmg * (1.0 - res / 100.0)) as f64).max((artsdmg * 0.05) as f64)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            let mut skillhitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * ((2) as f64).min((self.unit.targets as f64) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skillhitdmg = hitdmg;
            }
            sp_cost = (self.unit.skill_cost as f64);
            avgphys = (sp_cost * hitdmg + skillhitdmg) / (sp_cost + 1.0);
            dps = avgphys
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                + artsdps
                    * ((1.0 + (self.unit.skill_index as f64)) as f64)
                        .min((self.unit.targets as f64) as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0) * hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                + hitdmg
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0))
                    * ((2) as f64).min(((self.unit.targets as f64) - 1.0) as f64)
                + artsdps * ((3) as f64).min((self.unit.targets as f64) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for BluePoison {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for BluePoison {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for BluePoison {
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
