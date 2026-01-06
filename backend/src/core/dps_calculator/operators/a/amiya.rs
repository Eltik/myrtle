//! DPS calculations for Amiya
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Amiya operator implementation
pub struct Amiya {
    pub unit: OperatorUnit,
}

impl Amiya {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Amiya operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// if self.skill < 2:
    /// aspd = self.skill_params[0] * self.skill
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmgarts/(self.atk_interval/((self.attack_speed +aspd)/100))
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[0]
    /// hits = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hits * hitdmgarts/(self.atk_interval/(self.attack_speed/100))
    /// if self.skill == 3:
    /// final_atk = self.atk * (1 + self.buff_atk + self.params) + self.buff_atk_flat
    /// dps = final_atk/(self.atk_interval/(self.attack_speed/100)) * np.fmax(1,-defense) #this defense part has to be included, because np array
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

        // Amiya S3 ATK buff based on skill level (maps to Python mastery index)
        // skill_level 1-7 -> indices 1-7, skill_level 8-10 -> indices 8-10
        let params: f64 = [0.0, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.8, 2.0, 2.3]
            .get(self.unit.skill_level as usize)
            .copied()
            .unwrap_or(2.3);

        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmgarts: f64 = 0.0;

        if (self.unit.skill_index as f64) < 2.0 {
            aspd = self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmgarts
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            let mut hits = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hits * hitdmgarts
                / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        }
        if (self.unit.skill_index as f64) == 3.0 {
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + params) + self.unit.buff_atk_flat;
            dps = final_atk
                / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * ((1) as f64).max((-defense) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Amiya {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Amiya {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Amiya {
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
