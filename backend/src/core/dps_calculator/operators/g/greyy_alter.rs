//! DPS calculations for GreyyAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// GreyyAlter operator implementation
pub struct GreyyAlter {
    pub unit: OperatorUnit,
}

impl GreyyAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[];

    /// Creates a new GreyyAlter operator
    #[allow(unused_parens)]
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
    /// bonushits = 2 if self.module == 1 else 1
    /// dmg = 1 + 0.05 * self.module_lvl if self.module == 1 and self.module_lvl > 1 else 1
    /// if self.skill < 2:
    /// atkbuff = self.skill_params[0] * self.skill
    /// aspd = self.skill_params[1] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * dmg
    /// bonusdmg =  np.fmax(final_atk * 0.5 - defense, final_atk * 0.5 * 0.05) * dmg
    /// dps = (hitdmg + bonusdmg * bonushits) / self.atk_interval * (self.attack_speed + aspd) / 100 * self.targets
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05) * dmg
    /// bonusdmg = np.fmax(final_atk * 0.5 - defense, final_atk * 0.5 * 0.05) * dmg
    /// hitdmgarts = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * dmg
    /// dps = (hitdmg + bonusdmg * bonushits) / self.atk_interval * (self.attack_speed) / 100 * self.targets
    /// dps += hitdmgarts / 1.5 * self.targets
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        let mut bonushits = if ((self.unit.module_index as f64) as f64) == 1.0 {
            2.0
        } else {
            1.0
        };
        let mut dmg = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            1.0 + 0.05 * ((self.unit.module_level as f64) as f64)
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmg;
            bonusdmg =
                ((final_atk * 0.5 - defense) as f64).max((final_atk * 0.5 * 0.05) as f64) * dmg;
            dps = (hitdmg + bonusdmg * bonushits) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmg;
            bonusdmg =
                ((final_atk * 0.5 - defense) as f64).max((final_atk * 0.5 * 0.05) as f64) * dmg;
            hitdmgarts = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * dmg;
            dps = (hitdmg + bonusdmg * bonushits) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed)
                / 100.0
                * (self.unit.targets as f64);
            dps += hitdmgarts / 1.5 * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for GreyyAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for GreyyAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for GreyyAlter {
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
