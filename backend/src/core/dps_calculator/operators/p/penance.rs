//! DPS calculations for Penance
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Penance operator implementation
pub struct Penance {
    pub unit: OperatorUnit,
}

impl Penance {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Penance operator
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
    /// atkbuff = 0.08 if self.module == 2 and self.module_dmg else 0
    ///
    /// if self.skill < 2:
    /// atk_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost
    /// final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk  * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale *(1-res/100), final_atk * atk_scale * 0.05)
    /// if self.skill == 0: skilldmg = hitdmg
    /// sp_cost = sp_cost + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/((self.attack_speed)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + atks_per_skillactivation * hitdmg) / atks_per_skillactivation
    /// dps = avghit / self.atk_interval * self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// atk_scale = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmgarts = np.fmax(final_atk *atk_scale *(1-res/100), final_atk * atk_scale * 0.05)
    /// dps = hitdmgarts * self.targets
    /// if self.skill == 3:
    /// atk_interval = 2.5
    /// atkbuff += self.skill_params[2]
    /// final_atk = self.atk * (1 + atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk - defense, final_atk * 0.05)
    /// dps = hitdmg/(atk_interval/(self.attack_speed/100))
    ///
    /// if self.hits > 0:
    /// arts_scale = self.talent2_params[0]
    /// artsdmg = np.fmax(final_atk * arts_scale * (1-res/100), final_atk * arts_scale * 0.05)
    /// dps += artsdmg * self.hits
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
        let defense = enemy.defense;
        let res = enemy.res;

        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0 && self.unit.module_damage {
            0.08
        } else {
            0.0
        };
        if (self.unit.skill_index as f64) < 2.0 {
            atk_scale = self.unit.skill_parameters[0];
            sp_cost = (self.unit.skill_cost as f64);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            skilldmg = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmg;
            }
            sp_cost = sp_cost + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg + atks_per_skillactivation * hitdmg) / atks_per_skillactivation;
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atk_scale = self.unit.skill_parameters[1];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmgarts = ((final_atk * atk_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmgarts * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 3.0 {
            atk_interval = 2.5;
            atkbuff += self.unit.skill_parameters[2];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk - defense) as f64).max((final_atk * 0.05) as f64);
            dps = hitdmg / (atk_interval / (self.unit.attack_speed / 100.0));
        }
        if 1.0 /* self.hits - needs manual implementation */ > 0.0 {
            let mut arts_scale = self.unit.talent2_parameters[0];
            let mut artsdmg = ((final_atk * arts_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * arts_scale * 0.05) as f64);
            dps += artsdmg * 1.0 /* self.hits - needs manual implementation */;
        }
        return dps;
    }
}

impl std::ops::Deref for Penance {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Penance {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
