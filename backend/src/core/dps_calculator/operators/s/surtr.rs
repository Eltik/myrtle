//! DPS calculations for Surtr
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};
use super::super::ConditionalTuple;

/// Surtr operator implementation
pub struct Surtr {
    pub unit: OperatorUnit,
}

impl Surtr {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Conditionals for this operator
    /// Format: (type, name, inverted, skills, modules, min_elite, min_module_level)
    pub const CONDITIONALS: &'static [ConditionalTuple] = &[
        ("skill", "KillingHitsOnly", false, &[], &[], 0, 0),
        ("module", "NotBlocking", false, &[], &[1], 0, 0),
    ];

    /// Creates a new Surtr operator
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
    /// resignore = self.talent1_params[0]
    /// newres = np.fmax(0, res - resignore)
    /// aspd = 8 if self.module == 1 and self.module_dmg else 0
    /// if self.skill == 1:
    /// atk_scale = self.skill_params[0]
    /// hits = self.skill_cost
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// skilldmgarts = np.fmax(final_atk * atk_scale *(1-newres/100), final_atk * atk_scale * 0.05)
    /// avghit = (hits * hitdmgarts + skilldmgarts)/(hits + 1)
    /// if self.skill_dmg:
    /// avghit = skilldmgarts
    /// dps = avghit/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// if self.skill == 2:
    /// atkbuff += self.skill_params[0]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// atk_scale = self.skill_params[3]
    /// one_target_dmg = np.fmax(final_atk * atk_scale *(1-newres/100), final_atk * atk_scale * 0.05)
    /// two_target_dmg = np.fmax(final_atk * (1-newres/100), final_atk * 0.05)
    /// dps = one_target_dmg/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// if self.targets > 1:
    /// dps = 2 * two_target_dmg/(self.atk_interval/((self.attack_speed+aspd)/100))
    /// if self.skill in [0,3]:
    /// atkbuff += self.skill_params[0] * self.skill/3
    /// maxtargets = self.skill_params[6] if self.skill == 3 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *(1-newres/100), final_atk * 0.05)
    /// dps = hitdmgarts/(self.atk_interval/((self.attack_speed+aspd)/100)) * min(self.targets,maxtargets)
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

        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut newres: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = 0.0;
        let mut resignore = self.unit.talent1_parameters.get(0).copied().unwrap_or(0.0);
        newres = ((0) as f64).max((res - resignore) as f64);
        aspd = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            8.0
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) == 1.0 {
            atk_scale = self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            let mut hits = (self.unit.skill_cost as f64);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts =
                ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            let mut skilldmgarts = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            avghit = (hits * hitdmgarts + skilldmgarts) / (hits + 1.0);
            if self.unit.skill_damage {
                avghit = skilldmgarts;
            }
            dps = avghit
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            atk_scale = self.unit.skill_parameters.get(3).copied().unwrap_or(0.0);
            let mut one_target_dmg = ((final_atk * atk_scale * (1.0 - newres / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut two_target_dmg =
                ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = one_target_dmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0));
            if (self.unit.targets as f64) > 1.0 {
                dps = 2.0 * two_target_dmg
                    / ((self.unit.attack_interval as f64)
                        / ((self.unit.attack_speed + aspd) / 100.0));
            }
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            atkbuff += self.unit.skill_parameters.get(0).copied().unwrap_or(0.0)
                * (self.unit.skill_index as f64)
                / 3.0;
            let mut maxtargets = if ((self.unit.skill_index as f64) as f64) == 3.0 {
                self.unit.skill_parameters.get(6).copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts =
                ((final_atk * (1.0 - newres / 100.0)) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmgarts
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * ((self.unit.targets as f64) as f64).min((maxtargets) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Surtr {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Surtr {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Surtr {
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
