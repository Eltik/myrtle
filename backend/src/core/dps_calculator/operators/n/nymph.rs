//! DPS calculations for Nymph
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Nymph operator implementation
pub struct Nymph {
    pub unit: OperatorUnit,
}

impl Nymph {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Nymph operator
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
    /// talent1_scale = self.talent1_params[0] if self.talent_dmg and self.elite > 0 else 0
    /// atkbuff = self.talent2_params[0] * self.talent2_params[1] if self.elite == 2 and self.talent2_dmg else 0
    /// aspd = self.talent2_params[2] if self.module == 1 and self.module_lvl > 1 and self.talent2_dmg else 0
    /// burst_scale = 1.1 if self.module == 1 else 1
    ///
    /// if self.skill == 1:
    /// atkbuff += self.skill_params[0]
    /// necrosis_scale = self.skill_params[1]
    /// ele_scale = self.skill_params[2]
    /// final_atk = self.atk * (1+atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// eledmg = 0
    /// if self.trait_dmg and self.talent_dmg:
    /// eledmg = final_atk * ele_scale
    /// dps = (hitdmg+eledmg)/self.atk_interval * (self.attack_speed+aspd)/100
    ///
    /// if self.skill == 2:
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2
    /// atk_scale = self.skill_params[0]
    /// talent1_overwrite = self.skill_params[3]
    /// necrosis_scale = self.skill_params[1]
    /// final_atk = self.atk * (1+atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * (1-res/100), final_atk * atk_scale * 0.05) * self.targets
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 + skilldmg/sp_cost
    ///
    /// if self.skill in [0,3]:
    /// atkbuff += self.skill_params[0] if self.skill == 3 else 0
    /// aspd += self.skill_params[1] if self.skill == 3 else 0
    /// final_atk = self.atk * (1+atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)
    /// if self.trait_dmg and self.talent_dmg and self.skill == 3:
    /// hitdmg = final_atk * np.fmax(1,-res) /(1+self.buff_fragile) * burst_scale
    /// dps = hitdmg/self.atk_interval * (self.attack_speed+aspd)/100 * min(self.targets,1+self.skill/3)
    ///
    /// extra_dmg = 0
    /// if self.talent_dmg and self.trait_dmg:
    /// dmg_rate = talent1_scale
    /// if self.skill == 2:
    /// dmg_rate = talent1_overwrite
    /// extra_dmg = final_atk * dmg_rate
    ///
    /// return dps + extra_dmg
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

        let mut extra_dmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut talent1_overwrite: f64 = 0.0;
        let mut burst_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dmg_rate: f64 = 0.0;
        let mut necrosis_scale: f64 = 0.0;
        let mut ele_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut eledmg: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        let mut talent1_scale =
            if self.unit.talent_damage && ((self.unit.elite as f64) as f64) > 0.0 {
                self.unit.talent1_parameters[0]
            } else {
                0.0
            };
        atkbuff = if ((self.unit.elite as f64) as f64) == 2.0 && self.unit.talent2_damage {
            self.unit.talent2_parameters[0] * self.unit.talent2_parameters[1]
        } else {
            0.0
        };
        aspd = if ((self.unit.module_index as f64) as f64) == 1.0
            && ((self.unit.module_level as f64) as f64) > 1.0
            && self.unit.talent2_damage
        {
            self.unit.talent2_parameters[2]
        } else {
            0.0
        };
        burst_scale = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.1
        } else {
            1.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            atkbuff += self.unit.skill_parameters[0];
            necrosis_scale = self.unit.skill_parameters[1];
            ele_scale = self.unit.skill_parameters[2];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            eledmg = 0.0;
            if self.unit.trait_damage && self.unit.talent_damage {
                eledmg = final_atk * ele_scale;
            }
            dps = (hitdmg + eledmg) / (self.unit.attack_interval as f64)
                * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            atk_scale = self.unit.skill_parameters[0];
            talent1_overwrite = self.unit.skill_parameters[3];
            necrosis_scale = self.unit.skill_parameters[1];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64)
                * (self.unit.targets as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                + skilldmg / sp_cost;
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff += if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters[0]
            } else {
                0.0
            };
            aspd += if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters[1]
            } else {
                0.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64);
            if self.unit.trait_damage
                && self.unit.talent_damage
                && (self.unit.skill_index as f64) == 3.0
            {
                hitdmg = final_atk * ((1) as f64).max((-res) as f64)
                    / (1.0 + self.unit.buff_fragile)
                    * burst_scale;
            }
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * ((self.unit.targets as f64) as f64)
                    .min((1.0 + (self.unit.skill_index as f64) / 3.0) as f64);
        }
        extra_dmg = 0.0;
        if self.unit.talent_damage && self.unit.trait_damage {
            dmg_rate = talent1_scale;
            if (self.unit.skill_index as f64) == 2.0 {
                dmg_rate = talent1_overwrite;
            }
            extra_dmg = final_atk * dmg_rate;
        }
        return dps + extra_dmg;
    }
}

impl std::ops::Deref for Nymph {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Nymph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
