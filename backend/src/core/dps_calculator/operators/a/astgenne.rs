//! DPS calculations for Astgenne
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Astgenne operator implementation
pub struct Astgenne {
    pub unit: OperatorUnit,
}

impl Astgenne {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2];

    /// Creates a new Astgenne operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// aspd = self.talent1_params[0] * self.talent1_params[2] if self.talent_dmg and self.elite > 0 else 0
    /// targetscaling = [0,1,2,3,4] if self.module == 2 else [0, 1, 1.85, 1.85+0.85**2, 1.85+0.85**2+0.85**3]
    /// if self.elite < 2: targetscaling = [0, 1, 1.85, 1.85+0.85**2, 1.85+0.85**2]
    /// targets = min(4, self.targets)
    /// ####the actual skills
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05) * targetscaling[targets]
    /// skill_targetscaling = [0,1,4,6,8] if self.module == 2 else [0, 1, 2 * 1.85, 2*(1.85+0.85**2), 2*(1.85+0.85**2+0.85**3)]
    /// skilldmg = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05) * skill_targetscaling[targets]
    /// if self.skill == 0: skilldmg = hitdmg
    /// sp_cost = sp_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed+aspd)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1 and self.skill == 1:
    /// if self.skill_params[4] > 1:
    /// avghit = (skilldmg + (atks_per_skillactivation - 1) * hitdmg) / atks_per_skillactivation
    /// else:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval*(self.attack_speed+aspd)/100
    ///
    /// if self.skill == 2:
    /// final_atk = self.atk * (1 + self.buff_atk + self.skill_params[0]) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/self.atk_interval*(self.attack_speed+aspd)/100
    /// if self.targets > 1:
    /// dps = hitdmg/self.atk_interval*(self.attack_speed+aspd)/100 * 2 * targetscaling[targets]
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

        let mut aspd: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skill_scale: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;

        aspd = if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent1_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut targetscaling = if ((self.unit.module_index as f64) as f64) == 2.0 {
            [0.0, 1.0, 2.0, 3.0, 4.0]
        } else {
            [
                0.0,
                1.0,
                1.85,
                1.85 + (0.85 as f64).powf(2 as f64),
                1.85 + (0.85 as f64).powf(2 as f64) + (0.85 as f64).powf(3 as f64),
            ]
        };
        if (self.unit.elite as f64) < 2.0 {
            targetscaling = [
                0.0,
                1.0,
                1.85,
                1.85 + (0.85 as f64).powf(2 as f64),
                1.85 + (0.85 as f64).powf(2 as f64),
            ];
        }
        let mut targets = ((4) as f64).min((self.unit.targets as f64) as f64);
        // ###the actual skills
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * targetscaling[(targets) as usize];
            let mut skill_targetscaling = if ((self.unit.module_index as f64) as f64) == 2.0 {
                [0.0, 1.0, 4.0, 6.0, 8.0]
            } else {
                [
                    0.0,
                    1.0,
                    2.0 * 1.85,
                    2.0 * (1.85 + (0.85 as f64).powf(2 as f64)),
                    2.0 * (1.85 + (0.85 as f64).powf(2 as f64) + (0.85 as f64).powf(3 as f64)),
                ]
            };
            skilldmg = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * skill_targetscaling[(targets) as usize];
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmg;
            }
            sp_cost = sp_cost / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 && (self.unit.skill_index as f64) == 1.0 {
                if self.unit.skill_parameters[4] > 1.0 {
                    avghit = (skilldmg + (atks_per_skillactivation - 1.0) * hitdmg)
                        / atks_per_skillactivation;
                } else {
                    avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                        / (((atks_per_skillactivation) as f64).trunc() + 1.0);
                }
            }
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            final_atk = self.unit.atk
                * (1.0
                    + self.unit.buff_atk
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0))
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                    / 100.0
                    * 2.0
                    * targetscaling[(targets) as usize];
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Astgenne {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Astgenne {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
