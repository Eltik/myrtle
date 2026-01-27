//! DPS calculations for Vulpisfoglia
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Vulpisfoglia operator implementation
pub struct Vulpisfoglia {
    pub unit: OperatorUnit,
}

impl Vulpisfoglia {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "w/o Talent1", true, &[], &[], 1, 0),
        ("module", "blocking", false, &[], &[1], 0, 0),
    ];

    /// Creates a new Vulpisfoglia operator
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
    /// atkbuff = 0.08 if self.module == 1 and self.module_dmg else 0
    /// arts_scale = self.talent1_params[0] if self.elite > 0 and self.talent_dmg else 0
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk  - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * arts_scale * (1-res/100), final_atk * arts_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// if self.skill == 0: skilldmg = hitdmg
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2
    /// atkcycle = self.atk_interval/((self.attack_speed)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// if self.skill_params[2] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = (avghit+hitdmgarts)/self.atk_interval*(self.attack_speed)/100 * self.targets
    /// if self.skill == 3:
    /// atkbuff += self.skill_params[1]
    /// aspd = self.skill_params[2] / 2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgarts = np.fmax(final_atk * arts_scale * (1-res/100), final_atk * arts_scale * 0.05)
    /// dps = (hitdmg+hitdmgarts) / self.atk_interval * (self.attack_speed + aspd) / 100 * min(self.targets,2)
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
        let mut skill_scale: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut aspd: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            0.08
        } else {
            0.0
        };
        let mut arts_scale = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage {
            self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * arts_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * arts_scale * 0.05) as f64);
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmg;
            }
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                if self.unit.skill_parameters[2] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = (avghit + hitdmgarts) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff += self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            aspd = self.unit.skill_parameters.get(2).copied().unwrap_or(0.0) / 2.0;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgarts = ((final_atk * arts_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * arts_scale * 0.05) as f64);
            dps = (hitdmg + hitdmgarts) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64).min((2) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Vulpisfoglia {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Vulpisfoglia {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Vulpisfoglia {
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
