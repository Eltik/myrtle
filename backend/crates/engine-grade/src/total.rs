use std::collections::HashSet;

use crate::GradeData as GameData;
use crate::inputs::GradeInputs;
use crate::roguelike::grade_roguelike;
use crate::sandbox::grade_sandbox;
use crate::stages::grade_stages;
use crate::{medals::grade_medals, operators::grade_operators_in};

pub const SECTION_WEIGHT_OPERATOR: f64 = 0.85;
pub const SECTION_WEIGHT_BASE: f64 = 0.35;
pub const SECTION_WEIGHT_ROGUELIKE: f64 = 0.3;
pub const SECTION_WEIGHT_MEDAL: f64 = 0.2;
pub const SECTION_WEIGHT_STAGE: f64 = 0.6;
pub const SECTION_WEIGHT_SANDBOX: f64 = 0.2;

pub const SECTION_WEIGHT_TOTAL: f64 = SECTION_WEIGHT_OPERATOR
    + SECTION_WEIGHT_BASE
    + SECTION_WEIGHT_ROGUELIKE
    + SECTION_WEIGHT_MEDAL
    + SECTION_WEIGHT_STAGE
    + SECTION_WEIGHT_SANDBOX;

pub struct UserGrade {
    pub operator_grade: f64,
    pub base_grade: f64,
    pub base_utilization: f64,
    pub base_infrastructure: f64,
    pub roguelike_grade: f64,
    pub medal_grade: f64,
    pub stage_grade: f64,
    pub sandbox_grade: f64,
    pub overall: String,
    pub total_score: f64,
}

pub fn calculate_user_grade(game_data: &GameData, inputs: &GradeInputs) -> UserGrade {
    let user_roster = &inputs.roster;
    let supports = &inputs.supports;
    let roguelike_data = &inputs.roguelike;
    let user_medals = &inputs.medals;
    let stage_grade = grade_stages(
        game_data,
        &inputs.stage_clears,
        &inputs.known_stages,
        inputs.now_unix,
    );
    let sandbox_grade = grade_sandbox(game_data, inputs.sandbox.as_ref());

    let support_ids: HashSet<&str> = supports.iter().map(String::as_str).collect();
    let owned_operators: HashSet<&str> =
        user_roster.iter().map(|e| e.operator_id.as_str()).collect();
    let operator_grade =
        grade_operators_in(user_roster, game_data, &support_ids, inputs.score_model);
    let base = inputs.base;
    let base_grade = base.score;
    let roguelike_grade = grade_roguelike(roguelike_data, &game_data.roguelike);
    let medal_grade = grade_medals(
        user_medals,
        &game_data.medals,
        &owned_operators,
        inputs.now_unix,
    );

    let scores: Vec<(f64, f64)> = vec![
        (SECTION_WEIGHT_OPERATOR, operator_grade),
        (SECTION_WEIGHT_BASE, base_grade),
        (SECTION_WEIGHT_ROGUELIKE, roguelike_grade),
        (SECTION_WEIGHT_MEDAL, medal_grade),
        (SECTION_WEIGHT_STAGE, stage_grade),
        (SECTION_WEIGHT_SANDBOX, sandbox_grade),
    ];

    let total_score = scores.iter().map(|(w, v)| w * v).sum::<f64>() / SECTION_WEIGHT_TOTAL;
    let overall = score_to_grade(total_score);

    UserGrade {
        operator_grade,
        base_grade,
        base_utilization: base.utilization,
        base_infrastructure: base.infrastructure,
        roguelike_grade,
        medal_grade,
        stage_grade,
        sandbox_grade,
        overall,
        total_score,
    }
}

#[must_use]
pub fn score_to_grade(score: f64) -> String {
    match score {
        s if s >= 0.90 => "S+",
        s if s >= 0.75 => "S",
        s if s >= 0.60 => "A",
        s if s >= 0.45 => "B",
        s if s >= 0.30 => "C",
        s if s >= 0.15 => "D",
        _ => "F",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn grade_ladder_boundaries_are_inclusive() {
        let cases = [
            (1.0, "S+"),
            (0.90, "S+"),
            (0.899_999, "S"),
            (0.75, "S"),
            (0.749_999, "A"),
            (0.60, "A"),
            (0.45, "B"),
            (0.30, "C"),
            (0.15, "D"),
            (0.149_999, "F"),
            (0.0, "F"),
        ];
        for (score, grade) in cases {
            assert_eq!(score_to_grade(score), grade, "score {score}");
        }
    }

    #[test]
    fn out_of_range_scores_land_on_the_ends() {
        assert_eq!(score_to_grade(1.5), "S+");
        assert_eq!(score_to_grade(-0.2), "F");
        // NaN fails every `>=` guard and falls through to the bottom rung.
        assert_eq!(score_to_grade(f64::NAN), "F");
    }

    #[test]
    fn section_weights_give_the_documented_shares() {
        assert!(close(SECTION_WEIGHT_TOTAL, 2.5));
        let share = |w: f64| w / SECTION_WEIGHT_TOTAL;
        assert!(close(share(SECTION_WEIGHT_OPERATOR), 0.34));
        assert!(close(share(SECTION_WEIGHT_STAGE), 0.24));
        assert!(close(share(SECTION_WEIGHT_BASE), 0.14));
        assert!(close(share(SECTION_WEIGHT_ROGUELIKE), 0.12));
        assert!(close(share(SECTION_WEIGHT_MEDAL), 0.08));
        assert!(close(share(SECTION_WEIGHT_SANDBOX), 0.08));
    }
}
