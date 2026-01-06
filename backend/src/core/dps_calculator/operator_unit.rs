use crate::core::local::types::{enemy::Enemy, operator::OperatorModule};

use super::operator_data::OperatorData;

const MAX_LEVELS: [[i32; 6]; 3] = [
    [30, 30, 40, 45, 50, 50],
    [0, 0, 55, 60, 70, 80],
    [0, 0, 0, 70, 80, 90],
];

const MAX_PROMOTIONS: [i32; 6] = [0, 0, 1, 2, 2, 2];
const MAX_SKILL_LEVELS: [i32; 3] = [4, 7, 10];

pub struct OperatorUnit {
    pub data: OperatorData,
    pub params: OperatorParams,

    pub default_skill_index: i32,
    pub default_potential: i32,
    pub default_module_index: i32,

    pub available_skills: Vec<i32>,

    pub targets: i32,

    pub rarity: i32,

    pub atk: f64,
    pub attack_interval: f32,
    pub attack_speed: f64,

    pub elite: i32,
    pub level: i32,
    pub potential: i32,
    pub skill_level: i32,
    pub trust: i32,

    pub operator_module: Option<OperatorModule>,
    pub operator_module_level: i32,
    pub module_index: i32,
    pub module_level: i32,

    pub skill_index: i32,
    pub skill_parameters: Vec<f64>,
    pub skill_duration: f64,
    pub skill_cost: i32,

    pub sp_boost: f32,

    pub is_physical: bool,
    pub is_ranged: bool,

    pub talent1_parameters: Vec<f64>,
    pub talent2_parameters: Vec<f64>,

    pub drone_atk: f64,
    pub drone_atk_interval: f32,

    pub trait_damage: bool,
    pub trait_damage_name: Option<String>,
    pub trait_damage_names: Vec<String>,

    pub talent_damage: bool,
    pub talent_damage_name: Option<String>,
    pub talent_damage_names: Vec<String>,

    pub talent2_damage: bool,
    pub talent2_damage_name: Option<String>,
    pub talent2_damage_names: Vec<String>,

    pub skill_damage: bool,
    pub skill_damage_name: Option<String>,
    pub skill_damage_names: Vec<String>,

    pub module_damage: bool,
    pub module_damage_name: Option<String>,
    pub module_damage_names: Vec<String>,

    pub buff_name: String,
    pub buff_atk: f64,
    pub buff_atk_flat: f64,
    pub buff_fragile: f64,

    // Operator-specific fields for advanced calculations
    pub shreds: Vec<f64>, // Defense shred values [def_shred_mult, def_shred_flat, res_shred_mult, res_shred_flat]

    /// Ammo count for operators like ExecutorAlter (set in __init__)
    pub ammo: f64,
}

impl OperatorUnit {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        operator_data: OperatorData,
        params: OperatorParams,
        default_skill_index: i32,
        default_potential: i32,
        default_module_index: i32,
        available_skills: Vec<i32>,
    ) -> Self {
        let rarity = operator_data.rarity;

        // Apply defaults to params
        let all_cond = params.all_cond.unwrap_or(true);
        let trust = params.trust.unwrap_or(100);
        let targets = params.targets.unwrap_or(1).max(1);
        let sp_boost = params.sp_boost.unwrap_or(0.0);

        // Get conditionals with defaults
        let conditionals = params.conditionals.as_ref();
        let trait_damage = if !all_cond {
            false
        } else {
            conditionals.and_then(|c| c.trait_damage).unwrap_or(true)
        };
        let talent_damage = if !all_cond {
            false
        } else {
            conditionals.and_then(|c| c.talent_damage).unwrap_or(true)
        };
        let talent2_damage = if !all_cond {
            false
        } else {
            conditionals.and_then(|c| c.talent2_damage).unwrap_or(true)
        };
        let skill_damage = if !all_cond {
            false
        } else {
            conditionals.and_then(|c| c.skill_damage).unwrap_or(true)
        };
        let module_damage = if !all_cond {
            false
        } else {
            conditionals.and_then(|c| c.module_damage).unwrap_or(true)
        };

        // Set attack interval
        let attack_interval = operator_data.atk_interval;

        // Set is_ranged and is_physical
        let is_ranged = operator_data.is_ranged;
        let is_physical = operator_data.is_physical;

        // Calculate elite level
        let promotion = params.promotion.unwrap_or(-1);
        let mut elite = if promotion < 0 { 2 } else { promotion };
        elite = elite.max(0).min(MAX_PROMOTIONS[(rarity - 1) as usize]);

        // Calculate level
        let param_level = params.level.unwrap_or(-1);
        let max_level_for_elite = MAX_LEVELS[elite as usize][(rarity - 1) as usize];
        let level = if param_level > 0 && param_level < max_level_for_elite {
            param_level
        } else {
            max_level_for_elite
        };

        // Calculate potential
        let param_potential = params.potential.unwrap_or(-1);
        let potential = if (1..=6).contains(&param_potential) {
            param_potential
        } else if (1..=6).contains(&default_potential) {
            default_potential
        } else {
            1
        };

        // Calculate skill index
        // Note: skill_index is 1-indexed (1=S1, 2=S2, 3=S3) to match Python semantics
        // Python uses skill=0 for "no skill" (basic attack), skill=1,2,3 for S1,S2,S3
        let mut skill_index = 0;
        if rarity > 2 {
            let param_skill_index = params.skill_index.unwrap_or(0);
            let skills_len = operator_data.data.skills.len() as i32;

            skill_index = param_skill_index;

            // Validate skill_index is within range (1-indexed, so skills_len >= skill_index)
            if !(skills_len >= skill_index || skill_index == 0) {
                skill_index = if skills_len >= default_skill_index {
                    default_skill_index
                } else {
                    skills_len // Last skill in 1-indexed
                };
            }

            // Amiya is a special child
            let is_amiya = operator_data
                .data
                .id
                .as_deref()
                .map(|id| id == "char_002_amiya")
                .unwrap_or(false);

            // For E1 or < 6-star with S3 selected, limit to S2 max (1-indexed: 3 -> 2)
            if !is_amiya && (elite == 1 || (rarity < 6 && skill_index == 3)) {
                skill_index = if skills_len >= 2 {
                    2 // S2 in 1-indexed
                } else if skills_len >= 1 {
                    1 // S1 in 1-indexed
                } else {
                    0
                };
            }

            // For E0, only S1 available (1-indexed: 1)
            if elite == 0 && skill_index >= 1 {
                skill_index = if skills_len >= 1 { 1 } else { 0 };
            }
        }

        // Calculate skill level
        let mastery_level = params.mastery_level.unwrap_or(-1);
        let skill_level =
            if mastery_level > 0 && mastery_level + 6 < MAX_SKILL_LEVELS[elite as usize] {
                match mastery_level {
                    3 => 9,
                    2 => 8,
                    1 => 7,
                    _ => 9,
                }
            } else {
                MAX_SKILL_LEVELS[elite as usize]
            };

        // Calculate trust (already got default above, but need to clamp)
        let trust = if (0..100).contains(&trust) {
            trust
        } else {
            100
        };

        // Calculate module
        // Note: module_index is 1-indexed (1=X, 2=Y, 3=Δ) like Python
        // For array access, we use (module_index - 1)
        let mut operator_module: Option<OperatorModule> = None;
        let mut operator_module_level = 0;
        let mut module_index: i32 = 0;
        let mut module_level: i32 = 0;

        let max_level_e2 = MAX_LEVELS[2][(rarity - 1) as usize];
        if elite == 2 && level >= max_level_e2 - 30 {
            let available_modules = &operator_data.available_modules;

            // Get default module by order number (char_equip_order)
            if default_module_index >= 1 {
                let found_default = available_modules
                    .iter()
                    .enumerate()
                    .find(|(_, m)| m.module.char_equip_order == default_module_index);
                if let Some((idx, _)) = found_default {
                    operator_module = Some(available_modules[idx].clone());
                    module_index = default_module_index;
                }
            }

            if !available_modules.is_empty() {
                let param_module_index = params.module_index.unwrap_or(0);

                if param_module_index == 0 {
                    operator_module = None;
                    module_index = 0;
                } else if param_module_index >= 1 {
                    // Python uses module_index as 1-indexed selection of available modules
                    // module_index=1 means X module (char_equip_order=1 if exists, else first with order>0)
                    // module_index=2 means Y module (char_equip_order=2 if exists, else second with order>0)
                    // First try to find by exact char_equip_order match
                    let found_module = available_modules
                        .iter()
                        .find(|m| m.module.char_equip_order == param_module_index);

                    // If not found by exact match, try to find nth module with order > 0
                    let found_module = found_module.or_else(|| {
                        let mut real_modules: Vec<_> = available_modules
                            .iter()
                            .filter(|m| m.module.char_equip_order > 0)
                            .collect();
                        // Sort by char_equip_order for consistent ordering
                        real_modules.sort_by_key(|m| m.module.char_equip_order);
                        let array_idx = (param_module_index - 1) as usize;
                        real_modules.get(array_idx).copied()
                    });

                    if let Some(module) = found_module {
                        operator_module = Some(module.clone());
                        module_index = param_module_index;

                        let param_module_level = params.module_level.unwrap_or(-1);
                        operator_module_level = if param_module_level <= 3 && param_module_level > 1
                        {
                            param_module_level
                        } else {
                            3
                        };

                        if trust < 50 {
                            operator_module_level = 1;
                        }
                        if trust < 100 {
                            operator_module_level = operator_module_level.min(2);
                        }

                        module_level = operator_module_level;
                    }
                }
            }
        }

        // Set default attack speed
        let mut attack_speed: f64 = 100.0;

        // Calculate ATK based on elite level
        let max_level_for_elite_f64 = MAX_LEVELS[elite as usize][(rarity - 1) as usize] as f64;
        let mut atk = match elite {
            0 => {
                let min = operator_data.atk.e0.min as f64;
                let max = operator_data.atk.e0.max as f64;
                min + ((max - min) * (level as f64 - 1.0)) / (max_level_for_elite_f64 - 1.0)
            }
            1 => {
                let min = operator_data.atk.e1.min as f64;
                let max = operator_data.atk.e1.max as f64;
                min + ((max - min) * (level as f64 - 1.0)) / (max_level_for_elite_f64 - 1.0)
            }
            2 => {
                let min = operator_data.atk.e2.min as f64;
                let max = operator_data.atk.e2.max as f64;
                min + ((max - min) * (level as f64 - 1.0)) / (max_level_for_elite_f64 - 1.0)
            }
            _ => 0.0,
        };

        // Apply potential ATK bonus
        if potential >= operator_data.atk_potential.required_potential {
            atk += operator_data.atk_potential.value;
        }

        // Apply trust ATK bonus
        atk += (operator_data.atk_trust * trust as f64) / 100.0;

        // Apply potential ASPD bonus
        if potential >= operator_data.aspd_potential.required_potential {
            attack_speed += operator_data.aspd_potential.value;
        }

        // Apply trust ASPD bonus
        attack_speed += (operator_data.aspd_trust * trust as f64) / 100.0;

        // Apply module bonuses
        if elite == 2
            && level >= max_level_e2 - 30
            && let Some(ref op_module) = operator_module
        {
            let module_id = op_module.module.id.as_deref().unwrap_or("");

            let module_atk = operator_data
                .atk_module
                .iter()
                .find(|m| m.module_id == module_id && m.level as i32 == operator_module_level)
                .map(|m| m.value)
                .unwrap_or(0);

            let module_aspd = operator_data
                .aspd_module
                .iter()
                .find(|m| m.module_id == module_id && m.level as i32 == operator_module_level)
                .map(|m| m.value)
                .unwrap_or(0);

            atk += module_atk as f64;
            attack_speed += module_aspd as f64;
        }

        // Set skill parameters
        // Note: skill_index is 1-indexed (1=S1, 2=S2, 3=S3) like Python
        // For array access, we use (skill_index - 1) since Python does skill_parameters[skill-1]
        let mut skill_parameters: Vec<f64> = Vec::new();
        let mut skill_cost = -1;
        let mut skill_duration: f64 = -1.0;

        if rarity > 2 && skill_index >= 1 {
            // Python uses skill_parameters[skill-1] where skill is 1-indexed
            let skill_idx = (skill_index - 1) as usize;
            let skill_lvl = (skill_level - 1) as usize;

            skill_parameters = operator_data
                .skill_parameters
                .get(skill_idx)
                .and_then(|params| params.get(skill_lvl))
                .cloned()
                .unwrap_or_default();

            skill_cost = operator_data
                .skill_costs
                .get(skill_idx)
                .and_then(|costs| costs.get(skill_lvl))
                .copied()
                .unwrap_or(-1);

            skill_duration = operator_data
                .skill_durations
                .get(skill_idx)
                .and_then(|durations| durations.get(skill_lvl))
                .copied()
                .unwrap_or(-1.0);
        }

        // Calculate talent1 parameters
        let mut talent1_parameters = operator_data.talent1_defaults.clone();
        if !operator_data.talent1_parameters.is_empty() {
            let mut current_promo = 0;
            let mut current_req_level = 0;
            let mut current_req_potential = 0;
            let mut current_req_module_lvl = 0;

            for talent_data in &operator_data.talent1_parameters {
                if elite >= talent_data.required_promotion
                    && talent_data.required_promotion >= current_promo
                    && level >= talent_data.required_level
                    && talent_data.required_level >= current_req_level
                {
                    let op_module_id = operator_module
                        .as_ref()
                        .and_then(|m| m.module.id.as_deref())
                        .unwrap_or("");

                    if op_module_id.is_empty() {
                        if talent_data.required_module_id.is_empty()
                            && potential > talent_data.required_potential
                            && potential > current_req_potential
                        {
                            talent1_parameters = talent_data.talent_data.clone();
                            current_promo = talent_data.required_promotion;
                            current_req_level = talent_data.required_level;
                            current_req_potential = talent_data.required_potential;
                            current_req_module_lvl = talent_data.required_module_level;
                        }
                    } else {
                        // Operator has a module equipped
                        // Accept entry if:
                        // 1. Entry requires no specific module (required_module_id is empty), OR
                        // 2. Entry requires a specific module that matches the equipped one
                        let module_requirement_satisfied =
                            talent_data.required_module_id.is_empty()
                                || op_module_id == talent_data.required_module_id;

                        if module_requirement_satisfied
                            && operator_module_level >= talent_data.required_module_level
                            && operator_module_level >= current_req_module_lvl
                            && potential > talent_data.required_potential
                            && potential > current_req_potential
                        {
                            talent1_parameters = talent_data.talent_data.clone();
                            current_promo = talent_data.required_promotion;
                            current_req_level = talent_data.required_level;
                            current_req_potential = talent_data.required_potential;
                            current_req_module_lvl = talent_data.required_module_level;
                        }
                    }
                }
            }
            // Suppress unused variable warnings
            let _ = current_promo;
            let _ = current_req_level;
            let _ = current_req_potential;
            let _ = current_req_module_lvl;
        }

        // Calculate talent2 parameters
        let mut talent2_parameters = operator_data.talent2_defaults.clone();
        if !operator_data.talent2_parameters.is_empty() {
            let mut current_promo = 0;
            let mut current_req_level = 0;
            let mut current_req_potential = 0;
            let mut current_req_module_lvl = 0;

            for talent_data in &operator_data.talent2_parameters {
                if elite >= talent_data.required_promotion
                    && talent_data.required_promotion >= current_promo
                    && level >= talent_data.required_level
                    && talent_data.required_level >= current_req_level
                {
                    let op_module_id = operator_module
                        .as_ref()
                        .and_then(|m| m.module.id.as_deref())
                        .unwrap_or("");

                    if op_module_id.is_empty() {
                        if talent_data.required_module_id.is_empty()
                            && potential > talent_data.required_potential
                            && potential > current_req_potential
                        {
                            talent2_parameters = talent_data.talent_data.clone();
                            current_promo = talent_data.required_promotion;
                            current_req_level = talent_data.required_level;
                            current_req_potential = talent_data.required_potential;
                            current_req_module_lvl = talent_data.required_module_level;
                        }
                    } else {
                        // Operator has a module equipped
                        // Accept entry if:
                        // 1. Entry requires no specific module (required_module_id is empty), OR
                        // 2. Entry requires a specific module that matches the equipped one
                        let module_requirement_satisfied =
                            talent_data.required_module_id.is_empty()
                                || op_module_id == talent_data.required_module_id;

                        if module_requirement_satisfied
                            && operator_module_level >= talent_data.required_module_level
                            && operator_module_level >= current_req_module_lvl
                            && potential > talent_data.required_potential
                            && potential > current_req_potential
                        {
                            talent2_parameters = talent_data.talent_data.clone();
                            current_promo = talent_data.required_promotion;
                            current_req_level = talent_data.required_level;
                            current_req_potential = talent_data.required_potential;
                            current_req_module_lvl = talent_data.required_module_level;
                        }
                    }
                }
            }
            // Suppress unused variable warnings
            let _ = current_promo;
            let _ = current_req_level;
            let _ = current_req_potential;
            let _ = current_req_module_lvl;
        }

        // Calculate drone/summon parameters
        // Select drone based on skill_index (skill 1 -> drone 0, skill 2 -> drone 1, etc.)
        let mut drone_atk: f64 = 0.0;
        // Default to 1.0 to avoid division by zero (matches Python's default)
        let mut drone_atk_interval: f32 = 1.0;

        if !operator_data.drone_atk.is_empty() {
            // Python logic: slot = self.skill - 1 if self.skill > 0 else 2
            // If skill == 0, default to drone 3 (slot 2)
            // If fewer than 2 drones exist, always use slot 0
            let slot = if skill_index > 0 {
                (skill_index - 1) as usize
            } else {
                2 // Default to third drone for skill 0
            };

            // Cap at the actual number of drones
            let capped_slot = if operator_data.drone_atk.len() < 2 {
                0 // If fewer than 2 drones, always use first
            } else {
                slot.min(operator_data.drone_atk.len().saturating_sub(1))
            };

            // Get attack interval for selected drone
            drone_atk_interval = operator_data
                .drone_atk_interval
                .get(capped_slot)
                .copied()
                .unwrap_or(1.0); // Default to 1.0 to avoid division by zero

            // Get ATK for selected drone
            if let Some(selected_drone) = operator_data.drone_atk.get(capped_slot) {
                drone_atk = match elite {
                    0 => {
                        let min = selected_drone.e0.min as f64;
                        let max = selected_drone.e0.max as f64;
                        if max_level_for_elite_f64 > 1.0 {
                            min + ((max - min) * (level as f64 - 1.0))
                                / (max_level_for_elite_f64 - 1.0)
                        } else {
                            max
                        }
                    }
                    1 => {
                        let min = selected_drone.e1.min as f64;
                        let max = selected_drone.e1.max as f64;
                        if max_level_for_elite_f64 > 1.0 {
                            min + ((max - min) * (level as f64 - 1.0))
                                / (max_level_for_elite_f64 - 1.0)
                        } else {
                            max
                        }
                    }
                    2 => {
                        let min = selected_drone.e2.min as f64;
                        let max = selected_drone.e2.max as f64;
                        if max_level_for_elite_f64 > 1.0 {
                            min + ((max - min) * (level as f64 - 1.0))
                                / (max_level_for_elite_f64 - 1.0)
                        } else {
                            max
                        }
                    }
                    _ => 0.0,
                };
            }
        }

        // Apply base buffs
        let base_atk = params.base_buffs.atk.unwrap_or(1.0) as f64;
        let base_atk_flat = params.base_buffs.flat_atk.unwrap_or(0) as f64;
        atk = atk * base_atk + base_atk_flat;

        // Build buff name
        let mut buff_name = String::new();

        if base_atk > 1.0 {
            buff_name += &format!(" bAtk+{:.0}%", 100.0 * base_atk);
        } else if base_atk < 1.0 {
            buff_name += &format!(" bAtk{:.0}%", 100.0 * base_atk);
        }

        let buff_atk = params.buffs.atk.unwrap_or(0.0) as f64;
        if buff_atk > 1.0 {
            buff_name += &format!(" atk+{:.0}%", 100.0 * buff_atk);
        } else if buff_atk < 1.0 && buff_atk != 0.0 {
            buff_name += &format!(" atk{:.0}%", 100.0 * buff_atk);
        }

        let buffs_aspd = params.buffs.aspd.unwrap_or(0) as f64;
        attack_speed += buffs_aspd;
        if buffs_aspd > 1.0 {
            buff_name += &format!(" aspd+{:.0}%", 100.0 * buffs_aspd);
        } else if buffs_aspd < 1.0 && buffs_aspd != 0.0 {
            buff_name += &format!(" aspd{:.0}%", 100.0 * buffs_aspd);
        }

        let buff_atk_flat = params.buffs.flat_atk.unwrap_or(0) as f64;
        if buff_atk_flat > 0.0 {
            buff_name += &format!(" atk+{buff_atk_flat:.0}");
        } else if buff_atk_flat < 0.0 {
            buff_name += &format!(" atk{buff_atk_flat:.0}");
        }

        let buff_fragile = params.buffs.fragile.unwrap_or(0.0) as f64;
        if buff_fragile > 1.0 {
            buff_name += &format!(" dmg+{:.0}%", 100.0 * buff_fragile);
        } else if buff_fragile < 1.0 && buff_fragile != 0.0 {
            buff_name += &format!(" dmg{:.0}%", 100.0 * buff_fragile);
        }

        if sp_boost > 0.0 {
            buff_name += &format!(" +{sp_boost:.0}SP/s");
        }

        // Calculate shred values from params
        // shreds format: [def_mult, def_flat, res_mult, res_flat]
        // def/res are percentages (e.g., 40 = -40% DEF/RES), so multiplier = 1 - (value/100)
        let (shred_def_mult, shred_def_flat, shred_res_mult, shred_res_flat) =
            if let Some(ref shred) = params.shred {
                let def_percent = shred.def.unwrap_or(0);
                let def_flat = shred.def_flat.unwrap_or(0);
                let res_percent = shred.res.unwrap_or(0);
                let res_flat = shred.res_flat.unwrap_or(0);

                // Build buff name for display
                if def_percent != 0 {
                    buff_name += &format!(" -{def_percent}%def");
                }
                if def_flat != 0 {
                    buff_name += &format!(" -{def_flat}def");
                }
                if res_percent != 0 {
                    buff_name += &format!(" -{res_percent}%res");
                }
                if res_flat != 0 {
                    buff_name += &format!(" -{res_flat}res");
                }

                // Convert percentage to multiplier: 40% shred = 0.6 multiplier
                let def_mult = 1.0 - (def_percent as f64 / 100.0);
                let res_mult = 1.0 - (res_percent as f64 / 100.0);

                (def_mult, def_flat as f64, res_mult, res_flat as f64)
            } else {
                // Default: no shred applied (1.0 multiplier, 0 flat)
                (1.0, 0.0, 1.0, 0.0)
            };

        Self {
            data: operator_data,
            params,

            default_skill_index,
            default_potential,
            default_module_index,

            available_skills,

            targets,
            rarity,

            atk,
            attack_interval,
            attack_speed,

            elite,
            level,
            potential,
            skill_level,
            trust,

            operator_module,
            operator_module_level,
            module_index,
            module_level,

            skill_index,
            skill_parameters,
            skill_duration,
            skill_cost,

            sp_boost,

            is_physical,
            is_ranged,

            talent1_parameters,
            talent2_parameters,

            drone_atk,
            drone_atk_interval,

            trait_damage,
            trait_damage_name: None,
            trait_damage_names: Vec::new(),

            talent_damage,
            talent_damage_name: None,
            talent_damage_names: Vec::new(),

            talent2_damage,
            talent2_damage_name: None,
            talent2_damage_names: Vec::new(),

            skill_damage,
            skill_damage_name: None,
            skill_damage_names: Vec::new(),

            module_damage,
            module_damage_name: None,
            module_damage_names: Vec::new(),

            buff_name,
            buff_atk,
            buff_atk_flat,
            buff_fragile,

            // Shreds: [def_mult, def_flat, res_mult, res_flat]
            shreds: vec![
                shred_def_mult,
                shred_def_flat,
                shred_res_mult,
                shred_res_flat,
            ],

            // Default ammo (overridden by operator __init__ if needed)
            ammo: 1.0,
        }
    }

    pub fn normal_attack(
        &self,
        enemy: &EnemyStats,
        extra_buffs: Option<&ExtraBuffs>,
        hits: Option<i32>,
        aoe: Option<i32>,
    ) -> f64 {
        let default_buffs = ExtraBuffs::default();
        let extra = extra_buffs.unwrap_or(&default_buffs);
        let hits = hits.unwrap_or(1) as f64;
        let aoe = aoe.unwrap_or(1) as f64;

        let final_atk =
            self.atk * (1.0 + extra.atk + self.buff_atk) + extra.flat_atk + self.buff_atk_flat;

        let hit_dmg = if !self.is_physical {
            (final_atk * (1.0 - enemy.res / 100.0)).max(final_atk * 0.05)
        } else {
            (final_atk - enemy.defense).max(final_atk * 0.05)
        };

        ((hits * hit_dmg) / self.attack_interval as f64)
            * ((self.attack_speed + extra.aspd) / 100.0)
            * aoe
    }

    pub fn skill_dps(&self, _enemy: &EnemyStats) -> f64 {
        // This method should be overridden by specific operator implementations
        panic!("Not implemented");
    }

    pub fn total_dmg(&self, enemy: &EnemyStats) -> f64 {
        if self.skill_duration < 1.0 || self.skill_index == -1 {
            self.skill_dps(enemy)
        } else {
            self.skill_dps(enemy) * self.skill_duration
        }
    }

    pub fn average_dps(&mut self, enemy: &EnemyStats) -> f64 {
        if self.skill_duration < 1.0 || self.skill_index == -1 {
            self.skill_dps(enemy)
        } else {
            let tmp = self.skill_index;
            let skill_dps = self.skill_dps(enemy);

            self.skill_index = -1;
            let off_skill_dps = self.skill_dps(enemy);
            self.skill_index = tmp;

            let cycle_dmg = skill_dps * self.skill_duration
                + (off_skill_dps * self.skill_cost as f64) / (1.0 + self.sp_boost as f64);

            cycle_dmg
                / (self.skill_duration + self.skill_cost as f64 / (1.0 + self.sp_boost as f64))
        }
    }
}

#[derive(Default)]
pub struct EnemyStats {
    pub defense: f64,
    pub res: f64,
}

/// Trait for all DPS calculator operators
/// This enables dynamic dispatch and registry-based operator lookup
pub trait DpsCalculator {
    /// Calculate DPS against the given enemy stats
    fn skill_dps(&self, enemy: &EnemyStats) -> f64;

    /// Get a reference to the underlying OperatorUnit
    fn unit(&self) -> &OperatorUnit;

    /// Get a mutable reference to the underlying OperatorUnit
    fn unit_mut(&mut self) -> &mut OperatorUnit;
}

/// Type alias for operator constructor functions
pub type OperatorConstructor =
    fn(OperatorData, OperatorParams) -> Box<dyn DpsCalculator + Send + Sync>;

#[derive(Default)]
pub struct ExtraBuffs {
    pub atk: f64,
    pub flat_atk: f64,
    pub aspd: f64,
}

pub struct OperatorParams {
    pub potential: Option<i32>,
    pub promotion: Option<i32>,
    pub level: Option<i32>,
    pub trust: Option<i32>,

    pub skill_index: Option<i32>,
    pub mastery_level: Option<i32>,

    pub module_index: Option<i32>,
    pub module_level: Option<i32>,

    pub buffs: OperatorBuffs,
    pub base_buffs: OperatorBaseBuffs,

    pub sp_boost: Option<f32>,

    pub targets: Option<i32>,
    pub enemies: Option<Vec<Enemy>>,

    pub conditionals: Option<OperatorConditionals>,
    pub all_cond: Option<bool>,

    pub graph_type: Option<i32>,
    pub fix_value: Option<i32>,

    pub max_def: Option<i32>,
    pub max_res: Option<i32>,
    pub res: Option<i32>,
    pub def: Option<i32>,

    pub shred: Option<OperatorShred>,

    pub normal_dps: Option<f64>,
}

impl Default for OperatorParams {
    fn default() -> Self {
        Self {
            potential: None,
            promotion: None,
            level: None,
            trust: Some(100),
            skill_index: None,
            mastery_level: None,
            module_index: None,
            module_level: None,
            buffs: OperatorBuffs::default(),
            base_buffs: OperatorBaseBuffs::default(),
            sp_boost: Some(0.0),
            targets: Some(1),
            enemies: None,
            conditionals: None,
            all_cond: Some(true),
            graph_type: Some(0),
            fix_value: Some(40),
            max_def: Some(3000),
            max_res: Some(120),
            res: None,
            def: None,
            shred: None,
            normal_dps: Some(0.0),
        }
    }
}

/// 0: ATK buff as a percentage decimal (eg. 0.4)
/// 1: Flat ATK buff (eg. 102)
/// 2: ASPD buff (eg. 52)
/// 3: Fragile debuff as a percentage decimal (eg. 0.3)
#[derive(Default)]
pub struct OperatorBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
    pub aspd: Option<i32>,
    pub fragile: Option<f32>,
}

/// 0: ATK buff as a percentage decimal (eg. 0.4)
/// 1: Flat ATK buff (eg. 102)
#[derive(Default)]
pub struct OperatorBaseBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
}

pub struct OperatorConditionals {
    pub trait_damage: Option<bool>,
    pub talent_damage: Option<bool>,
    pub talent2_damage: Option<bool>,
    pub skill_damage: Option<bool>,
    pub module_damage: Option<bool>,
}

impl Default for OperatorConditionals {
    fn default() -> Self {
        Self {
            trait_damage: Some(true),
            talent_damage: Some(true),
            talent2_damage: Some(true),
            skill_damage: Some(true),
            module_damage: Some(true),
        }
    }
}

/// 0: DEF shred as a percentage decimal (eg. 0.4)
/// 1: Flat DEF shred (eg. 102)
/// 2: RES shred as a percentage decimal (eg. 0.4)
/// 3: Flat RES shred (eg. 102)
#[derive(Default)]
pub struct OperatorShred {
    pub def: Option<i32>,
    pub def_flat: Option<i32>,
    pub res: Option<i32>,
    pub res_flat: Option<i32>,
}
