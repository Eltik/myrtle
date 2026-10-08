use std::collections::HashSet;

use sqlx::PgPool;
use uuid::Uuid;

use crate::core::{
    gamedata::types::GameData,
    grade::{
        base::score::grade_base, grade_medals::grade_medals, grade_operators::grade_operators,
        grade_roguelike::grade_roguelike, sandbox::grade_sandbox, stages::grade_stages,
    },
};
use crate::database::queries::building::get_building;
use crate::database::queries::medals::get_user_medals;
use crate::database::queries::roguelike::get_roguelike_progress;
use crate::database::queries::roster::get_roster;
use crate::database::queries::roster::get_supports;

/// Relative section weights; a section's share is `weight / SECTION_WEIGHT_TOTAL`.
///
/// - Operators (34%): the core investment, capped near a third so roster depth
///   alone can't carry an account.
/// - Stages (24%): combat progression is what the roster is for, so stages
///   outweigh every supporting system, base included.
/// - Base (14%): every mature account converges on the same maxed base, so it
///   differentiates less than stages.
/// - Roguelike (12%): the deepest side mode, above the breadth sections.
/// - Medals / Sandbox (8% each): completionism, kept light so it stays optional.
pub const SECTION_WEIGHT_OPERATOR: f64 = 0.85;
pub const SECTION_WEIGHT_BASE: f64 = 0.35;
pub const SECTION_WEIGHT_ROGUELIKE: f64 = 0.3;
pub const SECTION_WEIGHT_MEDAL: f64 = 0.2;
pub const SECTION_WEIGHT_STAGE: f64 = 0.6;
pub const SECTION_WEIGHT_SANDBOX: f64 = 0.2;

/// Mirrored in the frontend's `Score/helpers.ts` (`SUBSCORES`); keep in sync.
pub const SECTION_WEIGHT_TOTAL: f64 = SECTION_WEIGHT_OPERATOR
    + SECTION_WEIGHT_BASE
    + SECTION_WEIGHT_ROGUELIKE
    + SECTION_WEIGHT_MEDAL
    + SECTION_WEIGHT_STAGE
    + SECTION_WEIGHT_SANDBOX;

pub struct UserGrade {
    pub operator_grade: f64,
    pub base_grade: f64,
    /// The base grade's stationing-utilization component.
    pub base_utilization: f64,
    /// The base grade's infrastructure-completeness component.
    pub base_infrastructure: f64,
    pub roguelike_grade: f64,
    pub medal_grade: f64,
    pub stage_grade: f64,
    pub sandbox_grade: f64,
    pub overall: String,
    pub total_score: f64,
}

pub async fn calculate_user_grade(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<UserGrade, sqlx::Error> {
    let (
        user_roster,
        supports,
        building_json,
        roguelike_data,
        user_medals,
        stage_grade,
        sandbox_grade,
    ) = tokio::try_join!(
        get_roster(pool, user_id),
        get_supports(pool, user_id),
        get_building(pool, user_id),
        get_roguelike_progress(pool, user_id),
        get_user_medals(pool, user_id),
        grade_stages(pool, user_id, game_data),
        grade_sandbox(pool, user_id, game_data),
    )?;

    let support_ids: HashSet<&str> = supports.iter().map(|s| s.operator_id.as_str()).collect();
    let owned_operators: HashSet<&str> =
        user_roster.iter().map(|e| e.operator_id.as_str()).collect();
    let operator_grade = grade_operators(&user_roster, game_data, &support_ids);
    let base = grade_base(&user_roster, building_json.as_ref(), game_data);
    let base_grade = base.score;
    let roguelike_grade = grade_roguelike(&roguelike_data, &game_data.roguelike);
    let medal_grade = grade_medals(&user_medals, &game_data.medals, &owned_operators);

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

    Ok(UserGrade {
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
    })
}

fn score_to_grade(score: f64) -> String {
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
