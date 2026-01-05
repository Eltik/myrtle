//! DPS calculations for Vigil
//!
//! Auto-generated from ArknightsDpsCompare damage_formulas.py

use super::super::super::operator_data::OperatorData;
use super::super::super::operator_unit::{DpsCalculator, EnemyStats, OperatorParams, OperatorUnit};

/// Vigil operator implementation
pub struct Vigil {
    pub unit: OperatorUnit,
}

impl Vigil {
    /// Available skills for this operator
    pub const AVAILABLE_SKILLS: &'static [i32] = &[1, 2, 3];

    /// Available modules for this operator
    pub const AVAILABLE_MODULES: &'static [i32] = &[1, 2];

    /// Creates a new Vigil operator
    pub fn new(operator_data: OperatorData, params: OperatorParams) -> Self {
        let unit = OperatorUnit::new(
            operator_data,
            params,
            3, // default_skill_index
            6, // default_potential
            2, // default_module_index
            Self::AVAILABLE_SKILLS.to_vec(),
        );

        Self { unit }
    }

    /// Calculates DPS against an enemy
    ///
    /// Original Python implementation:
    ///
    /// atk_scale = 1
    /// defignore = 0
    /// wolves = 0
    /// if self.talent_dmg:
    /// wolves = 3 if self.skill_dmg  else 1
    /// if self.trait_dmg:
    /// atk_scale = 1.65 if self.module == 2 else 1.5
    /// defignore = self.talent2_params[0] if self.elite == 2 else 0
    /// newdef = np.fmax(0, defense - defignore)
    /// wolfdef = np.fmax(0, defense - self.talent2_params[0]) if self.elite == 2 else defense
    /// ####the actual skills
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// final_wolf  = self.drone_atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// if self.skill < 2:
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// hitdmgwolf = np.fmax(final_wolf - wolfdef, final_wolf * 0.05)
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.talent_dmg: dps += hitdmgwolf/self.drone_atk_interval * self.attack_speed/100 * wolves
    ///
    /// if self.skill == 2:
    /// skill_scale = self.skill_params[1]
    /// sp_cost = self.skill_cost/(1 + self.sp_boost) + 1.2 #lockout
    ///
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// hitdmgwolf = np.fmax(final_wolf - wolfdef, final_wolf * 0.05)
    /// hitdmgwolfskill = np.fmax(final_wolf * skill_scale - wolfdef, final_wolf * skill_scale * 0.05)
    /// atkcycle = self.drone_atk_interval/(self.attack_speed/100)
    /// atks_per_skillactivation = sp_cost / atkcycle
    /// avghit = hitdmgwolfskill
    /// if atks_per_skillactivation > 1:
    /// avghit = (hitdmgwolfskill + (atks_per_skillactivation - 1) * hitdmgwolf) / atks_per_skillactivation
    /// dps = hitdmg/self.atk_interval * self.attack_speed/100
    /// if self.talent_dmg: dps += avghit/self.drone_atk_interval * self.attack_speed/100 * wolves
    ///
    /// if self.skill == 3:
    /// skill_scale = self.skill_params[0]
    /// final_atk = self.atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// final_wolf  = self.drone_atk * (1 + self.buff_atk) + self.buff_atk_flat
    /// hitdmg = np.fmax(final_atk * atk_scale - newdef, final_atk * atk_scale * 0.05)
    /// hitdmgwolf = np.fmax(final_wolf - wolfdef, final_wolf * 0.05)
    /// hitdmgarts = np.fmax(final_atk * skill_scale * (1-res/100), final_atk * 0.05)
    /// hitdps = 3 * hitdmg/self.atk_interval * self.attack_speed/100
    /// artdps = 0
    /// if self.talent_dmg:
    /// hitdps += wolves * hitdmgwolf/self.drone_atk_interval * self.attack_speed/100
    /// artdps = wolves * hitdmgarts/self.drone_atk_interval * self.attack_speed/100
    /// if self.trait_dmg:
    /// artdps += 3 * hitdmgarts/self.atk_interval * self.attack_speed/100
    /// dps = hitdps + artdps
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
        let mut skill_scale: f64 = 0.0;
        let mut hitdmgarts: f64 = 0.0;
        let mut atk_interval: f64 = self.unit.attack_interval as f64;
        let mut dps: f64 = 0.0;
        let mut sp_cost: f64 = 0.0;
        let mut avghit: f64 = 0.0;
        let mut hitdmg: f64 = 0.0;
        let mut atk_scale: f64 = 0.0;
        let mut defignore: f64 = 0.0;
        let mut hitdmgwolf: f64 = 0.0;
        let mut artdps: f64 = 0.0;

        atk_scale = 1.0;
        defignore = 0.0;
        let mut wolves = 0.0;
        if self.unit.talent_damage {
            wolves = if self.unit.skill_damage { 3.0 } else { 1.0 };
            if self.unit.trait_damage {
                atk_scale = if ((self.unit.module_index as f64) as f64) == 2.0 {
                    1.65
                } else {
                    1.5
                };
                defignore = if ((self.unit.elite as f64) as f64) == 2.0 {
                    self.unit.talent2_parameters.first().copied().unwrap_or(0.0)
                } else {
                    0.0
                };
            }
        }
        let mut newdef = ((0) as f64).max((defense - defignore) as f64);
        let mut wolfdef = if ((self.unit.elite as f64) as f64) == 2.0 {
            ((0) as f64)
                .max((defense - self.unit.talent2_parameters.first().copied().unwrap_or(0.0)) as f64)
        } else {
            defense
        };
        // ###the actual skills
        final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        let mut final_wolf =
            self.unit.drone_atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
        if (self.unit.skill_index as f64) < 2.0 {
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmgwolf = ((final_wolf - wolfdef) as f64).max((final_wolf * 0.05) as f64);
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if self.unit.talent_damage {
                dps += hitdmgwolf / (self.unit.drone_atk_interval as f64) * self.unit.attack_speed
                    / 100.0
                    * wolves;
            }
        }
        if (self.unit.skill_index as f64) == 2.0 {
            skill_scale = self.unit.skill_parameters.get(1).copied().unwrap_or(0.0);
            sp_cost = (self.unit.skill_cost as f64) / (1.0 + (self.unit.sp_boost as f64)) + 1.2;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmgwolf = ((final_wolf - wolfdef) as f64).max((final_wolf * 0.05) as f64);
            let mut hitdmgwolfskill = ((final_wolf * skill_scale - wolfdef) as f64)
                .max((final_wolf * skill_scale * 0.05) as f64);
            let mut atkcycle =
                (self.unit.drone_atk_interval as f64) / (self.unit.attack_speed / 100.0);
            let mut atks_per_skillactivation = sp_cost / atkcycle;
            avghit = hitdmgwolfskill;
            if atks_per_skillactivation > 1.0 {
                avghit = (hitdmgwolfskill + (atks_per_skillactivation - 1.0) * hitdmgwolf)
                    / atks_per_skillactivation;
            }
            dps = hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            if self.unit.talent_damage {
                dps += avghit / (self.unit.drone_atk_interval as f64) * self.unit.attack_speed
                    / 100.0
                    * wolves;
            }
        }
        if (self.unit.skill_index as f64) == 3.0 {
            skill_scale = self.unit.skill_parameters.first().copied().unwrap_or(0.0);
            final_atk = self.unit.atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            final_wolf = self.unit.drone_atk * (1.0 + self.unit.buff_atk) + self.unit.buff_atk_flat;
            hitdmg = ((final_atk * atk_scale - newdef) as f64)
                .max((final_atk * atk_scale * 0.05) as f64);
            hitdmgwolf = ((final_wolf - wolfdef) as f64).max((final_wolf * 0.05) as f64);
            hitdmgarts = ((final_atk * skill_scale * (1.0 - res / 100.0)) as f64)
                .max((final_atk * 0.05) as f64);
            let mut hitdps =
                3.0 * hitdmg / (self.unit.attack_interval as f64) * self.unit.attack_speed / 100.0;
            artdps = 0.0;
            if self.unit.talent_damage {
                hitdps += wolves * hitdmgwolf / (self.unit.drone_atk_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
                artdps = wolves * hitdmgarts / (self.unit.drone_atk_interval as f64)
                    * self.unit.attack_speed
                    / 100.0;
                if self.unit.trait_damage {
                    artdps += 3.0 * hitdmgarts / (self.unit.attack_interval as f64)
                        * self.unit.attack_speed
                        / 100.0;
                }
            }
            dps = hitdps + artdps;
        }
        return dps;
    }
}

impl std::ops::Deref for Vigil {
    type Target = OperatorUnit;

    fn deref(&self) -> &Self::Target {
        &self.unit
    }
}

impl std::ops::DerefMut for Vigil {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.unit
    }
}

impl DpsCalculator for Vigil {
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
