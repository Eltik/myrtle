//! DPS calculations for Windscoot
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Windscoot operator implementation
pub struct Windscoot {
    pub unit: OperatorUnit,
}

impl Windscoot {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Windscoot operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// atkbuff = 2 if self.trait_dmg else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// extrahit = np.fmax(final_atk * self.talent1_params[0] - defense, final_atk * 0.05) if self.elite > 0 and self.trait_dmg and self.talent_dmg else 0
    /// if self.skill == 0:
    /// return res * 0
    ///
    /// if self.skill == 1:
    /// aspd = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = (hitdmg+extrahit)/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = (hitdmg+extrahit)/self.atk_interval * self.attack_speed/100 * min(self.targets, 2)
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

        let mut atkbuff: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;

        atkbuff = if self.unit.trait_damage { 2.0 } else { 1.0 };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut extrahit = if ((self.unit.elite as f64) as f64) > 0.0
            && self.unit.trait_damage
            && self.unit.talent_damage
        {
            ((final_atk * self.unit.talent1_parameters.first().copied().unwrap_or(0.0) - defense)
                as f64)
                .max((final_atk * 0.05) as f64)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            return res * 0.0;
        }
        if (self.unit.skill_index as f64) == 1.0 {
            aspd = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = (hitdmg + extrahit) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = (hitdmg + extrahit) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Windscoot {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Windscoot {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
