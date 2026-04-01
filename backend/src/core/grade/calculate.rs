use std::collections::{HashMap, HashSet};

use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    core::gamedata::types::{
        GameData,
        module::ModuleType,
        operator::{Operator, OperatorModule, OperatorProfession, OperatorRarity},
    },
    database::{models::roster::RosterEntry, queries::roster},
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

fn grade_operators(roster: &[RosterEntry], game_data: &GameData) -> f64 {
    let roster_map: HashMap<&str, &RosterEntry> =
        roster.iter().map(|r| (r.operator_id.as_str(), r)).collect();
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (op_id, static_op) in &game_data.operators {
        if matches!(
            static_op.profession,
            OperatorProfession::Token | OperatorProfession::Trap
        ) {
            continue;
        }
        if static_op.is_not_obtainable {
            continue;
        }

        let rarity_weight = rarity_to_weight(&static_op.rarity);

        let op_score = match roster_map.get(op_id.as_str()) {
            Some(roster_entry) => grade_operator(roster_entry, static_op),
            None => 0.0, // user doesn't own this operator
        };

        weighted_sum += op_score * rarity_weight;
        weight_total += rarity_weight;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

fn rarity_to_weight(rarity: &OperatorRarity) -> f64 {
    match rarity {
        OperatorRarity::SixStar => 1.0,
        OperatorRarity::FiveStar => 0.7,
        OperatorRarity::FourStar => 0.4,
        OperatorRarity::ThreeStar => 0.15,
        OperatorRarity::TwoStar => 0.1,
        OperatorRarity::OneStar => 0.05,
    }
}

#[derive(Deserialize)]
struct MasteryEntry {
    mastery: i16,
}
#[derive(Deserialize)]
struct ModuleEntry {
    id: String,
    level: i16,
}
fn grade_operator(roster: &RosterEntry, static_op: &Operator) -> f64 {
    // Promotion (25%)
    let max_elite = (static_op.phases.len() - 1) as f64; // 0, 1, or 2
    let elite_score = if max_elite == 0.0 {
        1.0
    } else {
        roster.elite as f64 / max_elite
    };

    // Level (different weight based on rarity)
    let total_progress: f64 = static_op
        .phases
        .iter()
        .enumerate()
        .map(|(i, phase)| {
            if (i as i16) < roster.elite {
                phase.max_level as f64
            } else if i as i16 == roster.elite {
                roster.level as f64
            } else {
                0.0
            }
        })
        .sum();

    let total_max: f64 = static_op.phases.iter().map(|p| p.max_level as f64).sum();
    let level_score = (total_progress / total_max).powf(2.0);

    // Masteries (30%)
    let masteries: Vec<MasteryEntry> =
        serde_json::from_value(roster.masteries.clone()).unwrap_or_default();
    let num_skills = static_op.skills.len();
    let max_mastery = (num_skills as f64) * 3.0;
    let actual_mastery: f64 = masteries.iter().map(|m| m.mastery as f64).sum();
    let mastery_score = if max_mastery > 0.0 {
        actual_mastery / max_mastery // fully M9 operator would score 1.0, no masteries scores 0.0
    } else {
        0.0
    };

    // Modules (20%)
    let user_modules: Vec<ModuleEntry> =
        serde_json::from_value(roster.modules.clone()).unwrap_or_default();
    let available_modules: Vec<&OperatorModule> = static_op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect();
    let max_module_total = available_modules.len() as f64 * 3.0;
    let advanced_ids: HashSet<&str> = available_modules
        .iter()
        .map(|m| m.module.uni_equip_id.as_str())
        .collect();
    let actual_module_total: f64 = user_modules
        .iter()
        .filter(|m| advanced_ids.contains(m.id.as_str()))
        .map(|m| m.level as f64)
        .sum();

    let module_score = if max_module_total > 0.0 {
        actual_module_total / max_module_total
    } else {
        0.0
    };

    // Potential (10%)
    let potential_score = (roster.potential - 1) as f64 / 5.0;

    let mut dimensions: Vec<(f64, f64)> = vec![]; // (weight, score)
    if max_elite > 0.0 {
        dimensions.push((25.0, elite_score));
    }
    dimensions.push((level_weight(&static_op.rarity), level_score));
    let can_master = !static_op.skills.is_empty() && static_op.phases.len() >= 3;
    if can_master {
        dimensions.push((30.0, mastery_score));
    }
    if max_module_total > 0.0 {
        dimensions.push((20.0, module_score));
    }
    if potential_matters(static_op) {
        dimensions.push((10.0, potential_score));
    }

    let total_weight: f64 = dimensions.iter().map(|(w, _)| w).sum();
    let weighted_score: f64 = dimensions.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight;

    weighted_score
}

fn level_weight(rarity: &OperatorRarity) -> f64 {
    // Base weight is 15.0, scaled by relative LMD cost
    // 3★ and below are baseline (1.0x), 4★ ~1.5x, 5★ ~2.5x, 6★ ~4x
    match rarity {
        OperatorRarity::SixStar => 15.0 * 4.0,  // 60.0
        OperatorRarity::FiveStar => 15.0 * 2.5, // 37.5
        OperatorRarity::FourStar => 15.0 * 1.5, // 22.5
        OperatorRarity::ThreeStar => 15.0,
        OperatorRarity::TwoStar => 15.0,
        OperatorRarity::OneStar => 15.0,
    }
}

fn potential_matters(static_op: &Operator) -> bool {
    // For welfares and limiteds
    !static_op.can_use_general_potential_item || static_op.is_sp_char
}
