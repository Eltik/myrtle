//! DPS calculations for SpecterAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// SpecterAlter operator implementation
pub struct SpecterAlter {
    pub unit: OperatorUnit,
}

impl SpecterAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new SpecterAlter operator
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
    /// atkbuff = self.skill_params[0] * min(self.skill,1) if self.trait_dmg else 0
    /// if not self.trait_dmg and self.module == 1: atkbuff += 0.15
    ///
    /// if not self.trait_dmg:
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// doll_scale = self.talent1_params[1]
    /// hitdmg = np.fmax(final_atk * doll_scale * (1-res/100), final_atk * doll_scale * 0.05)
    /// return hitdmg
    ///
    /// if self.skill < 2:
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+self.skill_params[1])/100
    /// if self.skill == 3:
    /// dmgbonus = 1 + self.skill_params[2]
    /// if not self.skill_dmg: dmgbonus = 1
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk  - defense, final_atk * 0.05) * dmgbonus
    /// dps = dps = hitdmg/2.2 * self.attack_speed/100 * min(self.targets,2)
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

        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atkbuff = if self.unit.trait_damage {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                * (((self.unit.skill_index as f64) as f64) as f64).min((1) as f64)
        } else {
            0.0
        };
        if !self.unit.trait_damage && (self.unit.module_index as f64) == 1.0 {
            atkbuff += 0.15;
        }
        if !self.unit.trait_damage {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut doll_scale = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * doll_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * doll_scale * 0.05) as f64);
            return hitdmg;
        }
        if (self.unit.skill_index as f64) < 2.0 {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed
                    + self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut dmgbonus = 1.0 + self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            if !self.unit.skill_damage {
                dmgbonus = 1.0;
            }
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64) * dmgbonus;
            dps = hitdmg / 2.2 * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for SpecterAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for SpecterAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
