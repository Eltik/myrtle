use crate::core::local::types::{
    module::ModuleTarget,
    operator::{
        Operator, OperatorModule, OperatorPhase, OperatorPosition, OperatorProfession,
        OperatorRarity, Phase,
    },
};

const ZERO_DEFAULT_KEYS: &[&str] = &[
    "atk",
    "prob",
    "duration",
    "attack_speed",
    "attack@prob",
    "magic_resistance",
    "sp_recovery_per_sec",
    "base_attack_time",
    "magic_resist_penetrate_fixed",
];

pub struct OperatorData {
    pub data: Operator,

    pub rarity: i32,
    pub atk_interval: f32,
    pub available_modules: Vec<OperatorModule>,

    pub is_physical: bool,
    pub is_ranged: bool,

    // Attack values
    pub atk: OperatorAtk,
    pub atk_potential: PotentialValues,

    pub atk_module: Vec<ModuleValues>,

    pub atk_trust: f64,

    // ASPD values
    pub aspd_potential: PotentialValues,
    pub aspd_module: Vec<ModuleValues>,
    pub aspd_trust: f64,

    // Skill values
    pub skill_parameters: Vec<Vec<Vec<f64>>>,
    pub skill_durations: Vec<Vec<f64>>,
    pub skill_costs: Vec<Vec<i32>>,

    // Talent values
    pub has_second_talent: bool,

    pub talent1_parameters: Vec<TalentParameters>,
    pub talent2_parameters: Vec<TalentParameters>,

    pub talent1_defaults: Vec<f64>,
    pub talent2_defaults: Vec<f64>,

    // Module values
    pub talent1_module_extra: Vec<OperatorModuleExtra>,
    pub talent2_module_extra: Vec<OperatorModuleExtra>,

    // Summon-specific
    pub drone_atk: OperatorAtk,
    pub drone_atk_interval: Vec<f32>,
}

impl OperatorData {
    pub fn new(operator: Operator) -> Self {
        let rarity = Self::rarity_to_number(&operator.rarity);
        let atk_interval = operator
            .phases
            .first()
            .and_then(|p| p.attributes_key_frames.first())
            .map(|kf| kf.data.base_attack_time as f32)
            .unwrap_or(1.6);

        let atk = OperatorAtk {
            e0: Self::get_phase_atk(operator.phases.first()),
            e1: if rarity > 2 {
                Self::get_phase_atk(operator.phases.get(1))
            } else {
                MinMax::default()
            },
            e2: if rarity > 3 {
                Self::get_phase_atk(operator.phases.get(2))
            } else {
                MinMax::default()
            },
        };

        let atk_trust = operator
            .favor_key_frames
            .get(1)
            .map(|kf| kf.data.atk as f64)
            .unwrap_or(0.0);

        let mut atk_potential = PotentialValues {
            required_potential: 0,
            value: 0.0,
        };

        let mut aspd_potential = PotentialValues {
            required_potential: 0,
            value: 0.0,
        };

        for (i, potential) in operator.potential_ranks.iter().enumerate() {
            if potential.potential_type != "BUFF" {
                continue;
            }

            let modifier = potential
                .buff
                .as_ref()
                .and_then(|b| b.attributes.attribute_modifiers.as_ref())
                .and_then(|mods| mods.first());

            if let Some(mod_data) = modifier {
                if mod_data.attribute_type == "ATK" {
                    atk_potential = PotentialValues {
                        required_potential: (i + 2) as i32,
                        value: mod_data.value,
                    };
                }

                if mod_data.attribute_type == "ATTACK_SPEED" {
                    aspd_potential = PotentialValues {
                        required_potential: (i + 2) as i32,
                        value: mod_data.value,
                    };
                }
            }
        }

        let aspd_trust = operator
            .favor_key_frames
            .get(1)
            .map(|kf| kf.data.attack_speed)
            .unwrap_or(0.0);

        let is_physical = if operator.profession == OperatorProfession::Caster
            || operator.profession == OperatorProfession::Medic
            || operator.profession == OperatorProfession::Supporter
        {
            false
        } else if operator.sub_profession_id == "craftsman" {
            true
        } else {
            operator.sub_profession_id != "artsfghter"
        };

        let is_ranged = operator.position != OperatorPosition::Melee;

        let mut skill_parameters: Vec<Vec<Vec<f64>>> = Vec::new();
        let mut skill_durations: Vec<Vec<f64>> = Vec::new();
        let mut skill_costs: Vec<Vec<i32>> = Vec::new();

        for skill in &operator.skills {
            let mut current_skill_params: Vec<Vec<f64>> = Vec::new();
            let mut current_skill_durations: Vec<f64> = Vec::new();
            let mut current_skill_costs: Vec<i32> = Vec::new();

            if let Some(static_data) = &skill.static_data {
                for skill_level in &static_data.levels {
                    current_skill_durations.push(skill_level.duration);
                    current_skill_costs.push(skill_level.sp_data.sp_cost);

                    let current_input: Vec<f64> = skill_level
                        .blackboard
                        .iter()
                        .map(|entry| entry.value)
                        .collect();

                    current_skill_params.push(current_input);
                }
            }

            skill_parameters.push(current_skill_params);
            skill_durations.push(current_skill_durations);
            skill_costs.push(current_skill_costs);
        }

        let has_second_talent = operator.talents.len() > 1;

        let talent1_name = operator
            .talents
            .first()
            .and_then(|p| p.candidates.first())
            .and_then(|c| c.name.as_deref());
        let talent2_name = if has_second_talent {
            operator
                .talents
                .get(1)
                .and_then(|p| p.candidates.first())
                .and_then(|c| c.name.as_deref())
        } else {
            None
        };

        let mut talent1_parameters: Vec<TalentParameters> = Vec::new();
        let mut talent2_parameters: Vec<TalentParameters> = Vec::new();

        let talent1_candidates = operator
            .talents
            .first()
            .map(|t| t.candidates.as_slice())
            .unwrap_or(&[]);

        for candidate in talent1_candidates {
            let params = TalentParameters {
                required_promotion: Self::phase_to_number(&candidate.unlock_condition.phase),
                required_level: candidate.unlock_condition.level,
                required_module_id: String::new(),
                required_module_level: -1,
                required_potential: candidate.required_potential_rank,
                talent_data: candidate.blackboard.iter().map(|b| b.value).collect(),
            };

            talent1_parameters.push(params);
        }

        if has_second_talent {
            let talent2_candidates = operator
                .talents
                .get(1)
                .map(|t| t.candidates.as_slice())
                .unwrap_or(&[]);

            for candidate in talent2_candidates {
                let params = TalentParameters {
                    required_promotion: Self::phase_to_number(&candidate.unlock_condition.phase),
                    required_level: candidate.unlock_condition.level,
                    required_module_id: String::new(),
                    required_module_level: -1,
                    required_potential: candidate.required_potential_rank,
                    talent_data: candidate.blackboard.iter().map(|b| b.value).collect(),
                };

                talent2_parameters.push(params);
            }
        }

        let mut talent1_defaults: Vec<f64> = Vec::new();
        let mut talent2_defaults: Vec<f64> = Vec::new();

        let talent1_last_blackboard = operator
            .talents
            .first()
            .and_then(|t| t.candidates.last())
            .map(|c| c.blackboard.as_slice())
            .unwrap_or(&[]);

        for data in talent1_last_blackboard {
            if ZERO_DEFAULT_KEYS.contains(&data.key.as_str()) {
                talent1_defaults.push(0.0);
            } else {
                talent1_defaults.push(1.0);
            }
        }

        if has_second_talent {
            let talent2_last_blackboard = operator
                .talents
                .get(1)
                .and_then(|t| t.candidates.last())
                .map(|c| c.blackboard.as_slice())
                .unwrap_or(&[]);
            for data in talent2_last_blackboard {
                if ZERO_DEFAULT_KEYS.contains(&data.key.as_str()) {
                    talent2_defaults.push(0.0);
                } else {
                    talent2_defaults.push(1.0);
                }
            }
        }

        let mut available_modules: Vec<OperatorModule> = Vec::new();
        let mut atk_module: Vec<ModuleValues> = Vec::new();
        let mut aspd_module: Vec<ModuleValues> = Vec::new();

        for op_module in &operator.modules {
            available_modules.push(op_module.clone());

            for mod_level in &op_module.data.phases {
                for data in &mod_level.attribute_blackboard {
                    let module_value = ModuleValues {
                        module_id: op_module.module.id.clone().unwrap_or_default(),
                        value: data.value as i64,
                        level: mod_level.equip_level as f64,
                    };

                    if data.key == "atk" {
                        atk_module.push(module_value);
                    } else if data.key == "attack_speed" {
                        aspd_module.push(module_value.clone());
                    }
                }
            }
        }

        let op_id_suffix = operator
            .id
            .as_ref()
            .and_then(|id| id.split("_").nth(2))
            .unwrap_or("");

        let mut talent1_module_extra: Vec<OperatorModuleExtra> = Vec::new();
        let mut talent2_module_extra: Vec<OperatorModuleExtra> = Vec::new();

        for module_prefix in &["uniequip_002_", "uniequip_003_", "uniequip_004_"] {
            let module_key = format!("{module_prefix}{op_id_suffix}");

            if let Some(op_module) = operator
                .modules
                .iter()
                .find(|m| m.module.id.as_deref() == Some(module_key.as_str()))
            {
                Self::process_module_talents(
                    op_module,
                    talent1_name,
                    talent2_name,
                    &mut talent1_parameters,
                    &mut talent2_parameters,
                    &mut talent1_module_extra,
                    &mut talent2_module_extra,
                );
            }
        }

        let mut drone_atk = OperatorAtk {
            e0: MinMax::default(),
            e1: MinMax::default(),
            e2: MinMax::default(),
        };
        let mut drone_atk_interval: Vec<f32> = Vec::new();

        for drone_data in &operator.drones {
            // Get attack interval
            if let Some(base_attack_time) = drone_data
                .phases
                .first()
                .and_then(|p| p.attributes_key_frames.first())
                .map(|kf| kf.data.base_attack_time as f32)
            {
                drone_atk_interval.push(base_attack_time);
            }

            // E0 ATK values
            drone_atk.e0 = Self::get_phase_atk(drone_data.phases.first());

            // E1 ATK values (rarity > 2)
            if rarity > 2 {
                drone_atk.e1 = Self::get_phase_atk(drone_data.phases.get(1));
            }

            // E2 ATK values (rarity > 3)
            if rarity > 3 {
                drone_atk.e2 = Self::get_phase_atk(drone_data.phases.get(2));
            }
        }

        Self {
            data: operator,

            rarity,

            atk_interval,
            atk,
            atk_trust,
            atk_potential,

            aspd_potential,
            aspd_trust,

            is_physical,
            is_ranged,

            skill_parameters,
            skill_durations,
            skill_costs,

            has_second_talent,
            talent1_parameters,
            talent2_parameters,
            talent1_defaults,
            talent2_defaults,

            available_modules,
            atk_module,
            aspd_module,
            talent1_module_extra,
            talent2_module_extra,

            drone_atk,
            drone_atk_interval,
        }
    }

    fn rarity_to_number(rarity: &OperatorRarity) -> i32 {
        match rarity {
            OperatorRarity::SixStar => 6,
            OperatorRarity::FiveStar => 5,
            OperatorRarity::FourStar => 4,
            OperatorRarity::ThreeStar => 3,
            OperatorRarity::TwoStar => 2,
            OperatorRarity::OneStar => 1,
        }
    }

    fn get_phase_atk(phase: Option<&Phase>) -> MinMax {
        let min = phase
            .and_then(|p| p.attributes_key_frames.first())
            .map(|kf| kf.data.atk)
            .unwrap_or(0);

        let max = phase
            .and_then(|p| p.attributes_key_frames.get(1))
            .map(|kf| kf.data.atk)
            .unwrap_or(0);

        MinMax { min, max }
    }

    fn phase_to_number(phase: &OperatorPhase) -> i32 {
        match phase {
            OperatorPhase::Elite0 => 0,
            OperatorPhase::Elite1 => 1,
            OperatorPhase::Elite2 => 2,
        }
    }

    fn process_module_talents(
        operator_module: &OperatorModule,
        talent1_name: Option<&str>,
        talent2_name: Option<&str>,
        talent1_parameters: &mut Vec<TalentParameters>,
        talent2_parameters: &mut Vec<TalentParameters>,
        talent1_module_extra: &mut Vec<OperatorModuleExtra>,
        talent2_module_extra: &mut Vec<OperatorModuleExtra>,
    ) {
        for module_level in &operator_module.data.phases {
            let equip_level = module_level.equip_level;

            for part in &module_level.parts {
                if part.target != ModuleTarget::Talent
                    && part.target != ModuleTarget::TalentDataOnly
                {
                    continue;
                }

                let candidates = part
                    .add_or_override_talent_data_bundle
                    .candidates
                    .as_deref()
                    .unwrap_or(&[]);

                for candidate in candidates {
                    let candidate_name = Some(candidate.name.as_str());
                    let prefab_key = candidate.prefab_key.as_deref();

                    let is_primary_talent = prefab_key == Some("1") || prefab_key == Some("2");

                    let talent_data: Vec<f64> =
                        candidate.blackboard.iter().map(|b| b.value).collect();

                    if is_primary_talent {
                        let params = TalentParameters {
                            required_promotion: 2,
                            required_level: candidate.unlock_condition.level,
                            required_module_id: operator_module
                                .module
                                .id
                                .clone()
                                .unwrap_or_default(),
                            required_module_level: equip_level,
                            required_potential: candidate.required_potential_rank,
                            talent_data,
                        };

                        if candidate_name == talent1_name {
                            talent1_parameters.push(params);
                        } else if candidate_name == talent2_name {
                            talent2_parameters.push(params);
                        }
                    } else {
                        let extra = OperatorModuleExtra {
                            required_module_level: equip_level,
                            talent_data,
                        };

                        if candidate_name == talent1_name {
                            talent1_module_extra.push(extra);
                        } else if candidate_name == talent2_name {
                            talent2_module_extra.push(extra);
                        }
                    }
                }
            }
        }
    }
}

pub struct OperatorAtk {
    pub e0: MinMax,
    pub e1: MinMax,
    pub e2: MinMax,
}

#[derive(Default)]
pub struct MinMax {
    pub min: i32,
    pub max: i32,
}

pub struct PotentialValues {
    pub required_potential: i32,
    pub value: f64,
}

#[derive(Clone)]
pub struct ModuleValues {
    pub module_id: String,
    pub value: i64,
    pub level: f64,
}

pub struct TalentParameters {
    pub required_promotion: i32,
    pub required_level: i32,
    pub required_module_id: String,
    pub required_module_level: i32,
    pub required_potential: i32,
    pub talent_data: Vec<f64>,
}

pub struct OperatorModuleExtra {
    pub required_module_level: i32,
    pub talent_data: Vec<f64>,
}
