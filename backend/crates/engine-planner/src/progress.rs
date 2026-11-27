use serde::Deserialize;
use std::collections::HashMap;

use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::operator::Operator;
use myrtle_types::roster::RosterEntry;

use crate::{
    EXP_ITEM, LMD_ITEM, PlanError,
    types::{OperatorPlan, PlanInput, TargetModulePlan, TargetSkillPlan},
};

#[derive(Deserialize)]
struct RosterMastery {
    index: i16,
    mastery: i16,
}

#[derive(Deserialize)]
struct RosterModule {
    id: String,
    level: i16,
    #[serde(default)]
    locked: bool,
}

#[must_use]
pub const fn reaches_promotion(
    elite: i16,
    level: i16,
    required_elite: i16,
    required_level: i16,
) -> bool {
    elite > required_elite || (elite == required_elite && level >= required_level)
}

struct CurrentState {
    owned: bool,
    elite: i16,
    level: i16,
    skill_level: i16,
    masteries: Vec<RosterMastery>,
    modules: Vec<RosterModule>,
}

impl CurrentState {
    fn from_roster(entry: Option<&RosterEntry>) -> Self {
        Self {
            owned: entry.is_some(),
            elite: entry.map_or(0, |r| r.elite),
            level: entry.map_or(1, |r| r.level),
            skill_level: entry.map_or(1, |r| r.skill_level),
            masteries: entry
                .map(|r| serde_json::from_value(r.masteries.clone()).unwrap_or_default())
                .unwrap_or_default(),
            modules: entry
                .map(|r| serde_json::from_value(r.modules.clone()).unwrap_or_default())
                .unwrap_or_default(),
        }
    }

    fn mastery(&self, skill_index: i16) -> i16 {
        self.masteries
            .iter()
            .find(|m| m.index == skill_index)
            .map_or(0, |m| m.mastery)
    }

    // A locked module is stage 0
    fn module_stage(&self, module_id: &str) -> i16 {
        self.modules
            .iter()
            .find(|m| m.id == module_id)
            .map_or(0, |m| if m.locked { 0 } else { m.level })
    }

    const fn reaches_level(&self, target_elite: i16, target_level: i16) -> bool {
        reaches_promotion(self.elite, self.level, target_elite, target_level)
    }

    const fn reaches_skill_level(&self, target: i16) -> bool {
        self.skill_level >= target
    }

    fn reaches_mastery(&self, target: &TargetSkillPlan) -> bool {
        self.mastery(target.skill_index) >= target.mastery_level
    }

    fn reaches_module(&self, target: &TargetModulePlan) -> bool {
        self.module_stage(&target.module_id) >= target.module_stage
    }
}

struct PlanTargets {
    skills: Vec<TargetSkillPlan>,
    modules: Vec<TargetModulePlan>,
}

impl PlanTargets {
    fn parse(plan: &OperatorPlan) -> Result<Self, PlanError> {
        Ok(Self {
            skills: parse_target_skills(&plan.target_skills)?,
            modules: parse_target_modules(&plan.target_modules)?,
        })
    }
}

pub fn parse_target_skills(
    target_skills: &serde_json::Value,
) -> Result<Vec<TargetSkillPlan>, PlanError> {
    serde_json::from_value(target_skills.clone())
        .map_err(|_| PlanError::Invalid("Invalid target_skills format".into()))
}

pub fn parse_target_modules(
    target_modules: &serde_json::Value,
) -> Result<Vec<TargetModulePlan>, PlanError> {
    serde_json::from_value(target_modules.clone())
        .map_err(|_| PlanError::Invalid("Invalid target_modules format".into()))
}

fn plan_met(plan: &OperatorPlan, targets: &PlanTargets, current: &CurrentState) -> bool {
    current.owned
        && current.reaches_level(plan.target_elite, plan.target_level)
        && current.reaches_skill_level(plan.target_skill_level)
        && targets.skills.iter().all(|t| current.reaches_mastery(t))
        && targets.modules.iter().all(|t| current.reaches_module(t))
}

pub fn raise_to_roster(
    input: &mut PlanInput,
    roster_entry: Option<&RosterEntry>,
) -> Result<(), PlanError> {
    let Some(entry) = roster_entry else {
        return Ok(());
    };
    let current = CurrentState::from_roster(Some(entry));

    if current.reaches_level(input.target_elite, input.target_level) {
        input.target_elite = current.elite;
        input.target_level = current.level;
    }
    input.target_skill_level = input.target_skill_level.max(current.skill_level);

    let mut skills = parse_target_skills(&input.target_skills)?;
    for target in &mut skills {
        target.mastery_level = target
            .mastery_level
            .max(current.mastery(target.skill_index));
    }
    let mut modules = parse_target_modules(&input.target_modules)?;
    for target in modules.iter_mut().filter(|t| t.module_stage > 0) {
        target.module_stage = target
            .module_stage
            .max(current.module_stage(&target.module_id));
    }
    input.target_skills = serde_json::to_value(skills)?;
    input.target_modules = serde_json::to_value(modules)?;
    Ok(())
}

#[must_use]
pub fn stored_plan_met(plan: &OperatorPlan, roster_entry: Option<&RosterEntry>) -> bool {
    PlanTargets::parse(plan)
        .is_ok_and(|targets| plan_met(plan, &targets, &CurrentState::from_roster(roster_entry)))
}

fn add_cost(materials: &mut HashMap<String, i32>, item_id: &str, count: i32) {
    *materials.entry(item_id.to_owned()).or_insert(0) += count;
}

fn sum_level_range(values: &[i32], from_level: i16, to_level: i16) -> i32 {
    ((from_level as usize)..(to_level as usize))
        .filter_map(|level| values.get(level - 1))
        .sum()
}

pub fn calculate_leveling_costs(
    operator: &Operator,
    gamedata: &GameModel,
    current_elite: i16,
    current_level: i16,
    target_elite: i16,
    target_level: i16,
    materials: &mut HashMap<String, i32>,
) {
    let consts = &gamedata.consts;
    let phase_cap = |elite: i16| operator.phases[elite as usize].max_level as i16;

    // (elite, from level, to level) for each phase
    let segments: Vec<(i16, i16, i16)> = if target_elite > current_elite {
        let mut segments = vec![(current_elite, current_level, phase_cap(current_elite))];
        segments.extend(((current_elite + 1)..target_elite).map(|e| (e, 1, phase_cap(e))));
        segments.push((target_elite, 1, target_level));
        segments
    } else if target_elite == current_elite && target_level > current_level {
        vec![(target_elite, current_level, target_level)]
    } else {
        Vec::new()
    };

    let mut exp_needed = 0;
    let mut lmd_needed = 0;
    for (elite, from_level, to_level) in segments {
        if let Some(exp_map) = consts.character_exp_map.get(elite as usize) {
            exp_needed += sum_level_range(&exp_map.values, from_level, to_level);
        }
        if let Some(lmd_map) = consts.character_upgrade_cost_map.get(elite as usize) {
            lmd_needed += sum_level_range(&lmd_map.values, from_level, to_level);
        }
    }

    if lmd_needed > 0 {
        add_cost(materials, LMD_ITEM, lmd_needed);
    }
    if exp_needed > 0 {
        add_cost(materials, EXP_ITEM, exp_needed);
    }
}

pub fn get_plan_direct_materials(
    gamedata: &GameModel,
    plan: &OperatorPlan,
    operator: &Operator,
    roster_entry: Option<&RosterEntry>,
) -> Result<HashMap<String, i32>, PlanError> {
    let current = CurrentState::from_roster(roster_entry);
    let targets = PlanTargets::parse(plan)?;

    let mut materials = HashMap::new();

    if !current.reaches_level(plan.target_elite, plan.target_level) {
        calculate_leveling_costs(
            operator,
            gamedata,
            current.elite,
            current.level,
            plan.target_elite,
            plan.target_level,
            &mut materials,
        );

        for elite in (current.elite + 1)..=plan.target_elite {
            if let Some(evolve_costs) = &operator.phases[elite as usize].evolve_cost {
                for cost in evolve_costs {
                    add_cost(&mut materials, &cost.id, cost.count);
                }
            }
        }
    }

    if !current.reaches_skill_level(plan.target_skill_level) {
        // Entry `i` of `all_skill_level_up` raises skill level `i + 1` to `i + 2`.
        for i in (current.skill_level - 1)..(plan.target_skill_level - 1) {
            if let Some(lvl_up) = operator.all_skill_level_up.get(i as usize) {
                for cost in &lvl_up.lvl_up_cost {
                    add_cost(&mut materials, &cost.id, cost.count);
                }
            }
        }
    }

    for target_skill in &targets.skills {
        if current.reaches_mastery(target_skill) {
            continue;
        }
        let Some(skill_entry) = operator.skills.get(target_skill.skill_index as usize) else {
            continue;
        };
        let current_mastery = current.mastery(target_skill.skill_index);
        for i in (current_mastery as usize)..(target_skill.mastery_level as usize) {
            if let Some(cond) = skill_entry.level_up_cost_cond.get(i) {
                for cost in &cond.level_up_cost {
                    add_cost(&mut materials, &cost.id, cost.count);
                }
            }
        }
    }

    for target_module in &targets.modules {
        if current.reaches_module(target_module) {
            continue;
        }
        let Some(item_cost_map) = operator
            .modules
            .iter()
            .find(|m| m.module.uni_equip_id == target_module.module_id)
            .and_then(|m| m.module.item_cost.as_ref())
        else {
            continue;
        };
        let current_stage = current.module_stage(&target_module.module_id);
        for stage in (current_stage + 1)..=target_module.module_stage {
            if let Some(costs) = item_cost_map.get(&stage.to_string()) {
                for cost in costs {
                    add_cost(&mut materials, &cost.id, cost.count);
                }
            }
        }
    }

    Ok(materials)
}
