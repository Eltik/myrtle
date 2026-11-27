use myrtle_gamedata_raw::tables::operator::{Operator, OperatorPhase};

use crate::{
    PlanError,
    progress::{parse_target_modules, parse_target_skills, reaches_promotion},
    types::{PlanInput, PresetTarget},
};

/// Raidian's progression is locked to Integrated Strategies 6
const UNPLANNABLE_OPERATOR: &str = "char_4195_radian";

const MAX_ELITE: i16 = 2;
const LEVEL_CAPS: [i16; 3] = [50, 80, 90];
const MAX_SKILL_LEVEL: i16 = 7;
const MAX_SKILL_LEVEL_AT_ELITE_0: i16 = 4;
const MAX_MASTERY: i16 = 3;
const MAX_MODULE_STAGE: i16 = 3;
const MAX_PRESET_NAME_CHARS: usize = 100;

const fn phase_to_int(phase: &OperatorPhase) -> i16 {
    match phase {
        OperatorPhase::Elite0 => 0,
        OperatorPhase::Elite1 => 1,
        OperatorPhase::Elite2 => 2,
    }
}

#[must_use]
pub fn module_phase_to_int(phase: &str) -> i16 {
    match phase {
        "PHASE_1" | "1" => 1,
        "PHASE_2" | "2" => 2,
        _ => 0,
    }
}

pub fn ensure_plannable_id(operator_id: &str) -> Result<(), PlanError> {
    if operator_id == UNPLANNABLE_OPERATOR {
        return Err(PlanError::Invalid("This operator is unplannable".into()));
    }
    Ok(())
}

pub fn validate_plan(operator: &Operator, plan: &PlanInput) -> Result<(), PlanError> {
    if operator.is_not_obtainable {
        return Err(PlanError::Invalid(format!(
            "Operator {} is not obtainable, and their upgrades cannot be planned",
            operator.name
        )));
    }

    let target_elite = plan.target_elite;
    let target_level = plan.target_level;
    let target_skill_level = plan.target_skill_level;

    let max_elite = (operator.phases.len() as i16) - 1;
    if target_elite < 0 || target_elite > max_elite {
        return Err(PlanError::Invalid(format!(
            "Invalid target elite promotion for operator {} (max: {})",
            operator.name, max_elite
        )));
    }

    let phase = &operator.phases[target_elite as usize];
    if target_level < 1 || target_level > (phase.max_level as i16) {
        return Err(PlanError::Invalid(format!(
            "Invalid target level for operator {} (max: {})",
            operator.name, phase.max_level
        )));
    }

    let max_skill_level = (operator.all_skill_level_up.len() + 1) as i16;
    if target_skill_level < 1 || target_skill_level > max_skill_level {
        return Err(PlanError::Invalid(format!(
            "Invalid target skill level for operator {} (max: {})",
            operator.name, max_skill_level
        )));
    }

    for idx in 2..=target_skill_level {
        if let Some(lvl_up) = operator.all_skill_level_up.get((idx - 2) as usize) {
            let required_phase = phase_to_int(&lvl_up.unlock_cond.phase);
            if required_phase > target_elite {
                return Err(PlanError::Invalid(format!(
                    "Target skill level {idx} requires Elite {required_phase} or higher"
                )));
            }
            let required_level = lvl_up.unlock_cond.level as i16;
            if required_phase == target_elite && required_level > target_level {
                return Err(PlanError::Invalid(format!(
                    "Target skill level {idx} requires level {required_level} at Elite {required_phase}"
                )));
            }
        }
    }

    for skill in &parse_target_skills(&plan.target_skills)? {
        if skill.skill_index < 0 || skill.skill_index >= (operator.skills.len() as i16) {
            return Err(PlanError::Invalid(format!(
                "Invalid skill index {} for operator {}",
                skill.skill_index, operator.name,
            )));
        }

        let skill_entry = &operator.skills[skill.skill_index as usize];
        let max_mastery = skill_entry.level_up_cost_cond.len() as i16;
        if skill.mastery_level < 0 || skill.mastery_level > max_mastery {
            return Err(PlanError::Invalid(format!(
                "Skill mastery must be between 0 and {} for operator {} skill index {}",
                max_mastery, operator.name, skill.skill_index
            )));
        }

        if skill.mastery_level > 0 && target_skill_level < MAX_SKILL_LEVEL {
            return Err(PlanError::Invalid(format!(
                "Mastery {} for operator {} skill index {} requires target skill level {MAX_SKILL_LEVEL}",
                skill.mastery_level, operator.name, skill.skill_index
            )));
        }

        for m in 1..=skill.mastery_level {
            if let Some(cond) = skill_entry.level_up_cost_cond.get((m - 1) as usize) {
                let required_phase = phase_to_int(&cond.unlock_cond.phase);
                let required_level = cond.unlock_cond.level as i16;
                if !reaches_promotion(target_elite, target_level, required_phase, required_level) {
                    return Err(PlanError::Invalid(format!(
                        "Mastery {} for operator {} skill index {} requires Elite {} level {}",
                        skill.mastery_level,
                        operator.name,
                        skill.skill_index,
                        required_phase,
                        required_level
                    )));
                }
            }
        }
    }

    for module in &parse_target_modules(&plan.target_modules)? {
        let op_mod = operator
            .modules
            .iter()
            .find(|m| m.module.uni_equip_id == module.module_id)
            .ok_or_else(|| {
                PlanError::Invalid(format!(
                    "Invalid module ID {} for operator {}",
                    module.module_id, operator.name
                ))
            })?;

        if !(0..=MAX_MODULE_STAGE).contains(&module.module_stage) {
            return Err(PlanError::Invalid(format!(
                "Module stage must be between 0 and {MAX_MODULE_STAGE}"
            )));
        }

        if module.module_stage > 0 {
            let required_phase = module_phase_to_int(&op_mod.module.unlock_evolve_phase);
            let required_level = op_mod.module.unlock_level as i16;
            if !reaches_promotion(target_elite, target_level, required_phase, required_level) {
                return Err(PlanError::Invalid(format!(
                    "Module unlock/upgrades require Elite {required_phase} level {required_level}."
                )));
            }
        }
    }

    Ok(())
}

pub fn validate_preset(name: &str, target: &PresetTarget) -> Result<(), PlanError> {
    if name.is_empty() || name.chars().count() > MAX_PRESET_NAME_CHARS {
        return Err(PlanError::Invalid(format!(
            "Preset name must be 1 to {MAX_PRESET_NAME_CHARS} characters"
        )));
    }
    if !(0..=MAX_ELITE).contains(&target.elite) {
        return Err(PlanError::Invalid(format!(
            "elite must be between 0 and {MAX_ELITE}"
        )));
    }
    let cap = LEVEL_CAPS[target.elite as usize];
    if let Some(level) = target.level
        && !(1..=cap).contains(&level)
    {
        return Err(PlanError::Invalid(format!(
            "level must be between 1 and {cap} at Elite {} or null for the cap",
            target.elite
        )));
    }
    if !(1..=MAX_SKILL_LEVEL).contains(&target.skill_level) {
        return Err(PlanError::Invalid(format!(
            "skill_level must be between 1 and {MAX_SKILL_LEVEL}"
        )));
    }
    if target.skill_level > MAX_SKILL_LEVEL_AT_ELITE_0 && target.elite < 1 {
        return Err(PlanError::Invalid(format!(
            "skill_level above {MAX_SKILL_LEVEL_AT_ELITE_0} requires Elite 1"
        )));
    }
    if target
        .masteries
        .iter()
        .any(|m| !(0..=MAX_MASTERY).contains(m))
    {
        return Err(PlanError::Invalid(format!(
            "each mastery must be between 0 and {MAX_MASTERY}"
        )));
    }
    if target.masteries.iter().any(|&m| m > 0)
        && (target.elite < MAX_ELITE || target.skill_level < MAX_SKILL_LEVEL)
    {
        return Err(PlanError::Invalid(format!(
            "masteries require Elite {MAX_ELITE} and skill_level {MAX_SKILL_LEVEL}"
        )));
    }
    if !(0..=MAX_MODULE_STAGE).contains(&target.module_stage) {
        return Err(PlanError::Invalid(format!(
            "module_stage must be between 0 and {MAX_MODULE_STAGE}"
        )));
    }
    if target.module_stage > 0 && target.elite < MAX_ELITE {
        return Err(PlanError::Invalid(format!(
            "modules require Elite {MAX_ELITE}"
        )));
    }
    Ok(())
}
