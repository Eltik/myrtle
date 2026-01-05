//! DPS calculations for SwireAlt
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// SwireAlt operator implementation
pub struct SwireAlt {
    pub unit: OperatorUnit,
}

impl SwireAlt {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Creates a new SwireAlt operator
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
    /// if self.talent_dmg and self.elite > 0:
    /// atkbuff = self.talent1_params[3] * self.talent1_params[2]
    /// if self.module == 2: atkbuff += 0.2
    ///
    /// atkcycle = (self.atk_interval/(self.attack_speed/100))
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// if self.skill == 1:
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100))
    /// if not self.skill_dmg: dps = dps * (3/atkcycle-1) /(3/atkcycle)
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// skilldmg = np.fmax(final_atk * skill_scale - defense, final_atk * skill_scale * 0.05)
    /// if self.skill_dmg: skilldmg *= 2
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100))
    /// dps = dps * (3/atkcycle-1) /(3/atkcycle) + skilldmg / 3
    /// if self.skill in [0,3]:
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * (1 + self.skill/3)
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

        let mut final_atk: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;

        atkbuff = 0.0;
        if self.unit.talent_damage && (self.unit.elite as f64) > 0.0 {
            atkbuff = self.unit.talent1_parameters.get(3).copied().unwrap_or(0.0)
                * self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
            if (self.unit.module_index as f64) == 2.0 {
                atkbuff += 0.2;
            }
        }
        let mut atkcycle = ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
        if (self.unit.skill_index as f64) == 1.0 {
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
            if !self.unit.skill_damage {
                dps = dps * (3.0 / atkcycle - 1.0) / (3.0 / atkcycle);
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            skilldmg = ((final_atk * skill_scale - defense) as f64)
                .max((final_atk * skill_scale * 0.05) as f64);
            if self.unit.skill_damage {
                skilldmg *= 2.0;
            }
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0));
            dps = dps * (3.0 / atkcycle - 1.0) / (3.0 / atkcycle) + skilldmg / 3.0;
        }
        if [0.0, 3.0].contains(&((self.unit.skill_index as f64) as f64)) {
            dps = hitdmg / ((self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0))
                * (1.0 + (self.unit.skill_index as f64) / 3.0);
        }
        return dps;
    }
}

impl std::ops::Deref for SwireAlt {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for SwireAlt {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for SwireAlt {
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
