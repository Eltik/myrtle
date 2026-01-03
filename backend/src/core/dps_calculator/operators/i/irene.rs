//! DPS calculations for Irene
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};

/// Irene operator implementation
pub struct Irene {
    pub unit: OperatorUnit,
}

impl Irene {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[2, 1];

    /// Creates a new Irene operator
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
    /// aspd = self.talent2_params[0]
    /// atkbuff = self.talent2_params[1] if self.module == 2 and self.module_lvl > 1 else 0
    /// if self.talent2_dmg:
    /// atkbuff *= 2
    /// aspd *= 2
    /// skill_dmg = 1.1 if self.module == 1 else 1
    /// newdef1 = defense if self.module != 2 else np.fmax(0, defense -70)
    /// defshred = 0
    /// if self.elite > 0:
    /// defshred = self.talent1_params[0]
    /// newdef2 = newdef1 * (1-defshred)
    ///
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1+atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// hitdmg1 = np.fmax(final_atk - newdef1, final_atk * 0.05)
    /// hitdmg2 = np.fmax(final_atk - newdef2, final_atk * 0.05)
    /// skill_dmg = np.fmax(final_atk * skill_scale - newdef2, final_atk * skill_scale * 0.05) * skill_dmg
    /// if self.skill == 0: skill_dmg = (hitdmg1+hitdmg2)/2
    /// sp_cost = self.skill_cost
    /// avgdmg = ((hitdmg1+hitdmg2) * sp_cost + 2 * skill_dmg)/(sp_cost + 1)
    /// dps = avgdmg / self.atk_interval * (self.attack_speed+aspd)/100
    ///
    /// if self.skill == 3:
    /// skill_scale1 = self.skill_params[0]
    /// hits = self.skill_params[3]
    /// skill_scale = self.skill_params[2]
    /// final_atk = self.atk * (1+atkbuff+ self.buff_atk) + self.buff_atk_flat
    /// initialhit1 = np.fmax(final_atk * skill_scale1 - newdef1, final_atk *skill_scale1 * 0.05)*skill_dmg
    /// initialhit2 = np.fmax(final_atk * skill_scale1 - newdef2, final_atk * skill_scale1 * 0.05)*skill_dmg
    /// hitdmg1 = np.fmax(final_atk * skill_scale - newdef1, final_atk *skill_scale * 0.05)*skill_dmg
    /// hitdmg2 = np.fmax(final_atk * skill_scale - newdef2, final_atk *skill_scale * 0.05)*skill_dmg
    /// dps = 0.5*initialhit1 + 0.5* initialhit2
    /// levduration = self.skill_params[1]
    /// if not self.talent_dmg: return (dps + hits * (0.5*hitdmg1+0.5*hitdmg2))
    /// else:
    /// if not self.skill_dmg:
    /// levduration = levduration /2
    /// flyinghits = min(hits, int(levduration / 0.3))
    /// dps += flyinghits * hitdmg2 + (hits-flyinghits) * (0.5*hitdmg1+0.5*hitdmg2)
    /// dps *= self.targets
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

        let mut levduration: f64 = 0.0;
        let mut flyinghits: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut hitdmg2: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut avgdmg: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg1: f64 = 0.0;
        let mut defshred: f64 = 0.0;

        aspd = self.unit.talent2_parameters[0];
        atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            self.unit.talent2_parameters[1]
        } else {
            0.0
        };
        if self.unit.talent2_damage {
            atkbuff *= 2.0;
            aspd *= 2.0;
        }
        let mut skill_dmg = if ((self.unit.module_index as f64) as f64) == 1.0 {
            1.1
        } else {
            1.0
        };
        let mut newdef1 = if ((self.unit.module_index as f64) as f64) != 2.0 {
            defense
        } else {
            ((0) as f64).max((defense - 70.0) as f64)
        };
        defshred = 0.0;
        if (self.unit.elite as f64) > 0.0 {
            defshred = self.unit.talent1_parameters[0];
        }
        let mut newdef2 = newdef1 * (1.0 - defshred);
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters[0];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg1 = ((final_atk - newdef1) as f64).max((final_atk * 0.05) as f64);
            hitdmg2 = ((final_atk - newdef2) as f64).max((final_atk * 0.05) as f64);
            skill_dmg = ((final_atk * skill_scale - newdef2) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * skill_dmg;
            if (self.unit.skill_index as f64) == 0.0 {
                skill_dmg = (hitdmg1 + hitdmg2) / 2.0;
            }
            sp_cost = (self.unit.skill_cost as f64);
            avgdmg = ((hitdmg1 + hitdmg2) * sp_cost + 2.0 * skill_dmg) / (sp_cost + 1.0);
            dps = avgdmg / (self.unit.attack_interval as f64) * (self.unit.attack_speed + aspd)
                / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            let mut skill_scale1 = self.unit.skill_parameters[0];
            let mut hits = self.unit.skill_parameters[3];
            skill_scale = self.unit.skill_parameters[2];
            final_atk =
                self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
            let mut initialhit1 = ((final_atk * skill_scale1 - newdef1) as f64)
                .max((final_atk * skill_scale1 * 0.05) as f64)
                * skill_dmg;
            let mut initialhit2 = ((final_atk * skill_scale1 - newdef2) as f64)
                .max((final_atk * skill_scale1 * 0.05) as f64)
                * skill_dmg;
            hitdmg1 = ((final_atk * skill_scale - newdef1) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * skill_dmg;
            hitdmg2 = ((final_atk * skill_scale - newdef2) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * skill_dmg;
            dps = 0.5 * initialhit1 + 0.5 * initialhit2;
            levduration = self.unit.skill_parameters[1];
            // UNTRANSLATED: if not self.talent_dmg: return (dps + hits * (0.5*hitdmg1+0.5*hitdmg2))
            // UNTRANSLATED ELSE (no matching if): else:
            if !self.unit.skill_damage {
                levduration = levduration / 2.0;
            }
            flyinghits = ((hits) as f64).min((((levduration / 0.3) as f64).trunc()) as f64);
            dps += flyinghits * hitdmg2 + (hits - flyinghits) * (0.5 * hitdmg1 + 0.5 * hitdmg2);
            dps *= (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Irene {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Irene {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
