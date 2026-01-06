//! DPS calculations for Gladiia
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Gladiia operator implementation
pub struct Gladiia {
    pub unit: OperatorUnit,
}

impl Gladiia {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] =
        &[("talent2", "vsHeavy", true, &[], &[], 0, 0)];

    /// Creates a new Gladiia operator
    #[allow(unused_parens)]
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atk_scale = min(self.talent2_params) if self.elite == 2 and self.talent2_dmg else 1
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * skill_scale - defense, final_atk* atk_scale * skill_scale * 0.05)
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg if self.skill == 1 else hitdmg
    /// if atks_per_skillactivation > 1 and self.skill == 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval * self.attack_speed/100
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[2]
    /// hitdmg = np.fmax(final_atk * skill_scale * atk_scale - defense, final_atk * skill_scale * atk_scale * 0.05)
    /// dps = hitdmg/2.7 * self.attack_speed/100 * min(self.targets,2)
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[0]
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale * (1-res/100), final_atk * atk_scale * skill_scale * 0.05)
    /// dps = hitdmg/1.5 * self.targets
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

        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atk_scale = if ((self.unit.elite as f64) as f64) == 2.0 && self.unit.talent2_damage {
            self.unit
                .talent2_parameters
                .iter()
                .cloned()
                .fold(f64::INFINITY, f64::min)
        } else {
            1.0
        };
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale * skill_scale - defense) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                skilldmg
            } else {
                hitdmg
            };
            if atks_per_skillactivation > 1.0 && (self.unit.skill_index as f64) == 1.0 {
                if self.unit.skill_parameters[2] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * skill_scale * atk_scale - defense) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            dps = hitdmg / 2.7 * self.unit.attack_speed / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            hitdmg = ((final_atk * atk_scale * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = hitdmg / 1.5 * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Gladiia {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Gladiia {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Gladiia {
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
