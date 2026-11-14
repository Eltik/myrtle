#![allow(clippy::unwrap_used, clippy::expect_used)]
use myrtle_engine_planner::{PlanError, types::PlanInput, validation::*};
use myrtle_gamedata_raw::tables::operator::{
    AllSkillLevelUp, EnrichedSkill, LevelUpCostCond, Operator, OperatorPhase, Phase,
    UnlockCondition,
};
use serde_json::json;

const fn unlock(phase: OperatorPhase, level: i32) -> UnlockCondition {
    UnlockCondition { phase, level }
}

fn six_star() -> Operator {
    let skill_level_up = |phase| AllSkillLevelUp {
        unlock_cond: unlock(phase, 1),
        ..Default::default()
    };
    Operator {
        name: "Test".to_owned(),
        phases: [50, 80, 90]
            .into_iter()
            .map(|max_level| Phase {
                max_level,
                ..Default::default()
            })
            .collect(),
        all_skill_level_up: [
            OperatorPhase::Elite0,
            OperatorPhase::Elite0,
            OperatorPhase::Elite0,
            OperatorPhase::Elite1,
            OperatorPhase::Elite1,
            OperatorPhase::Elite1,
        ]
        .into_iter()
        .map(skill_level_up)
        .collect(),
        skills: vec![EnrichedSkill {
            level_up_cost_cond: (0..3)
                .map(|_| LevelUpCostCond {
                    unlock_cond: unlock(OperatorPhase::Elite2, 1),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn plan_input(skill_level: i16, mastery: i16) -> PlanInput {
    PlanInput {
        target_elite: 2,
        target_level: 1,
        target_skill_level: skill_level,
        target_skills: json!([{"skill_index": 0, "mastery_level": mastery}]),
        target_modules: json!([]),
        display_on_profile: false,
    }
}

#[test]
fn mastery_requires_skill_level_seven() {
    let operator = six_star();
    assert!(validate_plan(&operator, &plan_input(7, 3)).is_ok());
    assert!(validate_plan(&operator, &plan_input(6, 0)).is_ok());
    match validate_plan(&operator, &plan_input(6, 1)) {
        Err(PlanError::Invalid(msg)) => {
            assert!(msg.contains("requires target skill level 7"), "{msg}");
        }
        other => panic!("expected a 400, got {other:?}"),
    }
}
