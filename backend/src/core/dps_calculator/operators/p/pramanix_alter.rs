//! DPS calculations for PramanixAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// PramanixAlter operator implementation
pub struct PramanixAlter {
    pub unit: OperatorUnit,
}

impl PramanixAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

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
    )] = &[
        ("module", "manyTargets", false, &[], &[2], 0, 0),
        ("skill", "iceOnly", true, &[2], &[], 0, 0),
    ];

    /// Creates a new PramanixAlter operator
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
    /// atk_scale = 1.15 if self.module == 2 and self.module_dmg else 1
    /// if self.skill == 0: return (defense * 0)
    /// if self.skill == 1:
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale * (1-res/100), final_atk * atk_scale * skill_scale * 0.05)
    /// dps = hitdmg * self.targets / (self.skill_cost+1) * (1 + self.sp_boost)
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// ice_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale * (1-res/100), final_atk  * skill_scale * atk_scale * 0.05)
    /// icedmg = np.fmax(final_atk * atk_scale * ice_scale * (1-res/100), final_atk * atk_scale * ice_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 * self.targets + icedmg * self.targets
    /// if not self.skill_dmg:
    /// dps = icedmg * self.targets
    /// if self.skill == 3:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// skill_scale = self.skill_params[5]
    /// newres = np.fmax(0, res - 10)
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale * (1-newres/100), final_atk  * skill_scale * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+30)/100 * self.targets
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

        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            return (defense * 0.0);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = hitdmg * (self.unit.targets as f64) / ((self.unit.skill_cost as f64) + 1.0)
                * (1.0 + (self.unit.sp_boost as f64));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut ice_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            let mut icedmg = ((final_atk * atk_scale * ice_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * ice_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
                * (self.unit.targets as f64)
                + icedmg * (self.unit.targets as f64);
            if !self.unit.skill_damage {
                dps = icedmg * (self.unit.targets as f64);
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            skill_scale = self.unit.skill_parameters.get(5).copied().unwrap_or(0.0);
            newres = ((0) as f64).max((res - 10.0) as f64);
            hitdmg = ((final_atk * atk_scale * skill_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + 30.0)
                / 100.0
                * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for PramanixAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for PramanixAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for PramanixAlter {
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
