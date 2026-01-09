//! Medal score calculation
//!
//! Calculates user scores based on medal completions with points
//! varying by rarity, category, and group completion bonuses.

use std::collections::HashSet;

use crate::core::local::types::medal::MedalData;
use crate::core::user::types::User;

use super::types::{MedalBreakdown, MedalCategoryScore, MedalGroupScore, MedalScore};

/// Point values for medal scoring
mod points {
    // === Rarity-based points ===
    /// Common medals - 660 available
    pub const T1_MEDAL: f32 = 5.0;
    /// Uncommon medals - 493 available
    pub const T2_MEDAL: f32 = 15.0;
    /// Rare medals - 89 available
    pub const T3_MEDAL: f32 = 50.0;
    /// Special difficulty medals - 62 available
    pub const T2D5_MEDAL: f32 = 75.0;

    // === Category multipliers ===
    pub const CATEGORY_PLAYER: f32 = 1.0;
    pub const CATEGORY_STAGE: f32 = 1.1;
    pub const CATEGORY_CAMP: f32 = 1.2;
    pub const CATEGORY_TOWER: f32 = 1.3;
    pub const CATEGORY_GROWTH: f32 = 1.0;
    pub const CATEGORY_STORY: f32 = 1.0;
    pub const CATEGORY_BUILD: f32 = 1.0;
    pub const CATEGORY_ACTIVITY: f32 = 1.0;
    pub const CATEGORY_ROGUE: f32 = 1.2;
    pub const CATEGORY_HIDDEN: f32 = 1.5;

    // === Group completion bonuses ===
    /// Small groups (<= 5 medals)
    pub const GROUP_COMPLETE_SMALL: f32 = 25.0;
    /// Medium groups (6-10 medals)
    pub const GROUP_COMPLETE_MEDIUM: f32 = 50.0;
    /// Large groups (> 10 medals)
    pub const GROUP_COMPLETE_LARGE: f32 = 100.0;
}

/// Calculate medal score for a user
pub fn calculate_medal_score(user: &User, medal_data: &MedalData) -> MedalScore {
    let mut breakdown = MedalBreakdown::default();
    let mut category_scores = Vec::new();
    let mut group_scores = Vec::new();
    let mut rarity_score = 0.0;
    let mut category_bonus_score = 0.0;
    let mut group_bonus_score = 0.0;

    // Get user's earned medals (fts >= 0 means completed)
    let earned_medal_ids: HashSet<String> = user
        .medal
        .medals
        .iter()
        .filter(|(_, entry)| entry.fts >= 0)
        .map(|(id, _)| id.clone())
        .collect();

    // Update total available counts
    breakdown.total_medals_available = medal_data.medals.len() as i32;

    // Count available medals by rarity
    for medal in medal_data.medals.values() {
        match medal.rarity.as_str() {
            "T1" => breakdown.t1_available += 1,
            "T2" => breakdown.t2_available += 1,
            "T3" => breakdown.t3_available += 1,
            "T2D5" => breakdown.t2d5_available += 1,
            _ => {}
        }
    }

    // Calculate per-category scores
    for (category, medal_ids) in &medal_data.medals_by_type {
        let category_multiplier = get_category_multiplier(category);
        let category_name = medal_data.get_category_name(category);

        let mut cat_score = MedalCategoryScore {
            category: category.clone(),
            category_name,
            medals_available: medal_ids.len() as i32,
            ..Default::default()
        };

        for medal_id in medal_ids {
            if earned_medal_ids.contains(medal_id)
                && let Some(medal) = medal_data.medals.get(medal_id)
            {
                cat_score.medals_earned += 1;

                let base_points = get_rarity_points(&medal.rarity);
                let multiplied_points = base_points * category_multiplier;
                let bonus = multiplied_points - base_points;

                rarity_score += base_points;
                category_bonus_score += bonus;
                cat_score.total_score += multiplied_points;

                // Update rarity counts
                match medal.rarity.as_str() {
                    "T1" => {
                        cat_score.t1_earned += 1;
                        breakdown.t1_earned += 1;
                    }
                    "T2" => {
                        cat_score.t2_earned += 1;
                        breakdown.t2_earned += 1;
                    }
                    "T3" => {
                        cat_score.t3_earned += 1;
                        breakdown.t3_earned += 1;
                    }
                    "T2D5" => {
                        cat_score.t2d5_earned += 1;
                        breakdown.t2d5_earned += 1;
                    }
                    _ => {}
                }
            }
        }

        // Calculate category completion percentage
        if cat_score.medals_available > 0 {
            cat_score.completion_percentage =
                (cat_score.medals_earned as f32 / cat_score.medals_available as f32) * 100.0;
        }

        // Update category-specific counts in breakdown
        update_category_count(&mut breakdown, category, cat_score.medals_earned);

        category_scores.push(cat_score);
    }

    // Calculate group completion bonuses
    for (group_id, group) in &medal_data.groups {
        let medals_total = group.medal_id.len() as i32;
        let medals_earned = group
            .medal_id
            .iter()
            .filter(|id| earned_medal_ids.contains(*id))
            .count() as i32;

        let is_complete = medals_earned == medals_total && medals_total > 0;
        let group_bonus = if is_complete {
            get_group_bonus(medals_total)
        } else {
            0.0
        };

        if is_complete {
            breakdown.groups_complete += 1;
            group_bonus_score += group_bonus;
        }
        breakdown.groups_total += 1;

        group_scores.push(MedalGroupScore {
            group_id: group_id.clone(),
            group_name: group.group_name.clone(),
            medals_earned,
            medals_total,
            completion_percentage: if medals_total > 0 {
                (medals_earned as f32 / medals_total as f32) * 100.0
            } else {
                0.0
            },
            group_bonus,
            is_complete,
        });
    }

    // Finalize breakdown
    breakdown.total_medals_earned = earned_medal_ids.len() as i32;
    if breakdown.total_medals_available > 0 {
        breakdown.total_completion_percentage = (breakdown.total_medals_earned as f32
            / breakdown.total_medals_available as f32)
            * 100.0;
    }

    // Sort category scores by total score descending
    category_scores.sort_by(|a, b| {
        b.total_score
            .partial_cmp(&a.total_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Sort group scores by completion percentage descending
    group_scores.sort_by(|a, b| {
        b.completion_percentage
            .partial_cmp(&a.completion_percentage)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_score = rarity_score + category_bonus_score + group_bonus_score;

    MedalScore {
        total_score,
        rarity_score,
        category_bonus_score,
        group_bonus_score,
        category_scores,
        group_scores,
        breakdown,
    }
}

/// Get base points for a medal rarity
fn get_rarity_points(rarity: &str) -> f32 {
    match rarity {
        "T1" => points::T1_MEDAL,
        "T2" => points::T2_MEDAL,
        "T3" => points::T3_MEDAL,
        "T2D5" => points::T2D5_MEDAL,
        _ => points::T1_MEDAL,
    }
}

/// Get category multiplier
fn get_category_multiplier(category: &str) -> f32 {
    match category {
        "playerMedal" => points::CATEGORY_PLAYER,
        "stageMedal" => points::CATEGORY_STAGE,
        "campMedal" => points::CATEGORY_CAMP,
        "towerMedal" => points::CATEGORY_TOWER,
        "growthMedal" => points::CATEGORY_GROWTH,
        "storyMedal" => points::CATEGORY_STORY,
        "buildMedal" => points::CATEGORY_BUILD,
        "activityMedal" => points::CATEGORY_ACTIVITY,
        "rogueMedal" => points::CATEGORY_ROGUE,
        "hiddenMedal" => points::CATEGORY_HIDDEN,
        _ => 1.0,
    }
}

/// Get group completion bonus based on group size
fn get_group_bonus(medal_count: i32) -> f32 {
    if medal_count <= 5 {
        points::GROUP_COMPLETE_SMALL
    } else if medal_count <= 10 {
        points::GROUP_COMPLETE_MEDIUM
    } else {
        points::GROUP_COMPLETE_LARGE
    }
}

/// Update category-specific medal counts in breakdown
fn update_category_count(breakdown: &mut MedalBreakdown, category: &str, count: i32) {
    match category {
        "playerMedal" => breakdown.player_medals = count,
        "stageMedal" => breakdown.stage_medals = count,
        "campMedal" => breakdown.camp_medals = count,
        "towerMedal" => breakdown.tower_medals = count,
        "growthMedal" => breakdown.growth_medals = count,
        "storyMedal" => breakdown.story_medals = count,
        "buildMedal" => breakdown.build_medals = count,
        "activityMedal" => breakdown.activity_medals = count,
        "rogueMedal" => breakdown.rogue_medals = count,
        "hiddenMedal" => breakdown.hidden_medals = count,
        _ => {}
    }
}
