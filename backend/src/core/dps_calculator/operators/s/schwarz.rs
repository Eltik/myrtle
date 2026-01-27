//! DPS calculations for Schwarz
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Schwarz operator implementation
pub struct Schwarz {
    pub unit: OperatorUnit,
}

impl Schwarz {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "minDefshred", true, &[3], &[], 1, 0),
        ("talent2", "w/o2ndSniper", true, &[], &[], 2, 0),
        ("module", "directFront", false, &[3], &[2], 0, 0),
    ];

    /// Creates a new Schwarz operator
    #[allow(unused_parens)]
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
    /// atkbuff = 0
    /// atk_scale = 1
    /// if self.talent2_dmg:
    /// atkbuff += self.talent2_params[0]
    /// crate = 0.2
    /// cdmg = 1.6
    /// defshred = 0.1 * self.elite
    /// if self.module == 2:
    /// cdmg += 0.05 * (self.module_lvl -1)
    /// if self.module_lvl > 1: defshred = 0.25
    /// newdef = defense * (1-defshred)
    /// if self.module == 2 and self.module_dmg:
    /// atk_scale = 1.05
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// crate2 = self.skill_params[1]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// if self.talent_dmg: hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// else: hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * cdmg - newdef, final_atk * atk_scale * cdmg * 0.05)
    /// if self.talent_dmg: skilldmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05)
    /// else: skilldmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk * atk_scale * skill_scale * 0.05)
    /// skillcrit = np.fmax(final_atk * atk_scale * cdmg * skill_scale - newdef, final_atk * atk_scale * cdmg* skill_scale * 0.05)
    /// avghit = crate * critdmg + (1-crate) * hitdmg
    /// avgskill = crate2 * skillcrit + (1-crate2) * skilldmg
    /// sp_cost = self.skill_cost
    /// avgphys = (sp_cost * avghit + avgskill) / (sp_cost + 1) if self.skill == 1 else avghit
    /// dps = avgphys/(self.atk_interval/(self.attack_speed/100))
    /// if self.skill == 2:
    /// crate = self.skill_params[1]
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// if self.talent_dmg: hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// else: hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// critdmg = np.fmax(final_atk * atk_scale * cdmg - newdef, final_atk * atk_scale * cdmg * 0.05)
    /// avghit = crate * critdmg + (1-crate) * hitdmg
    /// dps = avghit/(self.atk_interval/(self.attack_speed/100))
    /// if self.skill == 3:
    /// atk_interval = self.atk_interval + 0.4
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// critdmg = np.fmax(final_atk * atk_scale * cdmg - newdef, final_atk * atk_scale * cdmg * 0.05)
    /// dps = critdmg/(atk_interval/(self.attack_speed/100))
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
        clippy::eq_op,
        clippy::get_first
    )]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let mut defense = enemy.defense;
        let mut res = enemy.res;

        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut avgphys: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut cdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        atkbuff = 0.0;
        atk_scale = 1.0;
        if self.unit.talent2_damage {
            atkbuff += self.unit.talent2_parameters.get(0).copied().unwrap_or(0.0);
        }
        let mut crit_rate = 0.2;
        cdmg = 1.6;
        defshred = 0.1 * (self.unit.elite as f64);
        if (self.unit.module_index as f64) == 2.0 {
            cdmg += 0.05 * ((self.unit.module_level as f64) - 1.0);
            if (self.unit.module_level as f64) > 1.0 {
                defshred = 0.25;
            }
        }
        let mut newdef = defense * (1.0 - defshred);
        if (self.unit.module_index as f64) == 2.0 && self.unit.module_damage {
            atk_scale = 1.05;
        }
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            let mut crate2 = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            if self.unit.talent_damage {
                hitdmg = ((final_atk * atk_scale - newdef) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64);
            } else {
                hitdmg = ((final_atk * atk_scale - defense) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64);
            }
            critdmg = ((final_atk * atk_scale * cdmg - newdef) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            if self.unit.talent_damage {
                skilldmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                    .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            } else {
                skilldmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                    .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            }
            let mut skillcrit = ((final_atk * atk_scale * cdmg * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * cdmg * skill_scale * 0.05) as f64);
            avghit = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
            let mut avgskill = crate2 * skillcrit + (1.0 - crate2) * skilldmg;
            sp_cost = (self.unit.skill_cost as f64);
            avgphys = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                (sp_cost * avghit + avgskill) / (sp_cost + 1.0)
            } else {
                avghit
            };
            dps = avgphys / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            crit_rate = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            if self.unit.talent_damage {
                hitdmg = ((final_atk * atk_scale - newdef) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64);
            } else {
                hitdmg = ((final_atk * atk_scale - defense) as f64)
                    .max((final_atk * atk_scale * 0.05) as f64);
            }
            critdmg = ((final_atk * atk_scale * cdmg - newdef) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            avghit = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
            dps = avghit / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = (self.unit.attack_interval as f64) + 0.4;
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            critdmg = ((final_atk * atk_scale * cdmg - newdef) as f64)
                .max((final_atk * atk_scale * cdmg * 0.05) as f64);
            dps = critdmg / (atk_interval / (self.unit.attack_speed / 100.0));
        }
        return dps;
    }
}

impl std::ops::Deref for Schwarz {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Schwarz {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Schwarz {
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
