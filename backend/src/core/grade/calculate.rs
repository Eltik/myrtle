use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::{gamedata::types::GameData, grade::grade_operators::grade_operators},
    database::queries::roster,
};

pub struct UserGrade {
    pub operator_grade: f64,
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

    let total_score = operator_grade;
    let overall = score_to_grade(total_score);

    Ok(UserGrade {
        operator_grade,
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
