//! Where an operator stands against its plan, and what the gap costs.

use serde::Deserialize;
use std::collections::HashMap;

use super::{EXP_ITEM, LMD_ITEM};
use crate::{
    app::error::ApiError,
    core::gamedata::types::{GameData, operator::Operator},
    database::models::{
        planner::{OperatorPlan, TargetModulePlan, TargetSkillPlan},
        roster::RosterEntry,
    },
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
    /// The save keeps a module the player has not unlocked at `level` 1 with
    /// this flag set (every locked row in the database carries level 1 or
    /// more), so `level` alone reads a locked module as stage 1.
    #[serde(default)]
    locked: bool,
}

/// Whether Elite `elite` level `level` is at or past Elite `required_elite`
/// level `required_level`. A higher promotion passes at any level.
pub(super) const fn reaches_promotion(
    elite: i16,
    level: i16,
    required_elite: i16,
    required_level: i16,
) -> bool {
    elite > required_elite || (elite == required_elite && level >= required_level)
}

/// The roster state a plan is measured from. The material diff
/// (`get_plan_direct_materials`) and the completion check (`plan_met`) both
/// read it, and every cost the diff adds sits behind the negation of one of
/// the `reaches_*` checks `plan_met` requires. A met plan therefore costs
/// nothing by construction; the two cannot disagree.
struct CurrentState {
    owned: bool,
    elite: i16,
    level: i16,
    skill_level: i16,
    masteries: Vec<RosterMastery>,
    modules: Vec<RosterModule>,
}

impl CurrentState {
    /// An operator the player does not own reads as Elite 0 level 1, skill
    /// level 1, nothing mastered or unlocked.
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

    /// A locked module is stage 0, whatever level the save carries for it.
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

/// A plan's mastery and module targets, parsed from their jsonb columns.
struct PlanTargets {
    skills: Vec<TargetSkillPlan>,
    modules: Vec<TargetModulePlan>,
}

impl PlanTargets {
    fn parse(plan: &OperatorPlan) -> Result<Self, ApiError> {
        Ok(Self {
            skills: parse_target_skills(&plan.target_skills)?,
            modules: parse_target_modules(&plan.target_modules)?,
        })
    }
}

pub(super) fn parse_target_skills(
    target_skills: &serde_json::Value,
) -> Result<Vec<TargetSkillPlan>, ApiError> {
    serde_json::from_value(target_skills.clone())
        .map_err(|_| ApiError::BadRequest("Invalid target_skills format".into()))
}

pub(super) fn parse_target_modules(
    target_modules: &serde_json::Value,
) -> Result<Vec<TargetModulePlan>, ApiError> {
    serde_json::from_value(target_modules.clone())
        .map_err(|_| ApiError::BadRequest("Invalid target_modules format".into()))
}

/// Whether the plan has nothing left to do: the operator is owned and every
/// target is reached or passed. A target mastery or module stage of 0 is
/// always reached.
fn plan_met(plan: &OperatorPlan, targets: &PlanTargets, current: &CurrentState) -> bool {
    current.owned
        && current.reaches_level(plan.target_elite, plan.target_level)
        && current.reaches_skill_level(plan.target_skill_level)
        && targets.skills.iter().all(|t| current.reaches_mastery(t))
        && targets.modules.iter().all(|t| current.reaches_module(t))
}

/// `plan_met` for a stored plan, reading its own roster entry. A plan whose
/// target columns do not parse is not met.
pub(super) fn stored_plan_met(plan: &OperatorPlan, roster_entry: Option<&RosterEntry>) -> bool {
    PlanTargets::parse(plan)
        .is_ok_and(|targets| plan_met(plan, &targets, &CurrentState::from_roster(roster_entry)))
}

fn add_cost(materials: &mut HashMap<String, i32>, item_id: &str, count: i32) {
    *materials.entry(item_id.to_owned()).or_insert(0) += count;
}

/// The sum of `values[level - 1]` over `from_level..to_level`, each entry
/// being the cost of one level-up. Levels past the table add nothing.
fn sum_level_range(values: &[i32], from_level: i16, to_level: i16) -> i32 {
    ((from_level as usize)..(to_level as usize))
        .filter_map(|level| values.get(level - 1))
        .sum()
}

/// Adds the EXP and LMD that level an operator from Elite `current_elite`
/// level `current_level` to Elite `target_elite` level `target_level` into
/// `materials`. Each promotion crossed levels to that phase's cap first and
/// restarts at level 1. Promotion materials are not included.
pub(crate) fn calculate_leveling_costs(
    operator: &Operator,
    gamedata: &GameData,
    current_elite: i16,
    current_level: i16,
    target_elite: i16,
    target_level: i16,
    materials: &mut HashMap<String, i32>,
) {
    let consts = &gamedata.consts;
    let phase_cap = |elite: i16| operator.phases[elite as usize].max_level as i16;

    // (elite, from level, to level) for each phase the levelling passes through.
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

/// Materials from the current state to the plan's targets. Every cost is
/// added behind the negation of a `CurrentState::reaches_*` check, the same
/// checks `plan_met` requires, so a met plan yields an empty map.
pub(super) fn get_plan_direct_materials(
    gamedata: &GameData,
    plan: &OperatorPlan,
    operator: &Operator,
    roster_entry: Option<&RosterEntry>,
) -> Result<HashMap<String, i32>, ApiError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn roster_entry(
        elite: i16,
        level: i16,
        skill_level: i16,
        masteries: serde_json::Value,
        modules: serde_json::Value,
    ) -> RosterEntry {
        RosterEntry {
            user_id: Uuid::nil(),
            operator_id: "char_test".to_owned(),
            elite,
            level,
            exp: 0,
            potential: 0,
            skill_level,
            favor_point: 0,
            skin_id: None,
            default_skill: None,
            voice_lan: None,
            current_equip: None,
            current_tmpl: None,
            obtained_at: None,
            masteries,
            modules,
        }
    }

    fn plan(
        elite: i16,
        level: i16,
        skill_level: i16,
        skills: serde_json::Value,
        modules: serde_json::Value,
    ) -> OperatorPlan {
        let now = sqlx::types::chrono::Utc::now();
        OperatorPlan {
            id: Uuid::nil(),
            user_id: Uuid::nil(),
            operator_id: "char_test".to_owned(),
            target_elite: elite,
            target_level: level,
            target_skill_level: skill_level,
            target_skills: skills,
            target_modules: modules,
            display_on_profile: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn plan_met_cases() {
        use serde_json::json;
        let e2_s2m3 = || {
            plan(
                2,
                60,
                7,
                json!([{"skill_index": 1, "mastery_level": 3}]),
                json!([{"module_id": "mod_x", "module_stage": 2}]),
            )
        };
        let roster =
            |elite, level, masteries, modules| roster_entry(elite, level, 7, masteries, modules);
        let mod_x =
            |level: i16, locked: bool| json!([{"id": "mod_x", "level": level, "locked": locked}]);
        let s2m3 = json!([{"index": 1, "mastery": 3}]);

        let cases: Vec<(&str, OperatorPlan, Option<RosterEntry>, bool)> = vec![
            ("unowned", e2_s2m3(), None, false),
            (
                "unowned, E0 L1 plan",
                plan(0, 1, 1, json!([]), json!([])),
                None,
                false,
            ),
            (
                "exact",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), mod_x(2, false))),
                true,
            ),
            (
                "exceeded: higher level, stage, mastery elsewhere",
                e2_s2m3(),
                Some(roster(
                    2,
                    90,
                    json!([{"index": 0, "mastery": 3}, {"index": 1, "mastery": 3}]),
                    mod_x(3, false),
                )),
                true,
            ),
            (
                "higher elite at a lower level",
                plan(1, 70, 7, json!([]), json!([])),
                Some(roster(2, 1, json!([]), json!([]))),
                true,
            ),
            (
                "level short",
                e2_s2m3(),
                Some(roster(2, 59, s2m3.clone(), mod_x(2, false))),
                false,
            ),
            (
                "skill level short",
                e2_s2m3(),
                Some(roster_entry(2, 60, 6, s2m3.clone(), mod_x(2, false))),
                false,
            ),
            (
                "mastery on the wrong skill index",
                e2_s2m3(),
                Some(roster(
                    2,
                    60,
                    json!([{"index": 0, "mastery": 3}]),
                    mod_x(2, false),
                )),
                false,
            ),
            (
                "mastery short",
                e2_s2m3(),
                Some(roster(
                    2,
                    60,
                    json!([{"index": 1, "mastery": 2}]),
                    mod_x(2, false),
                )),
                false,
            ),
            (
                "module locked at a high level",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), mod_x(3, true))),
                false,
            ),
            (
                "module missing",
                e2_s2m3(),
                Some(roster(2, 60, s2m3.clone(), json!([]))),
                false,
            ),
            (
                "zero targets are always reached",
                plan(
                    2,
                    60,
                    7,
                    json!([{"skill_index": 2, "mastery_level": 0}]),
                    json!([{"module_id": "mod_x", "module_stage": 0}]),
                ),
                Some(roster(2, 60, json!([]), mod_x(1, true))),
                true,
            ),
        ];

        for (name, plan, entry, expected) in cases {
            assert_eq!(stored_plan_met(&plan, entry.as_ref()), expected, "{name}");
        }
    }

    fn operator_with_module(module_id: &str, stage_costs: &[(&str, &str, i32)]) -> Operator {
        use crate::core::gamedata::types::{
            module::{Module, ModuleItemCost},
            operator::OperatorModule,
        };
        let mut item_cost: HashMap<String, Vec<ModuleItemCost>> = HashMap::new();
        for (stage, id, count) in stage_costs {
            item_cost
                .entry((*stage).to_owned())
                .or_default()
                .push(ModuleItemCost {
                    id: (*id).to_owned(),
                    count: *count,
                    ..Default::default()
                });
        }
        Operator {
            modules: vec![OperatorModule {
                module: Module {
                    uni_equip_id: module_id.to_owned(),
                    item_cost: Some(item_cost),
                    ..Default::default()
                },
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    /// A locked module costs its stage 1 even though the save stores it at
    /// level 1; unlocked at level 1, stage 1 costs nothing.
    #[test]
    fn locked_module_is_stage_zero() {
        use serde_json::json;
        let gamedata = GameData::default();
        let operator = operator_with_module("mod_x", &[("1", "unlock", 5), ("2", "upgrade", 3)]);
        let target = plan(
            2,
            1,
            1,
            json!([]),
            json!([{"module_id": "mod_x", "module_stage": 1}]),
        );
        let module = |locked: bool| json!([{"id": "mod_x", "level": 1, "locked": locked}]);

        let locked = roster_entry(2, 1, 1, json!([]), module(true));
        let costs =
            get_plan_direct_materials(&gamedata, &target, &operator, Some(&locked)).unwrap();
        assert_eq!(costs, HashMap::from([("unlock".to_owned(), 5)]));
        assert!(!stored_plan_met(&target, Some(&locked)));

        let unlocked = roster_entry(2, 1, 1, json!([]), module(false));
        let costs =
            get_plan_direct_materials(&gamedata, &target, &operator, Some(&unlocked)).unwrap();
        assert!(costs.is_empty());
        assert!(stored_plan_met(&target, Some(&unlocked)));
    }
}
