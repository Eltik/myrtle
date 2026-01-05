//! DPS calculations for Phantom
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Phantom operator implementation
pub struct Phantom {
    pub unit: OperatorUnit,
}

impl Phantom {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2, 3];

    /// Creates a new Phantom operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// if self.skill == 2:
    /// selfhit = 0
    /// clonehit = 0
    /// mainbuff = 0.1 if self.module == 2 and self.module_dmg else 0
    /// atkbuff = 0.1 if self.module == 2 and self.module_lvl > 1 and self.talent_dmg else 0
    /// rate = self.skill_params[1]
    /// count = int(self.skill_params[0])
    /// for i in range(count):
    /// atkbuff += rate
    /// final_atk = self.atk * (1 + atkbuff + mainbuff + self.buff_atk) + self.buff_atk_flat
    /// final_clone = self.drone_atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// selfhit += np.fmax(final_atk - defense, final_atk * 0.05)
    /// clonehit += np.fmax(final_clone - defense, final_clone * 0.05)
    /// dps = selfhit /self.atk_interval * self.attack_speed/100 / count
    /// if self.talent_dmg:
    /// dps += clonehit /self.drone_atk_interval * self.attack_speed/100 / count
    /// else:
    /// mainbuff = 0.1 if self.module == 2 and self.module_dmg else 0
    /// atkbuff = 0.1 if self.module == 2 and self.module_lvl > 1 and self.talent_dmg else 0
    /// final_atk = self.atk * (1 + atkbuff + mainbuff + self.buff_atk) + self.buff_atk_flat
    /// final_clone = self.drone_atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// hitdmg_clone = np.fmax(final_clone - defense, final_clone * 0.05)
    /// dps = hitdmg /self.atk_interval * self.attack_speed/100
    /// if self.talent_dmg:
    /// dps += hitdmg_clone /self.drone_atk_interval * self.attack_speed/100
    ///
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

        let mut dps: f64 = 0.0;
        let mut hitdmg_clone: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut final_clone: f64 = 0.0;
        let mut mainbuff: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut hitdmg: f64 = 0.0;

        if (self.unit.skill_index as f64) == 2.0 {
            let mut selfhit = 0.0;
            let mut clonehit = 0.0;
            mainbuff = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage
            {
                0.1
            } else {
                0.0
            };
            atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
                && ((self.unit.module_level as f64) as f64) > 1.0
                && self.unit.talent_damage
            {
                0.1
            } else {
                0.0
            };
            let mut rate = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            let mut count =
                ((self.unit.skill_parameters.first().copied().unwrap_or(0.0)) as f64).trunc();
            // Implement for loop: for i in range(count):
            for _i in 0..(count as i32) {
                let i = _i as f64;
                atkbuff += rate;
                final_atk = self.unit.atk * (1.0 + atkbuff + mainbuff + self.unit.buff_atk)
                    + self.unit.buff_atk_flat;
                final_clone = self.unit.drone_atk * (1.0 + atkbuff + self.unit.buff_atk)
                    + self.unit.buff_atk_flat;
                selfhit += ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
                clonehit += ((final_clone - defense) as f64).max((final_clone * 0.05) as f64);
            }
            dps = selfhit / (self.unit.attack_interval as f64) * self.unit.attack_speed
                / 100.0
                / count;
            if self.unit.talent_damage {
                dps += clonehit / (self.unit.drone_atk_interval as f64) * self.unit.attack_speed
                    / 100.0
                    / count;
            }
        } else {
            mainbuff = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage
            {
                0.1
            } else {
                0.0
            };
            atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
                && ((self.unit.module_level as f64) as f64) > 1.0
                && self.unit.talent_damage
            {
                0.1
            } else {
                0.0
            };
            final_atk = self.unit.atk * (1.0 + atkbuff + mainbuff + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            final_clone = self.unit.drone_atk * (1.0 + atkbuff + self.unit.buff_atk)
                + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            hitdmg_clone = ((final_clone - defense) as f64).max((final_clone * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if self.unit.talent_damage {
                dps += hitdmg_clone / (self.unit.drone_atk_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
            }
        }
        return dps;
    }
}

impl std::ops::Deref for Phantom {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Phantom {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
