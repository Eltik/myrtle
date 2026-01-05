//! DPS calculations for YatoAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// YatoAlter operator implementation
pub struct YatoAlter {
    pub unit: OperatorUnit,
}

impl YatoAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new YatoAlter operator
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
    /// extra_arts = self.talent1_params[0]
    /// atkbuff = self.talent2_params[0] if self.elite == 2 and (self.skill != 0 or self.talent2_dmg) else 0
    /// try: atkbuff += self.talent2_params[2]
    /// except: pass
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// aspd = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * extra_arts * (1-res/100), final_atk * extra_arts * 0.05)
    /// dps = (hitdmg+hitdmgarts)/self.atk_interval * (self.attack_speed+aspd*self.skill)/100
    /// if self.skill == 1: dps *= 10 / 3
    /// if self.skill == 2:
    /// extra_arts *= self.skill_params[3]
    /// atk_scale = self.skill_params[1]
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * atk_scale * extra_arts * (1-res/100), final_atk * atk_scale * extra_arts * 0.05)
    /// dps = (hitdmg+ hitdmgarts) * self.targets * 16
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// hitdmgarts = np.fmax(final_atk * skill_scale * extra_arts * (1-res/100), final_atk * skill_scale * extra_arts * 0.05)
    /// dps = (hitdmg+ hitdmgarts) * self.targets
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
        let mut hitdmgarts: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        let mut extra_arts = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        atkbuff = if ((self.unit.elite as f64) as f64) == 2.0
            && (((self.unit.skill_index as f64) as f64) != 0.0 || self.unit.talent2_damage)
        {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        // Python: try: atkbuff += self.talent2_params[2] except: pass
        if let Some(val) = self.unit.talent2_parameters.get(2) {
            atkbuff += val;
        }
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            aspd = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * extra_arts * (1.0 - res / 100.0)) as f64)
                .max((final_atk * extra_arts * 0.05) as f64);
            dps = (hitdmg + hitdmgarts) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd * (self.unit.skill_index as f64))
                / 100.0;
            if (self.unit.skill_index as f64) == 1.0 {
                dps *= 10.0 / 3.0;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            extra_arts *= self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            atk_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmgarts = ((final_atk * atk_scale * extra_arts * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * extra_arts * 0.05) as f64);
            dps = (hitdmg + hitdmgarts) * (self.unit.targets as f64) * 16.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            hitdmgarts = ((final_atk * skill_scale * extra_arts * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * extra_arts * 0.05) as f64);
            dps = (hitdmg + hitdmgarts) * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for YatoAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for YatoAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for YatoAlter {
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
