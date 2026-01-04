//! DPS calculations for YutenjiNyamu
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// YutenjiNyamu operator implementation
pub struct YutenjiNyamu {
    pub unit: OperatorUnit,
}

impl YutenjiNyamu {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new YutenjiNyamu operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
            6, // default_potential
            1, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    /// atkbuff = self.talent2_params[2] if self.elite > 0 else 0
    /// hits = 3 if self.skill == 1 else 8
    /// if self.skill == 0: hits = 1
    /// prob = self.talent1_params[0]
    /// duration = self.talent1_params[1]
    /// fragile = self.talent1_params[2]
    /// counting_hits = hits * int(duration/self.atk_interval) + max(1,hits/2) #only approximation, the later hits in the chain have a higher fragile chance
    /// fragile_chance = 1 - (1-prob)**counting_hits
    /// fragile = fragile * fragile_chance + (1-fragile_chance)
    /// fragile = max(fragile, 1+self.buff_fragile)
    ///
    /// if self.skill == 0:
    /// skill_scale = self.skill_params[0] if self.skill == 1 else 1
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// splashhitdmg = np.fmax(0.5 * final_atk * atk_scale - defense, 0.5 * final_atk * atk_scale * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.targets > 1:
    /// dps += splashhitdmg/self.atk_interval * self.attack_speed/100 * (self.targets - 1)
    /// else:
    /// big_scale = self.skill_params[0]
    /// small_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// bighitdmg = np.fmax(final_atk * atk_scale * big_scale - defense, final_atk * atk_scale * big_scale * 0.05) * int(hits/3)
    /// bigsplashhitdmg = np.fmax(0.5 * final_atk * atk_scale * big_scale - defense, 0.5 * final_atk * atk_scale * big_scale * 0.05) * int(hits/3)
    /// smallhitdmg = np.fmax(final_atk * atk_scale * small_scale - defense, final_atk * atk_scale * small_scale * 0.05) * (hits - int(hits/3))
    /// smallsplashhitdmg = np.fmax(0.5 * final_atk * atk_scale * small_scale - defense, 0.5 * final_atk * atk_scale * small_scale * 0.05) * (hits - int(hits/3))
    /// dps = (bighitdmg+smallhitdmg)/self.atk_interval * self.attack_speed/100
    /// if self.targets > 1:
    /// dps += (bigsplashhitdmg+smallsplashhitdmg)/self.atk_interval * self.attack_speed/100 * (self.targets - 1)
    /// return dps * fragile/(1+self.buff_fragile)
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

        let mut skill_scale: f64 = 0.0;
        let mut small_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut smallsplashhitdmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut smallhitdmg: f64 = 0.0;
        let mut bighitdmg: f64 = 0.0;
        let mut bigsplashhitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut big_scale: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        atkbuff = if ((self.unit.elite as f64) as f64) > 0.0 {
            self.unit.talent2_parameters.get(2).copied().unwrap_or(0.0)
        } else {
            0.0
        };
        let mut hits = if ((self.unit.skill_index as f64) as f64) == 1.0 {
            3.0
        } else {
            8.0
        };
        if (self.unit.skill_index as f64) == 0.0 {
            hits = 1.0;
        }
        let mut prob = self.unit.talent1_parameters.first().copied().unwrap_or(0.0);
        let mut duration = self.unit.talent1_parameters.get(1).copied().unwrap_or(0.0);
        let mut fragile = self.unit.talent1_parameters.get(2).copied().unwrap_or(0.0);
        let mut counting_hits = hits
            * ((duration / (self.unit.attack_interval as f64)) as f64).trunc()
            + ((1) as f64).max((hits / 2.0) as f64);
        let mut fragile_chance = 1.0 - ((1.0 - prob) as f64).powf(counting_hits as f64);
        fragile = fragile * fragile_chance + (1.0 - fragile_chance);
        fragile = ((fragile) as f64).max((1.0 + self.unit.buff_fragile) as f64);
        if (self.unit.skill_index as f64) == 0.0 {
            skill_scale = if ((self.unit.skill_index as f64) as f64) == 1.0 {
                self.unit.skill_parameters.first().copied().unwrap_or(0.0)
            } else {
                1.0
            };
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut splashhitdmg = ((0.5 * final_atk * atk_scale - defense) as f64)
                .max((0.5 * final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps += splashhitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed
                    / 100.0
                    * ((self.unit.targets as f64) - 1.0);
            }
        } else {
            big_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            small_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            bighitdmg = ((final_atk * atk_scale * big_scale - defense) as f64)
                .max((final_atk * atk_scale * big_scale * 0.05) as f64)
                * ((hits / 3.0) as f64).trunc();
            bigsplashhitdmg = ((0.5 * final_atk * atk_scale * big_scale - defense) as f64)
                .max((0.5 * final_atk * atk_scale * big_scale * 0.05) as f64)
                * ((hits / 3.0) as f64).trunc();
            smallhitdmg = ((final_atk * atk_scale * small_scale - defense) as f64)
                .max((final_atk * atk_scale * small_scale * 0.05) as f64)
                * (hits - ((hits / 3.0) as f64).trunc());
            smallsplashhitdmg = ((0.5 * final_atk * atk_scale * small_scale - defense) as f64)
                .max((0.5 * final_atk * atk_scale * small_scale * 0.05) as f64)
                * (hits - ((hits / 3.0) as f64).trunc());
            dps = (bighitdmg + smallhitdmg) / (self.unit.attack_interval as f64)
                * self.unit.attack_speed
                / 100.0;
            if (self.unit.targets as f64) > 1.0 {
                dps += (bigsplashhitdmg + smallsplashhitdmg) / (self.unit.attack_interval as f64)
                    * self.unit.attack_speed
                    / 100.0
                    * ((self.unit.targets as f64) - 1.0);
            }
        }
        return dps * fragile / (1.0 + self.unit.buff_fragile);
    }
}

impl std::ops::Deref for YutenjiNyamu {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for YutenjiNyamu {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
