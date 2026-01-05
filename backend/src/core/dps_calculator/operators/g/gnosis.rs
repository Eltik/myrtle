//! DPS calculations for Gnosis
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Gnosis operator implementation
pub struct Gnosis {
    pub unit: OperatorUnit,
}

impl Gnosis {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Gnosis operator
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
    /// coldfragile = 0.5 * (max(self.talent1_params) - 1) if self.elite > 0 else 0
    /// frozenfragile = 2 * coldfragile
    /// coldfragile = max(coldfragile, self.buff_fragile)
    /// frozenfragile = max(frozenfragile, self.buff_fragile)
    /// frozenres = np.fmax(0, res - 15)
    /// atkbuff = 0.05 * self.module_lvl if self.module == 2 and self.module_lvl > 1 else 0
    /// extra_sp = 0.25 if self.module == 2 and self.skill == 1 and self.module_dmg else 0
    /// ####the actual skills
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0]
    /// sp_cost = self.skill_cost/(1+ self.sp_boost + extra_sp) + 1.2 #sp lockout
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)*(1+coldfragile)/(1+self.buff_fragile)
    /// skilldmg1 = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * skill_scale * 0.05)*(1+coldfragile)/(1+self.buff_fragile)
    /// skilldmg2 = np.fmax(final_atk * skill_scale * (1-frozenres/100), final_atk * skill_scale * 0.05)*(1+frozenfragile)/(1+self.buff_fragile)
    /// skilldmg = skilldmg1 + skilldmg2
    /// if self.skill == 0: skilldmg = hitdmg
    /// atkcycle = self.atk_interval/((self.attack_speed)/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = skilldmg
    /// if atks_per_skillactivation > 1:
    /// avghit = (skilldmg + int(atks_per_skillactivation) * hitdmg) / (int(atks_per_skillactivation)+1)
    /// dps = avghit/self.atk_interval*(self.attack_speed)/100
    ///
    /// if self.skill == 3:
    /// aspd = self.skill_params[1]
    /// final_atk = self.atk * (1 + self.buff_atk + atkbuff) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * (1-res/100), final_atk * 0.05)*(1+coldfragile)/(1+self.buff_fragile)
    /// if self.skill_dmg: hitdmg = np.fmax(final_atk * (1-frozenres/100), final_atk * 0.05)*(1+frozenfragile)/(1+self.buff_fragile)
    /// dps = hitdmg/(self.atk_interval/((self.attack_speed + aspd)/100)) * min(2, self.targets)
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

        let mut final_atk: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut aspd: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut skilldmg: f64 = 0.0;
        let mut dps: f64 = 0.0;
        let mut avghit: f64 = 0.0;

        let mut coldfragile = if ((self.unit.elite as f64) as f64) > 0.0 {
            0.5 * (self
                .unit
                .talent1_parameters
                .iter()
                .cloned()
                .fold(f64::NEG_INFINITY, f64::max)
                - 1.0)
        } else {
            0.0
        };
        let mut frozenfragile = 2.0 * coldfragile;
        coldfragile = ((coldfragile) as f64).max((self.unit.buff_fragile) as f64);
        frozenfragile = ((frozenfragile) as f64).max((self.unit.buff_fragile) as f64);
        let mut frozenres = ((0) as f64).max((res - 15.0) as f64);
        atkbuff = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.module_level as f64) as f64) > 1.0
        {
            0.05 * ((self.unit.module_level as f64) as f64)
        } else {
            0.0
        };
        let mut extra_sp = if ((self.unit.module_index as f64) as f64) == 2.0
            && ((self.unit.skill_index as f64) as f64) == 1.0
            && self.unit.module_damage
        {
            0.25
        } else {
            0.0
        };
        // ###the actual skills
        if (self.unit.skill_index as f64) < 2.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64)
                / (1.0 + (self.unit.sp_boost as f64) + extra_sp)
                + 1.2;
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * (1.0 + coldfragile)
                / (1.0 + self.unit.buff_fragile);
            let mut skilldmg1 = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * (1.0 + coldfragile)
                / (1.0 + self.unit.buff_fragile);
            let mut skilldmg2 = ((final_atk * skill_scale * (1.0 - frozenres / 100.0)) as f64)
                .max((final_atk * skill_scale * 0.05) as f64)
                * (1.0 + frozenfragile)
                / (1.0 + self.unit.buff_fragile);
            skilldmg = skilldmg1 + skilldmg2;
            if (self.unit.skill_index as f64) == 0.0 {
                skilldmg = hitdmg;
            }
            let mut atkcycle =
                (self.unit.attack_interval as f64) / ((self.unit.attack_speed) / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = skilldmg;
            if atks_per_skillactivation > 1.0 {
                avghit = (skilldmg + ((atks_per_skillactivation) as f64).trunc() * hitdmg)
                    / (((atks_per_skillactivation) as f64).trunc() + 1.0);
            }
            dps = avghit / (self.unit.attack_interval as f64) * (self.unit.attack_speed) / 100.0;
        }
        if (self.unit.skill_index as f64) == 3.0 {
            aspd = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            final_atk =
                self.unit.atk * (1.0 + self.unit.buff_atk + atkbuff) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * (1.0 - res / 100.0)) as f64).max((final_atk * 0.05) as f64)
                * (1.0 + coldfragile)
                / (1.0 + self.unit.buff_fragile);
            if self.unit.skill_damage {
                hitdmg = ((final_atk * (1.0 - frozenres / 100.0)) as f64)
                    .max((final_atk * 0.05) as f64)
                    * (1.0 + frozenfragile)
                    / (1.0 + self.unit.buff_fragile);
            }
            dps = hitdmg
                / ((self.unit.attack_interval as f64) / ((self.unit.attack_speed + aspd) / 100.0))
                * ((2) as f64).min((self.unit.targets as f64) as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Gnosis {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Gnosis {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Gnosis {
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
