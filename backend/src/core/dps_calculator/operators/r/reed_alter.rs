//! DPS calculations for ReedAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// ReedAlter operator implementation
pub struct ReedAlter {
    pub unit: OperatorUnit,
}

impl ReedAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "w/o cinder", true, &[3], &[], 1, 0),
        ("skill", "Sandwiched", false, &[2], &[], 0, 0),
    ];

    /// Creates a new ReedAlter operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            1, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    /// dmg_scale = self.talent1_params[2] if (self.talent_dmg and self.elite > 1) or self.skill == 3 else 1
    /// if self.skill < 2:
    /// atkbuff = self.skill_params[0] * self.skill
    /// aspd = self.skill_params[1] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-res/100), final_atk * 0.05) * dmg_scale
    /// dps = hitdmgarts/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[1]
    /// multiplier = 2 if self.skill_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(1-res/100,  0.05) * final_atk * atk_scale * dmg_scale * multiplier
    /// dps = hitdmgarts/0.8
    /// if self.skill == 3:
    /// atkbuff = self.skill_params[1]
    /// atk_scale = self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// directhits = np.fmax(final_atk *(1-res/100), final_atk * 0.05) * dmg_scale
    /// atkdps = directhits/self.atk_interval * self.attack_speed/100 * min(self.targets,2)
    /// skillhits = np.fmax(final_atk *(1-res/100), final_atk * 0.05) * dmg_scale * atk_scale
    /// skilldps = self.targets * skillhits
    /// dps = atkdps + skilldps
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

        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atk_scale: f64 = 0.0;

        let mut dmg_scale = if (self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 1.0)
            || ((self.unit.skill_index as f64) as f64) == 3.0
        {
            self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
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
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * dmg_scale;
            dps = hitdmgarts / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            let mut multiplier = if self.unit.skill_damage { 2.0 } else { 1.0 };
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((1.0 - res / 100.0) as f64).max((0.05) as f64)
                * final_atk
                * atk_scale
                * dmg_scale
                * multiplier;
            dps = hitdmgarts / 0.8;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            atk_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut directhits = ((final_atk * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * dmg_scale;
            let mut atkdps =
                directhits / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                    * ((self.unit.targets as f64) as f64).min((2) as f64);
            let mut skillhits = ((final_atk * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64)
                * dmg_scale
                * atk_scale;
            let mut skilldps = (self.unit.targets as f64) * skillhits;
            dps = atkdps + skilldps;
        }
        return dps;
    }
}

impl std::ops::Deref for ReedAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ReedAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for ReedAlter {
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
