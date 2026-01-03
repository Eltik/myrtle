//! DPS calculations for Pinecone
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_unit::{EnemyStats, OperatorParams, OperatorUnit};
use super::super::super::operator_data::OperatorData;

/// Pinecone operator implementation
pub struct Pinecone {
    pub unit: OperatorUnit,
}

impl Pinecone {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1];

    /// Creates a new Pinecone operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            1, // default_skill_index
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
    /// atk_scale = 1
    /// if self.trait_dmg or self.skill == 2: atk_scale = 1.6 if self.module == 1 else 1.5
    /// 
    /// if self.skill < 2:
    /// skill_scale = self.skill_params[0] * self.skill
    /// defignore = self.skill_params[1] if self.skill == 1 else 0
    /// newdef = np.fmax(0, defense - defignore)
    /// sp_cost = self.skill_cost +1.2 #sp_lockout
    /// final_atk = self.atk * (1+ self.buff_atk) + self.buff_atk_flat
    /// if self.talent_dmg: sp_cost = sp_cost / (1+ self.talent1_params[0])
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// skilldmg = np.fmax(final_atk * atk_scale * skill_scale - newdef, final_atk * skill_scale * atk_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * self.targets + skilldmg / sp_cost * self.targets
    /// if self.skill == 2:
    /// atkbuff = self.skill_params[0]
    /// if self.skill_dmg: atkbuff += 0.6
    /// final_atk = self.atk * (1+ atkbuff + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - defense, final_atk * atk_scale * 0.05)
    /// dps = hitdmg/(self.atk_interval/(self.attack_speed/100)) * self.targets
    /// return dps
    #[allow(unused_variables, unused_mut, unused_assignments, unused_parens, clippy::excessive_precision, clippy::unnecessary_cast, clippy::collapsible_if, clippy::double_parens, clippy::if_same_then_else, clippy::nonminimal_bool, clippy::overly_complex_bool_expr, clippy::needless_return, clippy::collapsible_else_if, clippy::neg_multiply, clippy::assign_op_pattern, clippy::eq_op)]
    pub fn skill_dps(&self, enemy: &EnemyStats) -> f64 {
        let defense = enemy.defense;
        let res = enemy.res;

        let mut dps: f64 = 0.0;
        let mut skilldmg: f64 = 0.0;
        let mut skill_scale: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut final_atk: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut atkbuff: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_interval: f64 = 0.0;

        atk_scale = 1.0;
        if self.unit.trait_damage || (self.unit.skill_index as f64) == 2.0 { atk_scale = if ((self.unit.module_index as f64) as f64) == 1.0 { 1.6 } else { 1.5 }; }
        if (self.unit.skill_index as f64) < 2.0 {
        skill_scale = self.unit.skill_parameters[0] * (self.unit.skill_index as f64);
        let mut defignore = if ((self.unit.skill_index as f64) as f64) == 1.0 { self.unit.skill_parameters[1] } else { 0.0 };
        let mut newdef = ((0) as f64).max((defense - defignore) as f64);
        sp_cost = (self.unit.skill_cost as f64) +1.2;
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if self.unit.talent_damage { sp_cost = sp_cost / (1.0 + self.unit.talent1_parameters[0]); }
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        skilldmg = ((final_atk * atk_scale * skill_scale - newdef) as f64).max((final_atk * skill_scale * atk_scale * 0.05) as f64);
        dps = hitdmg/((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0)) * (self.unit.targets as f64) + skilldmg / sp_cost * (self.unit.targets as f64);
        }
        if (self.unit.skill_index as f64) == 2.0 {
        atkbuff = self.unit.skill_parameters[0];
        // UNTRANSLATED: if self.skill_dmg: atkbuff += 0.6
        final_atk = self.unit.atk * (1.0 + atkbuff + self.unit.buff_atk) + self.unit.buff_atk_flat;
        hitdmg = ((final_atk * atk_scale - defense) as f64).max((final_atk * atk_scale * 0.05) as f64);
        dps = hitdmg/((self.unit.attack_interval as f64)/(self.unit.attack_speed/ 100.0)) * (self.unit.targets as f64);
        }
        return dps;
    }
}

impl std::ops::Deref for Pinecone {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Pinecone {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}
