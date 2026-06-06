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

pub struct UserGrade {
    pub operator_grade: f64,
    pub base_grade: f64,
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
    let base_grade = grade_base(&user_roster, building_json.as_ref(), game_data);
    let roguelike_grade = grade_roguelike(&roguelike_data, &game_data.roguelike);
    let medal_grade = grade_medals(&user_medals, &game_data.medals, &owned_operators);

    let scores: Vec<(f64, f64)> = vec![
        (1.0, operator_grade),
        (0.5, base_grade),
        (0.3, roguelike_grade),
        (0.2, medal_grade),
        (0.4, stage_grade),
        (0.2, sandbox_grade),
    ];

    let total_weight: f64 = scores.iter().map(|(w, _)| w).sum();
    let total_score = scores.iter().map(|(w, v)| w * v).sum::<f64>() / total_weight;
    let overall = score_to_grade(total_score);

    Ok(UserGrade {
        operator_grade,
        base_grade,
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
