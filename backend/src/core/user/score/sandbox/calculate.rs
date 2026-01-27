//! Sandbox score calculation

use crate::core::user::types::User;

use super::types::{SandboxAreaScore, SandboxBreakdown, SandboxScore};

/// Point values for sandbox scoring
mod points {
    pub const PLACE_DISCOVERED: f32 = 5.0;
    pub const PLACE_COMPLETED: f32 = 15.0;
    pub const NODE_BATTLE: f32 = 3.0;
    pub const NODE_CHOICE: f32 = 2.0;
    pub const NODE_EVENT: f32 = 2.0;
    pub const NODE_ENDING: f32 = 25.0;
    pub const NODE_TECH: f32 = 5.0;
    pub const NODE_TREASURE: f32 = 3.0;
    pub const NODE_LANDMARK: f32 = 8.0;
    pub const NODE_SPECIAL: f32 = 5.0;
    pub const TECH_TREE_COMPLETE: f32 = 50.0;
    pub const STORY_UNLOCKED: f32 = 10.0;
    pub const EVENT_COMPLETED: f32 = 1.0;
    pub const LOG_ENTRY: f32 = 2.0;
    pub const CHAPTER_COMPLETE: f32 = 15.0;
}

/// Calculate sandbox score for a user
pub fn calculate_sandbox_score(user: &User) -> SandboxScore {
    let deep_sea = &user.deep_sea;
    let mut breakdown = SandboxBreakdown::default();
    let mut area_scores = Vec::new();

    // Calculate places and nodes score by area (p1_ through p5_)
    for area_num in 1..=5 {
        let prefix = format!("p{}_", area_num);

        // Count places by state
        let mut places_completed = 0;
        let mut places_discovered = 0;
        let mut places_total = 0;

        for (key, state) in &deep_sea.places {
            if key.starts_with(&prefix) {
                places_total += 1;
                match *state {
                    2 => places_completed += 1,
                    1 => places_discovered += 1,
                    _ => {}
                }
            }
        }

        breakdown.places_completed += places_completed;
        breakdown.places_discovered += places_discovered;
        breakdown.places_total += places_total;

        // Calculate nodes for this area (nodes don't have area prefixes, so we count all)
        // We'll calculate nodes globally, not per-area
        let nodes_completed = 0;

        let places_score = (places_completed as f32 * points::PLACE_COMPLETED)
            + (places_discovered as f32 * points::PLACE_DISCOVERED);
        let nodes_score = 0.0; // Nodes calculated globally

        area_scores.push(SandboxAreaScore {
            area: area_num,
            places_score,
            nodes_score,
            total_score: places_score + nodes_score,
            places_completed,
            places_discovered,
            places_total,
            nodes_completed,
        });
    }

    // Calculate node breakdown by type (globally)
    for (key, state) in &deep_sea.nodes {
        if *state < 1 {
            continue; // Not completed
        }

        if key.starts_with("node_battle") {
            breakdown.battle_nodes_completed += 1;
        } else if key.starts_with("node_choice") {
            breakdown.choice_nodes_completed += 1;
        } else if key.starts_with("node_event") {
            breakdown.event_nodes_completed += 1;
        } else if key.starts_with("node_ending") {
            breakdown.ending_nodes_completed += 1;
        } else if key.starts_with("node_tech") {
            breakdown.tech_nodes_completed += 1;
        } else if key.starts_with("node_treasure") {
            breakdown.treasure_nodes_completed += 1;
        } else if key.starts_with("node_landmark") {
            breakdown.landmark_nodes_completed += 1;
        } else if key.starts_with("node_s") {
            breakdown.special_nodes_completed += 1;
        }
    }

    breakdown.total_nodes_completed = breakdown.battle_nodes_completed
        + breakdown.choice_nodes_completed
        + breakdown.event_nodes_completed
        + breakdown.ending_nodes_completed
        + breakdown.tech_nodes_completed
        + breakdown.treasure_nodes_completed
        + breakdown.landmark_nodes_completed
        + breakdown.special_nodes_completed;

    // Calculate tech trees completed
    if let Some(obj) = deep_sea.tech_trees.as_object() {
        breakdown.tech_trees_completed = obj
            .values()
            .filter(|v| {
                v.get("state")
                    .and_then(|s| s.as_i64())
                    .map(|s| s >= 2)
                    .unwrap_or(false)
            })
            .count() as i32;
    }

    // Calculate stories unlocked
    if let Some(obj) = deep_sea.stories.as_object() {
        breakdown.stories_unlocked = obj
            .values()
            .filter(|v| v.as_i64().map(|s| s > 0).unwrap_or(false))
            .count() as i32;
    }

    // Calculate events completed
    if let Some(obj) = deep_sea.events.as_object() {
        breakdown.events_completed = obj
            .values()
            .filter(|v| v.as_i64().map(|s| s > 0).unwrap_or(false))
            .count() as i32;
    }

    // Calculate log entries and chapters
    if let Some(obj) = deep_sea.logs.as_object() {
        breakdown.total_chapters = obj.len() as i32;
        for (_chapter, entries) in obj {
            if let Some(arr) = entries.as_array() {
                let entry_count = arr.len() as i32;
                breakdown.log_entries_collected += entry_count;
                if entry_count > 0 {
                    breakdown.chapters_with_logs += 1;
                }
            }
        }
    }

    // Calculate completion percentage
    if breakdown.places_total > 0 {
        breakdown.places_completion_percentage =
            (breakdown.places_completed as f32 / breakdown.places_total as f32) * 100.0;
    }

    // Calculate final scores
    let places_score = area_scores.iter().map(|a| a.places_score).sum();
    let nodes_score = (breakdown.battle_nodes_completed as f32 * points::NODE_BATTLE)
        + (breakdown.choice_nodes_completed as f32 * points::NODE_CHOICE)
        + (breakdown.event_nodes_completed as f32 * points::NODE_EVENT)
        + (breakdown.ending_nodes_completed as f32 * points::NODE_ENDING)
        + (breakdown.tech_nodes_completed as f32 * points::NODE_TECH)
        + (breakdown.treasure_nodes_completed as f32 * points::NODE_TREASURE)
        + (breakdown.landmark_nodes_completed as f32 * points::NODE_LANDMARK)
        + (breakdown.special_nodes_completed as f32 * points::NODE_SPECIAL);
    let tech_tree_score = breakdown.tech_trees_completed as f32 * points::TECH_TREE_COMPLETE;
    let stories_score = breakdown.stories_unlocked as f32 * points::STORY_UNLOCKED;
    let endings_score = breakdown.ending_nodes_completed as f32 * points::NODE_ENDING;
    let events_score = breakdown.events_completed as f32 * points::EVENT_COMPLETED;
    let logs_score = (breakdown.log_entries_collected as f32 * points::LOG_ENTRY)
        + (breakdown.chapters_with_logs as f32 * points::CHAPTER_COMPLETE);

    // Note: endings_score is already included in nodes_score, so we don't double count
    let total_score =
        places_score + nodes_score + tech_tree_score + stories_score + events_score + logs_score;

    SandboxScore {
        total_score,
        places_score,
        nodes_score,
        tech_tree_score,
        stories_score,
        endings_score, // For display purposes (subset of nodes_score)
        events_score,
        logs_score,
        area_scores,
        breakdown,
    }
}
