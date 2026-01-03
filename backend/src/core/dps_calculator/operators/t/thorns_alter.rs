//! DPS calculations for ThornsAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ThornsAlter operator implementation
pub struct ThornsAlter {
    pub unit: OperatorUnit,
}

impl ThornsAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new ThornsAlter operator
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
    /// atkbuff = min(self.talent1_params)
    /// extra_duration = max(self.talent1_params)
    /// aspd = self.talent2_params[0] if self.elite > 2 else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
    /// if self.skill != 0 and not self.trait_dmg: dps *= 0
    ///
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[4]
    /// hitdmgarts = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps += hitdmgarts * self.targets
    /// if self.skill == 3:
    /// duration = self.skill_params[2] + extra_duration
    /// shred_base = self.skill_params[10]
    /// shred_step = self.skill_params[12]
    /// skill_scale_base = self.skill_params[4]
    /// skill_scale_step = self.skill_params[6]
    /// dps = res * 0
    /// for i in range(int(duration)):
    /// newdef = defense * (1 + shred_base + min(i,15) * shred_step)
    /// newres = res * (1 + shred_base + min(i,15) * shred_step)
    /// skill_scale = skill_scale_base + skill_scale_step * min(i,15)
    /// dps += np.fmax(final_atk * skill_scale * (1-newres/100), final_atk * skill_scale * 0.05) * self.targets
    /// if self.trait_dmg: dps += np.fmax(final_atk - newdef, final_atk * 0.05) / self.atk_interval * (self.attack_speed + aspd) / 100
    /// dps = dps / duration
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        atkbuff = self
            .unit
            .talent1_parameters
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let mut extra_duration = self
            .unit
            .talent1_parameters
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        aspd = if ((self.unit.elite as f64) as f64) > 2.0 {
            self.unit.talent2_parameters[0]
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        // UNTRANSLATED: if self.skill != 0 and not self.trait_dmg: dps *= 0
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters[4];
            hitdmgarts = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps += hitdmgarts * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut duration = self.unit.skill_parameters[2] + extra_duration;
            let mut shred_base = self.unit.skill_parameters[10];
            let mut shred_step = self.unit.skill_parameters[12];
            let mut skill_scale_base = self.unit.skill_parameters[4];
            let mut skill_scale_step = self.unit.skill_parameters[6];
            dps = res * 0.0;
            // UNTRANSLATED FOR LOOP: for i in range(int(duration)):
            // TODO: Implement loop logic manually
            dps = dps / duration;
        }
        return dps;
    }
}

impl std::ops::Deref for ThornsAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ThornsAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
