//! DPS calculations for Dorothy
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Dorothy operator implementation
pub struct Dorothy {
    pub unit: OperatorUnit,
}

impl Dorothy {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Dorothy operator
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
    /// atkbuff = self.talent2_params[0] * self.talent2_params[1] if self.talent2_dmg else 0
    /// cdmg = 1.2 if self.module == 2 else 1
    /// if self.module == 1 and self.module_lvl == 3 and self.module_dmg: cdmg = 1.5
    /// sp_cost = max(self.skill_cost / (1+ self.sp_boost),5)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// mine_scale = self.skill_params[1] if self.trait_dmg and self.skill > 0 else 0
    ///
    /// if self.skill == 1:
    /// defshred = 1 + self.skill_params[2]
    /// hitdmgmine = np.fmax(final_atk * mine_scale - defense * defshred, final_atk * mine_scale * 0.05) * cdmg
    /// if not self.trait_dmg or not self.skill_dmg:
    /// defshred = 1
    /// elif not self.talent_dmg:
    /// defshred = 1 + 5 / sp_cost * self.skill_params[2]  #include uptime of the debuff for auto attacks
    /// hitdmg = np.fmax(final_atk - defense * defshred, final_atk * 0.05)
    /// if self.skill in [0,2]:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgmine = np.fmax(final_atk * mine_scale - defense, final_atk * mine_scale * 0.05) * cdmg
    /// if self.skill == 3:
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmgmine = np.fmax(final_atk * mine_scale * (1-res/100), final_atk * mine_scale * 0.05) * cdmg
    /// minedps = hitdmgmine/5 if self.talent_dmg else hitdmgmine/sp_cost
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100 + minedps * self.targets
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

        let mut hitdmg: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut cdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut sp_cost: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut defshred: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmgmine: f64 = 0.0;

        atkbuff = if self.unit.talent2_damage {
            self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
                * self.unit.talent2_parameters.get(1).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        cdmg = if ((self.unit.module_index as f64) as f64) == 2.0 {
            1.2
        } else {
            1.0
        };
        if (self.unit.module_index as f64) == 1.0
            && (self.unit.module_level as f64) == 3.0
            && self.unit.module_damage
        {
            cdmg = 1.5;
        }
        sp_cost = (((self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64))) as f64)
            .max((5) as f64);
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut mine_scale =
            if self.unit.trait_damage && ((self.unit.skill_index as f64) as f64) > 0.0 {
                self.unit.skill_parameters.get(1).copied().unwrap_or(0.0)
            } else {
                0.0
            };
        if (self.unit.skill_index as f64) == 1.0 {
            defshred = 1.0 + self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            hitdmgmine = ((final_atk * mine_scale - defense * defshred) as f64)
                .max((final_atk * mine_scale * 0.05) as f64)
                * cdmg;
            if !self.unit.trait_damage || !self.unit.skill_damage {
                defshred = 1.0;
            } else if !self.unit.talent_damage {
                defshred =
                    1.0 + 5.0 / sp_cost * self.unit.skill_parameters.get(2).copied().unwrap_or(0.0);
            }
            hitdmg = ((final_atk - defense * defshred) as f64).max((final_atk * 0.05) as f64);
        }
        if [0.0, 2.0].contains(&((self.unit.skill_index as f64) as f64)) {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgmine = ((final_atk * mine_scale - defense) as f64)
                .max((final_atk * mine_scale * 0.05) as f64)
                * cdmg;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmgmine = ((final_atk * mine_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * mine_scale * 0.05) as f64)
                * cdmg;
        }
        let mut minedps = if self.unit.talent_damage {
            hitdmgmine / 5.0
        } else {
            hitdmgmine / sp_cost
        };
        dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0
            + minedps * (self.unit.targets as f64);
        return dps;
    }
}

impl std::ops::Deref for Dorothy {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Dorothy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Dorothy {
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
