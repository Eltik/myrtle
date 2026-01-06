//! DPS calculations for Thorns
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Thorns operator implementation
pub struct Thorns {
    pub unit: OperatorUnit,
}

impl Thorns {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 3];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("trait", "rangedAtk", true, &[1], &[], 0, 0),
        ("talent", "vsRanged", false, &[], &[], 0, 0),
        ("skill", "firstActivation", true, &[3], &[], 0, 0),
        ("module", "(vsBoss)", true, &[], &[3], 0, 0),
    ];

    /// Creates a new Thorns operator
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
    ///
    /// bonus = 0.1 if self.module == 1 else 0
    /// arts_dot = 0 if self.elite < 2 else max(self.talent1_params)
    /// if not self.talent_dmg: arts_dot *= 0.5
    /// stacks = self.talent1_params[3] if self.module == 1 and self.module_lvl > 1 else 1
    /// arts_dot_dps = np.fmax(arts_dot *(1-res/100) , arts_dot * 0.05) * stacks
    ///
    /// if self.skill < 2:
    /// atk_scale = 1 if self.trait_dmg else 0.8
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0] * self.skill) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// dps = (hitdmg + bonusdmg)/self.atk_interval * self.attack_speed/100 + arts_dot_dps
    /// if self.module == 3:
    /// time_to_fallout = 1000/(dps*0.1) if self.module_dmg else 2000/(dps*0.1)
    /// if self.module_lvl == 1: dps += 6000/(time_to_fallout+10)
    /// else:
    /// fallout_dps = dps - arts_dot_dps + arts_dot
    /// dps = (fallout_dps * 10 + dps * time_to_fallout + 6000) / (10 + time_to_fallout)
    /// if self.skill == 2 and self.hits > 0:
    /// atk_scale = 0.8
    /// cooldown = self.skill_params[2]
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// if(1/self.hits < cooldown):
    /// dps = (hitdmg/cooldown + arts_dot_dps + bonusdmg/cooldown) * min(self.targets,4)
    /// if self.module == 3:
    /// time_to_fallout = 1000/(dps*0.1) / min(self.targets,4) if self.module_dmg else 2000/(dps*0.1) / min(self.targets,4)
    /// if self.module_lvl == 1: dps += 6000/(time_to_fallout+10)
    /// else:
    /// fallout_dps = dps - (arts_dot_dps + arts_dot) * min(self.targets,4)
    /// dps = (fallout_dps * 10 + dps * time_to_fallout + 6000) / (10 + time_to_fallout)
    /// else:
    /// cooldown = 1/self.hits
    /// dps = (hitdmg/cooldown + arts_dot_dps) * min(self.targets,4)
    /// if self.module == 3:
    /// time_to_fallout = 1000/(dps*0.1) / min(self.targets,4) if self.module_dmg else 2000/(dps*0.1) / min(self.targets,4)
    /// if self.module_lvl == 1: dps += 6000/(time_to_fallout+10)
    /// else:
    /// fallout_dps = dps - (arts_dot_dps + arts_dot) * min(self.targets,4)
    /// dps = (fallout_dps * 10 + dps * time_to_fallout + 6000) / (10 + time_to_fallout)
    /// elif self.skill == 2:
    /// return defense*0
    /// if self.skill == 3:
    /// bufffactor = 2 if self.skill_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk + bufffactor * self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// bonusdmg = np.fmax(final_atk * bonus *(1-res/100), final_atk * bonus * 0.05)
    /// dps = (hitdmg + bonusdmg)/self.atk_interval * (self.attack_speed + bufffactor * self.skill_params[1])/100 + arts_dot_dps
    /// if self.module == 3:
    /// time_to_fallout = 1000/(dps*0.1) if self.module_dmg else 2000/(dps*0.1)
    /// if self.module_lvl == 1: dps += 6000/(time_to_fallout+10)
    /// else:
    /// fallout_dps = dps - arts_dot_dps + arts_dot
    /// dps = (fallout_dps * 10 + dps * time_to_fallout + 6000) / (10 + time_to_fallout)
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

        let mut fallout_dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut time_to_fallout: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut cooldown: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut atk_scale: f64 = 0.0;
        let mut bonusdmg: f64 = 0.0;

        let mut bonus = if ((self.unit.module_index as f64) as f64) == 1.0 {
            0.1
        } else {
            0.0
        };
        let mut arts_dot = if ((self.unit.elite as f64) as f64) < 2.0 {
            0.0
        } else {
            self.unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max)
        };
        if !self.unit.talent_damage {
            arts_dot *= 0.5;
        }
        let mut stacks = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            self.unit.talent1_parameters.get(3).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut arts_dot_dps =
            ((arts_dot * (1.0 - res / 100.0)) as f64).max((arts_dot * 0.05) as f64) * stacks;
        if (self.unit.skill_index as f64) < 2.0 {
            atk_scale = if self.unit.trait_damage { 1.0 } else { 0.8 };
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            dps = (hitdmg + bonusdmg) / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                + arts_dot_dps;
            if (self.unit.module_index as f64) == 3.0 {
                time_to_fallout = if self.unit.module_damage {
                    1000.0 / (dps * 0.1)
                } else {
                    2000.0 / (dps * 0.1)
                };
                if (self.unit.module_level as f64) == 1.0 {
                    dps += 6000.0 / (time_to_fallout + 10.0);
                }
                // UNTRANSLATED ELSE (no matching if): else:
                fallout_dps = dps - arts_dot_dps + arts_dot;
                dps = (fallout_dps * 10.0 + dps * time_to_fallout + 6000.0)
                    / (10.0 + time_to_fallout);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 && 0.0 /* self.hits - defaults to 0 */ > 0.0 {
            atk_scale = 0.8;
            cooldown = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.get(0).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            // UNTRANSLATED: if(1/self.hits < cooldown):
            dps = (hitdmg / cooldown + arts_dot_dps + bonusdmg / cooldown)
                * ((self.unit.targets as f64) as f64).min((4) as f64);
            if (self.unit.module_index as f64) == 3.0 {
                time_to_fallout = if self.unit.module_damage {
                    1000.0 / (dps * 0.1) / ((self.unit.targets as f64) as f64).min((4) as f64)
                } else {
                    2000.0 / (dps * 0.1) / ((self.unit.targets as f64) as f64).min((4) as f64)
                };
                if (self.unit.module_level as f64) == 1.0 {
                    dps += 6000.0 / (time_to_fallout + 10.0);
                }
                // UNTRANSLATED ELSE (no matching if): else:
                fallout_dps = dps
                    - (arts_dot_dps + arts_dot)
                        * ((self.unit.targets as f64) as f64).min((4) as f64);
                dps = (fallout_dps * 10.0 + dps * time_to_fallout + 6000.0)
                    / (10.0 + time_to_fallout);
            }
            // UNTRANSLATED ELSE (no matching if): else:
            cooldown = 1.0 /0.0 /* self.hits - defaults to 0 */;
            dps = (hitdmg / cooldown + arts_dot_dps)
                * ((self.unit.targets as f64) as f64).min((4) as f64);
            if (self.unit.module_index as f64) == 3.0 {
                time_to_fallout = if self.unit.module_damage {
                    1000.0 / (dps * 0.1) / ((self.unit.targets as f64) as f64).min((4) as f64)
                } else {
                    2000.0 / (dps * 0.1) / ((self.unit.targets as f64) as f64).min((4) as f64)
                };
                if (self.unit.module_level as f64) == 1.0 {
                    dps += 6000.0 / (time_to_fallout + 10.0);
                }
                // UNTRANSLATED ELSE (no matching if): else:
                fallout_dps = dps
                    - (arts_dot_dps + arts_dot)
                        * ((self.unit.targets as f64) as f64).min((4) as f64);
                dps = (fallout_dps * 10.0 + dps * time_to_fallout + 6000.0)
                    / (10.0 + time_to_fallout);
            }
        } else if (self.unit.skill_index as f64) == 2.0 {
            return defense * 0.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut bufffactor = if self.unit.skill_damage { 2.0 } else { 1.0 };
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + bufffactor * self.unit.skill_parameters.get(0).copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            bonusdmg = ((final_atk * bonus * (1.0 - res / 100.0)) as f64)
                .max((final_atk * bonus * 0.05) as f64);
            dps = (hitdmg + bonusdmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed
                    + bufffactor * self.unit.skill_parameters.get(1).copied().unwrap_or(0.0))
                / 100.0
                + arts_dot_dps;
            if (self.unit.module_index as f64) == 3.0 {
                time_to_fallout = if self.unit.module_damage {
                    1000.0 / (dps * 0.1)
                } else {
                    2000.0 / (dps * 0.1)
                };
                if (self.unit.module_level as f64) == 1.0 {
                    dps += 6000.0 / (time_to_fallout + 10.0);
                }
                // UNTRANSLATED ELSE (no matching if): else:
                fallout_dps = dps - arts_dot_dps + arts_dot;
                dps = (fallout_dps * 10.0 + dps * time_to_fallout + 6000.0)
                    / (10.0 + time_to_fallout);
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Thorns {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Thorns {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Thorns {
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
