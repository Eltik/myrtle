//! DPS calculations for NearlAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// NearlAlter operator implementation
pub struct NearlAlter {
    pub unit: OperatorUnit,
}

impl NearlAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new NearlAlter operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
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
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    /// aspd = 30 if self.module == 2 and self.module_dmg else 0
    /// def_shred = self.talent2_params[0] if self.elite == 2 else 0
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * min(self.skill,1)) + self.buff_atk_flat
    /// if self.skill == 1: aspd += self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense * (1 - def_shred), final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// return dps
    #[allow(
        unused_variables,
        unused_mut,
        unused_assignments,
        unused_parens,
        clippy::excessive_precision,
        clippy::unnecessary_cast,
        clippy::collapsible_if,
        clippy::double_parens
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut atk_scale =
            if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
                1.15
            } else {
                1.0
            };
        let mut aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage
        {
            30.0
        } else {
            0.0
        };
        let mut def_shred = if ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters[0]
        } else {
            0.0
        };
        let mut final_atk = self.unit.atk
            * (1.0
                + self.unit.buff_atk
                + self.unit.skill_parameters[0]
                    * ((self.unit.skill_index as f64) as f64).min((1) as f64))
            + self.unit.buff_atk_flat;
        // UNTRANSLATED: if self.skill == 1: aspd += self.skill_params[1]
        let mut hitdmg = ((final_atk * atk_scale - defense * (1.0 - def_shred)) as f64)
            .max((final_atk * atk_scale * 0.05) as f64);
        
        hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0
    }
}

impl std::ops::Deref for NearlAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for NearlAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
