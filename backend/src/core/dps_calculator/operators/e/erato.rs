//! DPS calculations for Erato
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Erato operator implementation
pub struct Erato {
    pub unit: OperatorUnit,
}

impl Erato {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Erato operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            2, // default_skill_index
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
    /// atk_scale = 1.15 if self.module == 1 and self.module_dmg else 1
    /// newdef = defense * (1 - self.talent1_params[0]) if self.talent_dmg or self.skill == 1 else defense
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// hitdmg_base = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// if self.talent_dmg: hitdmg_base = hitdmg
    /// skilldmg = np.fmax(final_atk * skill_scale * atk_scale - newdef, final_atk * skill_scale * atk_scale * 0.05)
    /// if self.skill == 0: skilldmg = hitdmg
    /// sp_cost = self.skill_cost/(1+self.sp_boost) + 1.2 #sp lockout
    /// atkcycle = self.atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = int(sp_cost / atkcycle)
    /// hits_on_sleep = min(int(5 / atkcycle), atks_per_skillactivation)
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + hits_on_sleep * hitdmg + (atks_per_skillactivation-hits_on_sleep) * hitdmg_base) / (atks_per_skillactivation +1)
    /// dps = avghit/self.atk_interval*self.attack_speed/100
    ///
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// aspd = self.skill_params[1]
    /// final_atk = self.atk * (1 + atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// dps = hitdmg / self.atk_interval * (self.attack_speed + aspd) / 100
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

        let mut atkbuff: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;

        atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 && self.unit.module_damage {
            1.15
        } else {
            1.0
        };
        let mut newdef =
            if self.unit.talent_damage || ((self.unit.skill_index as f64) as f64) == 1.0 {
                defense * (1.0 - self.unit.talent1_parameters.first().copied().unwrap_or(0.0))
            } else {
                defense
            };
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            let mut hitdmg_base = ((final_atk * atk_scale - defense) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            if self.unit.talent_damage {
                hitdmg_base = hitdmg;
            }
            skilldmg = ((final_atk * skill_scale * atk_scale - newdef) as f64)
                .max((final_atk * skill_scale * atk_scale * 0.05) as f64);
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmg;
            }
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            let mut atkcycle =
                (self.unit.attack_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = ((sp_cost / atkcycle) as f64).trunc();
            let mut hits_on_sleep =
                ((((5.0 / atkcycle) as f64).trunc()) as f64).min((atks_per_skillactivation) as f64);
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg
                    + hits_on_sleep * hitdmg
                    + (atks_per_skillactivation - hits_on_sleep) * hitdmg_base)
                    / (atks_per_skillactivation + 1.0);
            }
            dps = avghit / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
        }
        if (self.unit.skill_index as f64) == 2.0 {
            atkbuff = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        return dps;
    }
}

impl std::ops::Deref for Erato {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Erato {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
