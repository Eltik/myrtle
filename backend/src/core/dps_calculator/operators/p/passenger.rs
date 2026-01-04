//! DPS calculations for Passenger
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Passenger operator implementation
pub struct Passenger {
    pub unit: OperatorUnit,
}

impl Passenger {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Passenger operator
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
    /// targetscaling = [0,1,2,3,4,5] if self.module == 2 else [0, 1, 1.85, 1.85+0.85**2, 1.85+0.85**2+0.85**3, 1.85+0.85**2+0.85**3+0.85**4]
    /// if self.module == 1: targetscaling = [0, 1, 1.9, 1.9+0.9**2, 1.9+0.9**2+0.9**3, 1.9+0.9**2+0.9**3+0.9**4]
    /// targets = min(5, self.targets) if self.skill == 2 else min(4, self.targets)
    /// if self.elite < 2 and self.skill == 3: targetscaling[4] = targetscaling[3]
    ///
    /// dmg_scale = self.talent1_params[1] if self.elite > 0 and self.talent_dmg else 1
    /// sp_boost = 0
    /// atkbuff = self.talent2_params[0] if self.talent2_dmg and self.elite == 2 else 0
    /// if self.module == 1 and self.module_lvl > 1 and self.talent2_dmg:
    /// sp_boost = 0.05 + 0.1 * self.module_lvl
    ///
    /// if self.skill == 1:
    /// sp_cost = self.skill_cost/(1+sp_boost + self.sp_boost) +1.2
    /// atk_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05)
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skill = int(sp_cost/atkcycle)
    /// avghit = (hitdmg * atks_per_skill + skilldmg) / (atks_per_skill + 1)
    /// dps = avghit/(self.atk_interval/(self.attack_speed/100)) * targetscaling[targets]
    /// if self.skill in [0,2]:
    /// atkbuff += self.skill_params[2] if self.skill == 2 else 0
    /// atk_interval = self.atk_interval * (1 + self.skill_params[0] * self.skill/2)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// dps = hitdmg/(atk_interval/(self.attack_speed/100)) * targetscaling[targets]
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost/(1+sp_boost + self.sp_boost) + 1.2
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// skillhit = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * targetscaling[targets]
    /// dps += 8 * skillhit / (sp_cost)
    /// return dps*dmg_scale
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
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;

        let mut targetscaling = if ((self.unit.module_index as f64) as f64) == 2.0 {
            [0.0, 1.0, 2.0, 3.0, 4.0, 5.0]
        } else {
            [
                0.0,
                1.0,
                1.85,
                1.85 + (0.85 as f64).powf(2 as f64),
                1.85 + (0.85 as f64).powf(2 as f64) + (0.85 as f64).powf(3 as f64),
                1.85 + (0.85 as f64).powf(2 as f64)
                    + (0.85 as f64).powf(3 as f64)
                    + (0.85 as f64).powf(4 as f64),
            ]
        };
        if (self.unit.module_index as f64) == 1.0 {
            targetscaling = [
                0.0,
                1.0,
                1.9,
                1.9 + (0.9 as f64).powf(2 as f64),
                1.9 + (0.9 as f64).powf(2 as f64) + (0.9 as f64).powf(3 as f64),
                1.9 + (0.9 as f64).powf(2 as f64)
                    + (0.9 as f64).powf(3 as f64)
                    + (0.9 as f64).powf(4 as f64),
            ];
        }
        let mut targets = if ((self.unit.skill_index as f64) as f64) == 2.0 {
            ((5) as f64).min((self.unit.targets as f64) as f64)
        } else {
            ((4) as f64).min((self.unit.targets as f64) as f64)
        };
        // UNTRANSLATED: if self.elite < 2 and self.skill == 3: targetscaling[4] = targetscaling[3]
        let mut dmg_scale = if ((self.unit.elite as f64) as f64) > 0.0 && self.unit.talent_damage {
            self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            1.0
        };
        let mut sp_boost = 0.0;
        atkbuff = if self.unit.talent2_damage && ((self.unit.elite as f64) as f64) == 2.0 {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
        } else {
            0.0
        };
        if (self.unit.module_index as f64) == 1.0
            && (self.unit.module_level as f64) > 1.0
            && self.unit.talent2_damage
        {
            sp_boost = 0.05 + 0.1 * (self.unit.module_level as f64);
        }
        if (self.unit.skill_index as f64) == 1.0 {
            sp_cost = (self.unit.skill_cost as f64)
                / (1.0 + sp_boost + (self.unit.sp_boost as f64))
                + 1.2;
            atk_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skill = ((sp_cost / atkcycle) as f64).trunc();
            avghit = (hitdmg * atks_per_skill + skilldmg) / (atks_per_skill + 1.0);
            dps = avghit / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * targetscaling[(targets) as usize];
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff += if ((self.unit.skill_index as f64) as f64) == 2.0 {
                self.unit.skill_parameters.get(2).copied().unwrap_or(0.0)
            } else {
                0.0
            };
            atk_interval = (self.unit.attack_interval as f64)
                * (1.0
                    + self.unit.skill_parameters.first().copied().unwrap_or(0.0)
                        * (self.unit.skill_index as f64)
                        / 2.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (atk_interval / (self.unit.attack_speed / 100.0))
                * targetscaling[(targets) as usize];
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64)
                / (1.0 + sp_boost + (self.unit.sp_boost as f64))
                + 1.2;
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut skillhit = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * targetscaling[(targets) as usize];
            dps += 8.0 * skillhit / (sp_cost);
        }
        return dps * dmg_scale;
    }
}

impl std::ops::Deref for Passenger {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Passenger {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
