use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::{
        gamedata::types::GameData,
        grade::{base::score::grade_base, grade_operators::grade_operators},
    },
    database::queries::{building, roster},
};

pub struct UserGrade {
    pub operator_grade: f64,
    pub base_grade: f64,
    pub overall: String,
    pub total_score: f64,
}

pub async fn calculate_user_grade(
    pool: &PgPool,
    user_id: Uuid,
    game_data: &GameData,
) -> Result<UserGrade, sqlx::Error> {
    let user_roster = roster::get_roster(pool, user_id).await?;
    let operator_grade = grade_operators(&user_roster, game_data);

    let building_json = building::get_building(pool, user_id).await?;
    let base_grade = grade_base(&user_roster, building_json.as_ref(), game_data);

    let scores: Vec<(f64, f64)> = vec![(1.0, operator_grade), (0.5, base_grade)];
    // Future:
    // scores.push((0.4, stage_grade));
    // scores.push((0.3, roguelike_grade));
    // scores.push((0.2, medal_grade));
    // scores.push((0.1, skin_grade));

    let total_weight: f64 = scores.iter().map(|(w, _)| w).sum();
    let total_score = scores.iter().map(|(w, v)| w * v).sum::<f64>() / total_weight;
    let overall = score_to_grade(total_score);

    Ok(UserGrade {
        operator_grade,
        base_grade,
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
