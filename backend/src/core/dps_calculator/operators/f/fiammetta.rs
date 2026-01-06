//! DPS calculations for Fiammetta
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Fiammetta operator implementation
pub struct Fiammetta {
    pub unit: OperatorUnit,
}

impl Fiammetta {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("talent", "w/o vigor", true, &[], &[], 0, 0),
        ("skill", "central hit", false, &[3], &[], 0, 0),
        ("module", "blockedTarget", false, &[], &[1], 0, 0),
    ];

    /// Creates a new Fiammetta operator
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
    /// atkbuff = 0
    /// aspd = 0
    /// atk_scale = 1.1 if self.module == 1 and self.module_dmg else 1
    /// def_shred = 100 if self.module == 2 else 0
    /// newdef = np.fmax(0, defense - def_shred)
    ///
    /// if self.module == 2:
    /// if self.module_lvl == 2: aspd += 5
    /// if self.module_lvl == 3: aspd += 10
    /// if self.talent_dmg and self.talent2_dmg:
    /// atkbuff += self.talent1_params[-2]
    /// elif self.talent_dmg:
    /// atkbuff += self.talent1_params[-4] #lets hope this works lol
    ///
    /// if self.skill < 2:
    /// atkbuff += self.skill_params[0] * self.skill
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * self.targets
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[3]
    /// if self.skill_dmg:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * atk_scale * skill_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed+aspd)/100)) * self.targets
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

        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut final_atk: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;

        atkbuff = 0.0;
        aspd = 0.0;
        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.1
        } else {
            1.0
        };
        let mut def_shred = if ((self.unit.module_index as f64) as f64) == 2.0 {
            100.0
        } else {
            0.0
        };
        let mut newdef = ((0) as f64).max((defense - def_shred) as f64);
        if (self.unit.module_index as f64) == 2.0 {
            if (self.unit.module_level as f64) == 2.0 {
                aspd += 5.0;
            }
            if (self.unit.module_level as f64) == 3.0 {
                aspd += 10.0;
            }
        }
        if self.unit.talent_damage && self.unit.talent2_damage {
            atkbuff += self.unit.talent1_parameters
                [(self.unit.talent1_parameters.len() as isize - 2.0 as isize) as usize];
        } else if self.unit.talent_damage {
            atkbuff += self.unit.talent1_parameters
                [(self.unit.talent1_parameters.len() as isize - 4.0 as isize) as usize];
        }
        if (self.unit.skill_index as f64) < 2.0 {
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            if self.unit.skill_damage {
                skill_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            }
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale * skill_scale - newdef) as f64)
                .max((final_atk * atk_scale * skill_scale * 0.05) as f64);
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Fiammetta {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Fiammetta {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Fiammetta {
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
