//! DPS calculations for ExecutorAlter
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// ExecutorAlter operator implementation
pub struct ExecutorAlter {
    pub unit: OperatorUnit,
}

impl ExecutorAlter {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new ExecutorAlter operator
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
    /// crate = self.talent1_params[0] + self.talent1_params[1] * self.ammo if self.elite > 0 and self.skill != 0 else 0
    /// try: critdefignore = self.talent1_params[2]
    /// except: critdefignore = 0
    /// crate = min(crate, 1)
    /// aspd = 12 if self.module == 2 and self.module_dmg else 0
    /// modatkbuff = 0.04 * self.module_lvl if self.module == 2 and self.module_lvl > 1 else 0
    ///
    /// atkbuff = self.skill_params[0] if self.skill > 0 else 0
    /// if self.skill < 2:
    /// defignore = self.skill_params[1] if self.skill == 1 else 0
    /// newdef = np.fmax(0, defense - defignore)
    /// critdef =np.fmax(0, defense - defignore - critdefignore)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + self.skill * modatkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - newdef, final_atk * 0.05)
    /// critdmg =  np.fmax(final_atk - newdef, final_atk * 0.05) + np.fmax(final_atk - critdef, final_atk * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    ///
    /// if self.skill == 2:
    /// critdef = np.fmax(0, defense - critdefignore)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + modatkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg =  np.fmax(final_atk - defense, final_atk * 0.05) + np.fmax(final_atk - critdef, final_atk * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/self.atk_interval * (self.attack_speed+aspd)/100 * self.targets
    ///
    /// if self.skill == 3:
    /// atkbuff += self.ammo * self.skill_params[1]
    /// critdef = np.fmax(0, defense - critdefignore)
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk + modatkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// critdmg =  np.fmax(final_atk - defense, final_atk * 0.05) + np.fmax(final_atk - critdef, final_atk * 0.05)
    /// avgdmg = crate * critdmg + (1-crate) * hitdmg
    /// dps = avgdmg/1.8 * (self.attack_speed+aspd)/100 * self.targets
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut final_atk: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut critdefignore: f64 = 0.0;
        let mut critdef: f64 = 0.0;
        let mut modatkbuff: f64 = 0.0;
        let mut critdmg: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;

        let mut crit_rate = if ((self.unit.elite as f64) as f64) > 0.0
            && ((self.unit.skill_index as f64) as f64) != 0.0
        {
            self.unit.talent1_parameters[0] + self.unit.talent1_parameters[1] * 1.0 /* 1.0 /* self.ammo - needs manual implementation */ - needs manual implementation */
        } else {
            0.0
        };
        critdefignore = 0.0; // try-except fallback
        crit_rate = ((crit_rate) as f64).min((1) as f64);
        aspd = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            12.0
        } else {
            0.0
        };
        modatkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            0.04 * ((self.unit.module_level as f64) as f64)
        } else {
            0.0
        };
        atkbuff = if ((self.unit.skill_index as f64) as f64) > 0.0 {
            self.unit.skill_parameters[0]
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            let mut defignore = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters[1]
            } else {
                0.0
            };
            let mut newdef = ((0) as f64).max((defense - defignore) as f64);
            critdef = ((0) as f64).max((defense - defignore - critdefignore) as f64);
            final_atk = self.unit.atk
                * (1.0
                    + atkbuff
                    + self.unit.buff_atk
                    + (self.unit.skill_index as f64) * modatkbuff)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk - newdef) as f64).max((final_atk * 0.05) as f64)
                + ((final_atk - critdef) as f64).max((final_atk * 0.05) as f64);
            avgdmg = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
            dps = avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
            critdef = ((0) as f64).max((defense - critdefignore) as f64);
            final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk + modatkbuff)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64)
                + ((final_atk - critdef) as f64).max((final_atk * 0.05) as f64);
            avgdmg = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
            dps = avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0
                * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atkbuff +=
                1.0 /* self.ammo - needs manual implementation */ * self.unit.skill_parameters[1];
            critdef = ((0) as f64).max((defense - critdefignore) as f64);
            final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk + modatkbuff)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            critdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64)
                + ((final_atk - critdef) as f64).max((final_atk * 0.05) as f64);
            avgdmg = crit_rate * critdmg + (1.0 - crit_rate) * hitdmg;
            dps =
                avgdmg / 1.8 * (self.unit.attack_speed + aspd) / 100.0 * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for ExecutorAlter {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for ExecutorAlter {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
