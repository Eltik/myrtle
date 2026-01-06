//! DPS calculations for Muelsyse
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Muelsyse operator implementation
pub struct Muelsyse {
    pub unit: OperatorUnit,
}

impl Muelsyse {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Muelsyse operator
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
    /// atk_scale = 1.5 if self.trait_dmg else 1
    /// if self.trait_dmg and self.module == 2: atk_scale = 1.65
    /// copy_factor = 1 if self.module == 1 and self.module_lvl == 3 else 0.5 + 0.2 * self.elite
    ///
    /// atkbuff = self.skill_params[2] if self.skill == 1 else self.skill_params[1] * min(self.skill,1)
    /// aspd = self.skill_params[3] if self.skill == 1 else 0
    ///
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed + aspd)/100
    ///
    /// main = 1 if self.talent_dmg else 0
    /// clone_atk = self.cloned_op.atk * copy_factor * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// if not self.cloned_op.ranged and self.talent2_dmg: clone_atk += 250
    /// summondamage = np.fmax(clone_atk * (1-res/100), clone_atk * 0.05) if not self.cloned_op.physical else np.fmax(clone_atk - defense, clone_atk * 0.05)
    /// extra_summons = 0
    /// extra_summons_skill = 0
    /// if self.cloned_op.ranged and self.talent_dmg:
    /// extra_summons += min(4,2.5/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100)))
    /// extra_summons_skill =  min(4,2.5/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100)) * 2) if self.skill == 2 else min(4,2.5/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100)))
    /// if self.skill == 0: extra_summons_skill = extra_summons
    /// extra_summons = (50 * extra_summons + 15 * extra_summons_skill) / 65
    ///
    /// if self.skill == 3 and self.cloned_op.ranged:
    /// extra_summons = 4 if self.skill_dmg else 2
    /// dps += (main+extra_summons) * summondamage/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100))
    /// if self.skill == 2 and self.cloned_op.ranged:
    /// dps += (main+extra_summons) * summondamage/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100)) * 2
    /// elif self.skill != 3 or (self.skill == 3 and not self.cloned_op.ranged):
    /// dps += (main+extra_summons) * summondamage/(self.cloned_op.atk_interval/((self.attack_speed + aspd)/100))
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

        // Cloned operator defaults to Ela (Trapmaster)
        // Ela's ATK depends on Muelsyse's module status (Python behavior)
        let cloned_op_atk: f64 = if self.unit.module_index > 0 {
            728.0
        } else {
            668.0
        };
        let cloned_op_atk_interval: f64 = 0.85;
        let cloned_op_ranged: bool = true;
        let cloned_op_physical: bool = true;

        // Muelsyse: Get effective module level for copy_factor calculation
        // When module_index > 0 but module_level is 0 (matching failed), default to 3
        let effective_module_level: i32 = if self.unit.module_index > 0 {
            if self.unit.operator_module_level > 0 {
                self.unit.operator_module_level
            } else if self.unit.module_level > 0 {
                self.unit.module_level
            } else {
                3 // Default to max level when module is selected
            }
        } else {
            0
        };

        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut extra_summons: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atk_scale = if self.unit.trait_damage { 1.5 } else { 1.0 };
        if self.unit.trait_damage && (self.unit.module_index as f64) == 2.0 {
            atk_scale = 1.65;
        }
        let mut copy_factor = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((effective_module_level as f64) as f64) == 3.0
        {
            1.0
        } else {
            0.5 + 0.2 * ((self.unit.elite as f64) as f64)
        };
        atkbuff = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
                * (((self.unit.skill_index as f64) as f64) as f64).min((1) as f64)
        };
        aspd = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            self.unit.skill_parameters.get(3).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg =
            ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd) / 100.0;
        let mut main = if self.unit.talent_damage { 1.0 } else { 0.0 };
        let mut clone_atk = cloned_op_atk * copy_factor * (1.0 + atkbuff + self.unit.buff_atk)
            + self.unit.buff_atk_flat;
        if !cloned_op_ranged && self.unit.talent2_damage {
            clone_atk += 250.0;
        }
        let mut summondamage = if !cloned_op_physical {
            ((clone_atk * (1.0 - res / 100.0)) as f64).max((clone_atk * 0.05) as f64)
        } else {
            ((clone_atk - defense) as f64).max((clone_atk * 0.05) as f64)
        };
        extra_summons = 0.0;
        let mut extra_summons_skill = 0.0;
        if cloned_op_ranged && self.unit.talent_damage {
            extra_summons += ((4) as f64).min(
                (2.5 / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0))) as f64,
            );
            extra_summons_skill = if ((self.unit.skill_index as f64) as f64) == 2.0 {
                ((4) as f64).min(
                    (2.5 / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                        * 2.0) as f64,
                )
            } else {
                ((4) as f64).min(
                    (2.5 / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0)))
                        as f64,
                )
            };
            if (self.unit.skill_index as f64) == 0.0 {
                extra_summons_skill = extra_summons;
            }
            extra_summons = (50.0 * extra_summons + 15.0 * extra_summons_skill) / 65.0;
        }
        if (self.unit.skill_index as f64) == 3.0 && cloned_op_ranged {
            extra_summons = if self.unit.skill_damage { 4.0 } else { 2.0 };
            dps += (main + extra_summons) * summondamage
                / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 && cloned_op_ranged {
            dps += (main + extra_summons) * summondamage
                / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0))
                * 2.0;
        } else if (self.unit.skill_index as f64) != 3.0
            || ((self.unit.skill_index as f64) == 3.0 && !cloned_op_ranged)
        {
            dps += (main + extra_summons) * summondamage
                / (cloned_op_atk_interval / ((self.unit.attack_speed + aspd) / 100.0));
        }
        return dps;
    }
}

impl std::ops::Deref for Muelsyse {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Muelsyse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Muelsyse {
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
