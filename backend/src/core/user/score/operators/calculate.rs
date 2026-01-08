use crate::core::local::types::operator::OperatorRarity;
use crate::core::local::types::trust::Favor;
use crate::core::user::types::CharacterData;

use super::helpers::{
    calculate_level_score, calculate_mastery_score, calculate_module_score,
    calculate_potential_score, calculate_trust_score, get_base_score, get_completion_status,
    rarity_to_int,
};
use super::types::OperatorScore;

/// Calculate the score for a single operator
pub fn calculate_operator_score(
    character: &CharacterData,
    operator_name: &str,
    rarity: &OperatorRarity,
    favor_table: &Favor,
) -> OperatorScore {
    // Base score from rarity
    let base_score = get_base_score(rarity);

    // Level score based on elite and level
    let level_score = calculate_level_score(character.evolve_phase, character.level);

    // Trust score
    let trust_score = calculate_trust_score(character.favor_point, favor_table);

    // Potential score
    let potential_score = calculate_potential_score(character.potential_rank);

    // Mastery score and details
    let (mastery_score, mastery_details) = calculate_mastery_score(&character.skills);

    // Module score and details
    let (module_score, module_details) = calculate_module_score(&character.equip);

    // Determine completion status
    let completion_status =
        get_completion_status(&mastery_details, &module_details, character.evolve_phase);

    // Total score
    let total_score =
        base_score + level_score + trust_score + potential_score + mastery_score + module_score;

    OperatorScore {
        char_id: character.char_id.clone(),
        name: operator_name.to_string(),
        rarity: rarity_to_int(rarity),
        base_score,
        level_score,
        trust_score,
        potential_score,
        mastery_score,
        module_score,
        total_score,
        completion_status,
        mastery_details,
        module_details,
    }
}
