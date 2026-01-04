//! DPS calculations for ExusiaiAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ExusiaiAlter operator implementation
pub struct ExusiaiAlter {
    pub unit: OperatorUnit,
}

impl ExusiaiAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new ExusiaiAlter operator
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
    /// atkbuff = 2 * self.talent2_params[0] if self.elite > 1 else 0
    /// explosion_prob = min(self.talent1_params[1:])
    /// explosion_scale = max(self.talent1_params)
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// explosionhit = np.fmax(final_atk * explosion_scale - defense, final_atk * explosion_scale * 0.05) * self.skill
    /// dps = (hitdmg + explosionhit * explosion_prob * self.targets) / self.atk_interval * (self.attack_speed) / 100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[0]
    /// aspd = 70 if self.skill_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// explosionhit = np.fmax(final_atk * explosion_scale - defense, final_atk * explosion_scale * 0.05)
    /// dps =  (hitdmg + explosionhit * explosion_prob * self.targets) / 0.6 * (self.attack_speed + aspd) / 100
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[5]
    /// skill_scale = self.skill_params[3]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// explosionhit = np.fmax(final_atk * explosion_scale - defense, final_atk * explosion_scale * 0.05)
    /// dps =  5 * (hitdmg + explosionhit * explosion_prob * self.targets) / self.atk_interval * (self.attack_speed) / 100
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

        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut explosionhit: f64 = 0.0;

        atkbuff = if ((self.unit.elite as f64) as f64) > 1.0 {
            2.0 * self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut explosion_prob = self.unit.talent1_parameters[1..]
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let mut explosion_scale = self
            .unit
            .talent1_parameters
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            explosionhit = ((final_atk * explosion_scale - defense) as f64)
                .max((final_atk * explosion_scale * 0.05) as f64)
                * (self.unit.skill_index as f64);
            dps = (hitdmg + explosionhit * explosion_prob * (self.unit.targets as f64))
                / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            aspd = if self.unit.skill_damage { 70.0 } else { 0.0 };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            explosionhit = ((final_atk * explosion_scale - defense) as f64)
                .max((final_atk * explosion_scale * 0.05) as f64);
            dps = (hitdmg + explosionhit * explosion_prob * (self.unit.targets as f64)) / 0.6
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.get(5).copied().unwrap_or(0.0);
            skill_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            explosionhit = ((final_atk * explosion_scale - defense) as f64)
                .max((final_atk * explosion_scale * 0.05) as f64);
            dps = 5.0 * (hitdmg + explosionhit * explosion_prob * (self.unit.targets as f64))
                / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed)
                / 100.0;
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 1:
    /// return(self.skill_dps(defense,res) * 8 * (self.atk_interval/(self.attack_speed/100)))
    /// elif self.skill == 2:
    /// aspd = 70 if self.skill_dmg else 0
    /// ammo = 40 if self.skill_dmg else 35
    /// return(self.skill_dps(defense,res) * ammo * (0.6/((self.attack_speed+aspd)/100)))
    /// elif self.skill == 3:
    /// return(self.skill_dps(defense,res) * 10 * (self.atk_interval/(self.attack_speed/100)))
    /// else:
    /// return(super().total_dmg(defense,res))
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
    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        if (self.unit.skill_index as f64) == 1.0 {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * 8 * (self.atk_interval/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else if (self.unit.skill_index as f64) == 2.0 {
            aspd = if self.unit.skill_damage { 70.0 } else { 0.0 };
            let mut ammo = if self.unit.skill_damage { 40.0 } else { 35.0 };
            // UNTRANSLATED: return(self.skill_dps(defense,res) * ammo * (0.6/((self.attack_speed+aspd)/100))) - method calls need manual implementation
            0.0 // placeholder
        } else if (self.unit.skill_index as f64) == 3.0 {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * 10 * (self.atk_interval/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for ExusiaiAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ExusiaiAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
