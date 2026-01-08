use std::collections::HashMap;

use crate::core::local::types::operator::{Operator, OperatorProfession, OperatorRarity};
use crate::core::local::types::skin::SkinData;
use crate::core::local::types::trust::Favor;
use crate::core::user::types::{CharacterSkill, EquipData};

use super::types::{CompletionStatus, MasteryDetails, ModuleDetails, SkinDetails};

/// Base points by rarity (reflects material cost investment)
pub fn get_base_score(rarity: &OperatorRarity) -> f32 {
    match rarity {
        OperatorRarity::SixStar => 500.0,
        OperatorRarity::FiveStar => 400.0,
        OperatorRarity::FourStar => 150.0,
        OperatorRarity::ThreeStar => 30.0,
        OperatorRarity::TwoStar => 10.0,
        OperatorRarity::OneStar => 5.0,
    }
}

/// Convert OperatorRarity to numeric value (1-6)
pub fn rarity_to_int(rarity: &OperatorRarity) -> i32 {
    match rarity {
        OperatorRarity::OneStar => 1,
        OperatorRarity::TwoStar => 2,
        OperatorRarity::ThreeStar => 3,
        OperatorRarity::FourStar => 4,
        OperatorRarity::FiveStar => 5,
        OperatorRarity::SixStar => 6,
    }
}

/// Calculate level score based on elite phase and current level
/// E0: level * 0.5
/// E1: 50 + (level * 1.0)
/// E2: 150 + (level * 2.0)
pub fn calculate_level_score(evolve_phase: i32, level: i32) -> f32 {
    match evolve_phase {
        0 => level as f32 * 0.5,
        1 => 50.0 + (level as f32 * 1.0),
        2 => 150.0 + (level as f32 * 2.0),
        _ => 0.0,
    }
}

pub fn calculate_trust_score(favor_point: i32, favor_table: &Favor) -> f32 {
    let trust_percent = get_trust_percent(favor_table, favor_point);

    // 0-100%: Main rewards (0-30 points) - stats matter here
    // 100-200%: Minor bonus (30-50 points) - dedication reward
    if trust_percent <= 100.0 {
        (trust_percent / 100.0 * 30.0) as f32
    } else {
        30.0 + ((trust_percent - 100.0) / 100.0 * 20.0) as f32
    }
}

/// Get trust percentage from favor points
/// Finds the highest threshold in favor_frames that the user has reached
fn get_trust_percent(favor_table: &Favor, favor_point: i32) -> f64 {
    favor_table
        .favor_frames
        .iter()
        .rev()
        .find(|frame| favor_point >= frame.data.favor_point)
        .map(|frame| frame.data.percent)
        .unwrap_or(0.0)
}

/// Calculate potential score
/// Potential 1: 0, Potential 2: 10, Potential 3: 25, Potential 4: 45, Potential 5: 70, Potential 6: 100
pub fn calculate_potential_score(potential_rank: i32) -> f32 {
    match potential_rank {
        1 => 0.0, // Default potential
        2 => 10.0,
        3 => 25.0,
        4 => 45.0,
        5 => 70.0,
        6 => 100.0, // Max potential
        _ => 0.0,
    }
}

/// Calculate mastery score and details from skills
/// Returns (total_score, MasteryDetails)
/// Per skill: M0=0, M1=30, M2=70, M3=150
/// Priority: M3 is the main power milestone, additional M3s (M6/M9) are less critical
/// Note: Completion bonuses moved to future vanity score system
pub fn calculate_mastery_score(skills: &[CharacterSkill]) -> (f32, MasteryDetails) {
    let mut total_score = 0.0;
    let mut m3_count = 0;
    let mut total_mastery_levels = 0;

    for skill in skills {
        let mastery = skill.specialize_level;
        total_mastery_levels += mastery;

        let skill_score = match mastery {
            0 => 0.0,
            1 => 30.0,
            2 => 70.0,
            3 => {
                m3_count += 1;
                150.0
            }
            _ => 0.0,
        };
        total_score += skill_score;
    }

    // No completion bonuses - those will be part of vanity score with skins
    let details = MasteryDetails {
        m3_count,
        total_mastery_levels,
    };

    (total_score, details)
}

/// Calculate module score and details from equipment
/// Level 1: 50, Level 2: 100, Level 3: 150 (main power milestone)
/// Level 4: 160, Level 5: 170, Level 6: 180 (small increments - less critical than M9)
/// Priority: Mod3 is the main milestone, Mod4/5/6 are minor power gains
/// Note: Completion bonuses moved to future vanity score system
pub fn calculate_module_score(equip: &HashMap<String, EquipData>) -> (f32, ModuleDetails) {
    let mut total_score = 0.0;
    let mut modules_unlocked = 0;
    let mut modules_at_max = 0;
    let mut highest_level = 0;

    for equip_data in equip.values() {
        modules_unlocked += 1;
        let level = equip_data.level;

        if level > highest_level {
            highest_level = level;
        }

        // Mod3 is the main milestone (150), Mod4/5/6 add small increments (+10 each)
        let module_score = match level {
            0 => 0.0,
            1 => 50.0,
            2 => 100.0,
            3 => {
                modules_at_max += 1;
                150.0
            }
            4 => {
                modules_at_max += 1;
                160.0 // +10 from Mod3
            }
            5 => {
                modules_at_max += 1;
                170.0 // +10 from Mod4
            }
            6 => {
                modules_at_max += 1;
                180.0 // +10 from Mod5
            }
            _ => 0.0,
        };
        total_score += module_score;
    }

    // No completion bonuses - those will be part of vanity score with skins
    let details = ModuleDetails {
        modules_unlocked,
        modules_at_max,
        highest_level,
    };

    (total_score, details)
}

/// Determine completion status based on mastery and module investment
pub fn get_completion_status(
    mastery_details: &MasteryDetails,
    module_details: &ModuleDetails,
    evolve_phase: i32,
) -> CompletionStatus {
    let m3_count = mastery_details.m3_count;
    let total_mastery = mastery_details.total_mastery_levels;
    let modules_at_max = module_details.modules_at_max;

    // M9 = absolutely completed
    if m3_count >= 3 {
        return CompletionStatus::AbsolutelyCompleted;
    }

    // M6 or multiple max modules = highly invested
    if m3_count >= 2 || modules_at_max >= 2 {
        return CompletionStatus::HighlyInvested;
    }

    // At least one M3 = partially completed
    if m3_count >= 1 {
        return CompletionStatus::PartiallyCompleted;
    }

    // Any mastery or modules or E2 = in progress
    if total_mastery > 0 || module_details.modules_unlocked > 0 || evolve_phase > 0 {
        return CompletionStatus::InProgress;
    }

    CompletionStatus::NotStarted
}

/// Check if an operator is a token or trap (should be excluded from scoring)
pub fn is_token_or_trap(char_id: &str, operator: &Operator) -> bool {
    // Check by profession
    if matches!(
        operator.profession,
        OperatorProfession::Token | OperatorProfession::Trap
    ) {
        return true;
    }

    // Check by char_id prefix
    if char_id.starts_with("token_") || char_id.starts_with("trap_") {
        return true;
    }

    false
}

/// Get detailed skin information for an operator
/// Analyzes owned skins by tier: L2D (animated), store-bought, and event rewards
pub fn get_operator_skin_details(
    char_id: &str,
    user_character_skins: &HashMap<String, i32>,
    skin_data: &SkinData,
) -> SkinDetails {
    let mut owned_count = 0;
    let mut total_available = 0;
    let mut owned_l2d = 0;
    let mut owned_store = 0;
    let mut owned_event = 0;
    let mut total_l2d = 0;
    let mut total_store = 0;
    let mut total_event = 0;

    for (skin_id, skin) in &skin_data.char_skins {
        // Only count skins for this operator
        if skin.char_id != char_id {
            continue;
        }

        // Skip default skins (E0/E1/E2)
        if !skin.is_buy_skin {
            continue;
        }

        // Determine skin type
        let is_l2d = skin.dyn_illust_id.is_some() || skin.dyn_portrait_id.is_some();
        let obtain_approach = skin.display_skin.obtain_approach.as_deref().unwrap_or("");
        let is_store = obtain_approach == "Store";
        let is_event = matches!(obtain_approach, "Event Reward" | "Event Gift");

        // Count totals by type
        total_available += 1;
        if is_l2d {
            total_l2d += 1;
        }
        if is_store {
            total_store += 1;
        } else if is_event {
            total_event += 1;
        }

        // Check if user owns this skin
        if user_character_skins.contains_key(skin_id) {
            owned_count += 1;
            if is_l2d {
                owned_l2d += 1;
            }
            if is_store {
                owned_store += 1;
            } else if is_event {
                owned_event += 1;
            }
        }
    }

    // Calculate completion percentage
    let completion_percentage = if total_available > 0 {
        (owned_count as f32 / total_available as f32) * 100.0
    } else {
        0.0
    };

    SkinDetails {
        owned_count,
        total_available,
        owned_l2d,
        owned_store,
        owned_event,
        total_l2d,
        total_store,
        total_event,
        completion_percentage,
    }
}

/// Calculate skin score based on collection completion
/// Scoring (modest to not overshadow gameplay investment):
/// - Store skins: 15 points each (18 OP investment)
/// - L2D skins: 21 points each (24 OP investment, +6 bonus over standard)
/// - Event skins: 8 points each (shows dedication/participation)
/// - Completion bonuses:
///   - 100% collection: +30 points
///   - 75%+ collection: +15 points
///   - 50%+ collection: +8 points
pub fn calculate_skin_score(details: &SkinDetails) -> f32 {
    // Base scoring by skin type (kept modest relative to gameplay investment)
    let store_score = details.owned_store as f32 * 15.0;
    let l2d_score = details.owned_l2d as f32 * 21.0;
    let event_score = details.owned_event as f32 * 8.0;

    let base_score = store_score + l2d_score + event_score;

    // Collection completion bonus (only if operator has available skins)
    let completion_bonus = if details.total_available > 0 {
        if details.completion_percentage >= 100.0 {
            30.0 // Full collection bonus
        } else if details.completion_percentage >= 75.0 {
            15.0
        } else if details.completion_percentage >= 50.0 {
            8.0
        } else {
            0.0
        }
    } else {
        0.0
    };

    base_score + completion_bonus
}
