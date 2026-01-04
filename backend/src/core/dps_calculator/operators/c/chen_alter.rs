//! DPS calculations for ChenAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ChenAlter operator implementation
pub struct ChenAlter {
    pub unit: OperatorUnit,
}

impl ChenAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new ChenAlter operator
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
    /// dps = 0
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// aspd = 8 if self.elite == 2 else 0 #im not going to include the water buff for now
    /// if self.module == 2:
    /// if self.module_lvl == 2: atkbuff += 0.1
    /// if self.module_lvl == 3:
    /// atkbuff += 0.28
    /// aspd = 20
    ///
    /// atk_scale = 1.6 if self.module == 1 else 1.5
    /// if self.skill == 0 and not self.trait_dmg: atk_scale = 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk* atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    /// if self.skill == 3:
    /// def_shred = self.skill_params[2] * (-1)
    /// if self.shreds[0] < 1 and self.shreds[0] > 0:
    /// defense = defense / self.shreds[0]
    /// newdefense = np.fmax(0, defense- def_shred)
    /// if self.shreds[0] < 1 and self.shreds[0] > 0:
    /// newdefense *= self.shreds[0]
    /// hitdmg = np.fmax(final_atk * atk_scale - newdefense, final_atk* atk_scale * 0.05)
    ///
    /// dps = 2 * hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
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
        let mut atkbuff: f64 = 0.0;
        let mut newdefense: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut defense: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;

        dps = 0.0;
        atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        aspd = if ((self.unit.elite as f64) as f64) == 2.0 {
            8.0
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 2.0 {
            // UNTRANSLATED: if self.module_lvl == 2: atkbuff += 0.1
            if (self.unit.module_level as f64) == 3.0 {
                atkbuff += 0.28;
                aspd = 20.0;
            }
        }
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.6
        } else {
            1.5
        };
        if (self.unit.skill_index as f64) == 0.0 && !self.unit.trait_damage {
            atk_scale = 1.0;
        }
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut def_shred = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0) * (-1.0);
            if self.unit.shreds[0] < 1.0 && self.unit.shreds[0] > 0.0 {
                defense = defense / self.unit.shreds.first().copied().unwrap_or(0.0);
            }
            newdefense = ((0) as f64).max((defense - def_shred) as f64);
            if self.unit.shreds[0] < 1.0 && self.unit.shreds[0] > 0.0 {
                newdefense *= self.unit.shreds.first().copied().unwrap_or(0.0);
            }
            hitdmg = ((final_atk * atk_scale - newdefense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = 2.0 * hitdmg / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 3:
    /// ammo = 16
    /// save_rate = self.talent1_params[0]
    /// ammo = ammo/(1-save_rate)
    /// return(self.skill_dps(defense,res) * ammo * (self.atk_interval/(self.attack_speed/100)))
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

        let mut atk_interval: f64 = 0.0;

        if (self.unit.skill_index as f64) == 3.0 {
            let mut ammo = 16.0;
            let mut save_rate = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
            ammo = ammo / (1.0 - save_rate);
            // UNTRANSLATED: return(self.skill_dps(defense,res) * ammo * (self.atk_interval/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(super().total_dmg(defense,res)) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for ChenAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ChenAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
