//! DPS calculations for Typhon
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Typhon operator implementation
pub struct Typhon {
    pub unit: OperatorUnit,
}

impl Typhon {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Typhon operator
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
    /// if self.module == 2 and self.module_dmg: atk_scale = 1.12
    /// crit_scale = self.talent2_params[0] if self.talent2_dmg and self.elite == 2 else 1
    /// def_ignore = 0 if self.elite == 0 else 5 * self.talent1_params[1]
    ///
    /// if self.skill < 2:
    /// atkbuff = self.skill_params[0] * self.skill
    /// aspd = self.skill_params[1] * self.skill
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * crit_scale - defense*(1-def_ignore), final_atk * atk_scale * crit_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense*(1-def_ignore), final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * crit_scale - defense*(1-def_ignore), final_atk * atk_scale * crit_scale * 0.05)
    /// if self.targets == 1 and self.module != 2: dps = (hitdmg+critdmg)/self.atk_interval * self.attack_speed/100
    /// else: dps = 2 * critdmg/self.atk_interval * self.attack_speed/100
    /// if self.skill == 3:
    /// self.atk_interval = 5.5
    /// hits = self.skill_params[4]
    /// atk_scale *= self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense*(1-def_ignore), final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * crit_scale - defense*(1-def_ignore), final_atk * atk_scale * crit_scale * 0.05)
    /// totaldmg = hits * hitdmg
    /// if self.talent2_dmg:
    /// totaldmg = (hits-1)*hitdmg + critdmg
    /// if self.talent2_dmg and self.module == 2 and self.module_lvl > 1:
    /// totaldmg = (hits-2)*hitdmg + 2*critdmg
    /// dps = totaldmg/self.atk_interval * self.attack_speed/100
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

        let mut critdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut totaldmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
            atk_scale = 1.12;
        }
        let mut crit_scale = if self.unit.talent2_damage && ((self.unit.elite as f64) as f64) == 2.0
        {
            self.unit.talent2_parameters[0]
        } else {
            1.0
        };
        let mut def_ignore = if ((self.unit.elite as f64) as f64) == 0.0 {
            0.0
        } else {
            5.0 * self.unit.talent1_parameters[1]
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff = self.unit.skill_parameters[0] * (self.unit.skill_index as f64);
            aspd = self.unit.skill_parameters[1] * (self.unit.skill_index as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * crit_scale - defense * (1.0 - def_ignore)) as f64)
                .max((final_atk * atk_scale * crit_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense * (1.0 - def_ignore)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * atk_scale * crit_scale - defense * (1.0 - def_ignore)) as f64)
                .max((final_atk * atk_scale * crit_scale * 0.05) as f64);
            if (self.unit.targets as f64) == 1.0 && (self.unit.module_index as f64) != 2.0 {
                dps = (hitdmg + critdmg) / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
            } else {
                dps = 2.0 * critdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed
                    / 100.0;
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            // UNTRANSLATED: self.atk_interval = 5.5
            let mut hits = self.unit.skill_parameters[4];
            atk_scale *= self.unit.skill_parameters[2];
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense * (1.0 - def_ignore)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            critdmg = ((final_atk * atk_scale * crit_scale - defense * (1.0 - def_ignore)) as f64)
                .max((final_atk * atk_scale * crit_scale * 0.05) as f64);
            totaldmg = hits * hitdmg;
            if self.unit.talent2_damage {
                totaldmg = (hits - 1.0) * hitdmg + critdmg;
            }
            if self.unit.talent2_damage
                && (self.unit.module_index as f64) == 2.0
                && (self.unit.module_level as f64) > 1.0
            {
                totaldmg = (hits - 2.0) * hitdmg + 2.0 * critdmg;
            }
            dps = totaldmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        return dps;
    }

    /// Calculates total damage (overridden from base)
    ///
    /// Original Python implementation:
    ///
    /// if self.skill == 3:
    /// ammo = self.skill_params[3]
    /// return(self.skill_dps(defense,res) * ammo * (5.5/(self.attack_speed/100)))
    /// else:
    /// return(self.skill_dps(defense,res) * self.skill_duration)
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

        if (self.unit.skill_index as f64) == 3.0 {
            let mut ammo = self.unit.skill_parameters[3];
            // UNTRANSLATED: return(self.skill_dps(defense,res) * ammo * (5.5/(self.attack_speed/100))) - method calls need manual implementation
            0.0 // placeholder
        } else {
            // UNTRANSLATED: return(self.skill_dps(defense,res) * self.skill_duration) - method calls need manual implementation
            0.0 // placeholder
        }
    }
}

impl std::ops::Deref for Typhon {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Typhon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
