use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::{
    core::gamedata::types::{
        GameData,
        module::ModuleType,
        operator::{Operator, OperatorModule, OperatorProfession, OperatorRarity},
    },
    database::models::roster::RosterEntry,
};

/// Dimension weights
const WEIGHT_ELITE: f64 = 25.0;
const WEIGHT_MASTERY: f64 = 30.0;
const WEIGHT_MODULE: f64 = 25.0;
const WEIGHT_POTENTIAL: f64 = 10.0;
const WEIGHT_SKILL_LEVEL: f64 = 20.0;

/// Maximum partial credit when no milestone (M3 / Mod3) has been reached.
const PARTIAL_CAP: f64 = 0.30;

/// Bonus partial credit from non-maxed entries when at least one milestone exists.
const PARTIAL_BONUS: f64 = 0.10;

/// A scoring dimension: `(weight, score)` where score is 0.0–1.0.
type Dimension = (f64, f64);

#[derive(Deserialize)]
struct MasteryEntry {
    mastery: i16,
}

#[derive(Deserialize)]
struct ModuleEntry {
    id: String,
    level: i16,
}

fn parse_masteries(roster: &RosterEntry) -> Vec<MasteryEntry> {
    serde_json::from_value(roster.masteries.clone()).unwrap_or_default()
}

fn parse_modules(roster: &RosterEntry) -> Vec<ModuleEntry> {
    serde_json::from_value(roster.modules.clone()).unwrap_or_default()
}

pub fn grade_operators(roster: &[RosterEntry], game_data: &GameData) -> f64 {
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

        let Some(roster_entry) = roster_map.get(op_id.as_str()) else {
            continue;
        };

        if !has_investment(roster_entry) {
            continue;
        }

        let rarity_weight = rarity_to_weight(&static_op.rarity);
        let op_score = grade_operator(roster_entry, static_op);

        weighted_sum += op_score * rarity_weight;
        weight_total += rarity_weight;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

fn grade_operator(roster: &RosterEntry, static_op: &Operator) -> f64 {
    let max_elite = (static_op.phases.len() - 1) as f64;
    let num_skills = static_op.skills.len();
    let can_master = num_skills > 0 && static_op.phases.len() >= 3;
    let advanced_modules = advanced_modules(static_op);

    let mut dimensions: Vec<Dimension> = vec![];

    // Elite promotion progress
    if max_elite > 0.0 {
        let elite_score = roster.elite as f64 / max_elite;
        dimensions.push((WEIGHT_ELITE, elite_score));
    }

    // Level progress
    let level_score = cumulative_level_progress(roster, static_op);
    let level_weight = match static_op.rarity {
        OperatorRarity::SixStar => 15.0,
        OperatorRarity::FiveStar => 18.0,
        OperatorRarity::FourStar => 20.0,
        _ => 40.0,
    };
    dimensions.push((level_weight, level_score));

    // Skill level
    if !can_master && num_skills > 0 {
        let sl_score = (roster.skill_level - 1) as f64 / 6.0; // SL1=0, SL7=1.0
        dimensions.push((WEIGHT_SKILL_LEVEL, sl_score));
    }

    // Mastery
    if can_master {
        let mastery_score = mastery_milestone_score(roster, num_skills);
        dimensions.push((WEIGHT_MASTERY, mastery_score));
    }

    // Modules
    if !advanced_modules.is_empty() {
        let module_score = module_milestone_score(roster, &advanced_modules);
        dimensions.push((WEIGHT_MODULE, module_score));
    }

    // Potential
    if potential_matters(static_op) {
        let pot_score = (roster.potential - 1) as f64 / 5.0;
        dimensions.push((WEIGHT_POTENTIAL, pot_score));
    }

    let total_weight: f64 = dimensions.iter().map(|(w, _)| w).sum();
    dimensions.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight
}

/// Calculates cumulative level progress across all elite phases.
/// Returns 0.0–1.0 with logarithmic compression.
///
/// Example for a 6-star at E2 L60:
///   completed: E0 (50 levels) + E1 (80 levels) + 60 of E2
///   total:     50 + 80 + 90 = 220
///   raw ratio: (50 + 80 + 60) / 220 = 0.864
///   after log:  ~0.90
fn cumulative_level_progress(roster: &RosterEntry, static_op: &Operator) -> f64 {
    let mut progress = 0.0;
    let mut total = 0.0;

    for (i, phase) in static_op.phases.iter().enumerate() {
        let max_lvl = phase.max_level as f64;
        total += max_lvl;

        if (i as i16) < roster.elite {
            progress += max_lvl;
        } else if i as i16 == roster.elite {
            progress += roster.level as f64;
        }
    }

    if total == 0.0 {
        return 1.0;
    }

    let raw = progress / total;
    log_curve_ratio(raw)
}

/// Returns 0.0–1.0 based on mastery milestones.
///
/// Without any M3, partial credit is capped at `PARTIAL_CAP` (0.30).
/// With M3 skills: 1 → 0.50, 2 → 0.75, all → 1.00, plus partial bonus.
fn mastery_milestone_score(roster: &RosterEntry, num_skills: usize) -> f64 {
    let masteries = parse_masteries(roster);
    let m3_count = masteries.iter().filter(|m| m.mastery >= 3).count();

    if m3_count == 0 {
        let total: f64 = masteries.iter().map(|m| m.mastery as f64).sum();
        let max = num_skills as f64 * 3.0;
        if max > 0.0 {
            (total / max) * PARTIAL_CAP
        } else {
            0.0
        }
    } else {
        let base = match m3_count {
            1 => 0.50,
            2 => 0.75,
            _ => 1.00,
        };

        let remaining_skills = num_skills - m3_count;
        if remaining_skills > 0 {
            let non_m3_mastery: f64 = masteries
                .iter()
                .filter(|m| m.mastery < 3)
                .map(|m| m.mastery as f64)
                .sum();
            let remaining_max = remaining_skills as f64 * 3.0;
            let partial = (non_m3_mastery / remaining_max) * PARTIAL_BONUS;
            (base + partial).min(1.0)
        } else {
            base
        }
    }
}

/// Returns 0.0–1.0 based on module milestones.
///
/// Without any Mod3, partial credit is capped at `PARTIAL_CAP` (0.30).
/// With Mod3: first → 0.50, second → 0.80, all → 1.00, plus partial bonus.
fn module_milestone_score(roster: &RosterEntry, advanced_modules: &[&OperatorModule]) -> f64 {
    let user_modules = parse_modules(roster);
    let num_available = advanced_modules.len();
    if num_available == 0 {
        return 0.0;
    }

    let advanced_ids: HashSet<&str> = advanced_modules
        .iter()
        .map(|m| m.module.uni_equip_id.as_str())
        .collect();

    let user_advanced: Vec<i16> = user_modules
        .iter()
        .filter(|m| advanced_ids.contains(m.id.as_str()))
        .map(|m| m.level)
        .collect();

    let mod3_count = user_advanced.iter().filter(|&&lvl| lvl >= 3).count();

    if mod3_count == 0 {
        let total_levels: f64 = user_advanced.iter().map(|&l| l as f64).sum();
        let max_total = num_available as f64 * 3.0;
        (total_levels / max_total) * PARTIAL_CAP
    } else {
        let base = mod3_count as f64 / num_available as f64;
        let milestone = base.max(0.50);

        let non_max_levels: f64 = user_advanced
            .iter()
            .filter(|&&lvl| lvl < 3)
            .map(|&l| l as f64)
            .sum();
        let remaining_max = (num_available - mod3_count) as f64 * 3.0;
        let partial = if remaining_max > 0.0 {
            (non_max_levels / remaining_max) * PARTIAL_BONUS
        } else {
            0.0
        };

        (milestone + partial).min(1.0)
    }
}

fn advanced_modules(static_op: &Operator) -> Vec<&OperatorModule> {
    static_op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect()
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

fn potential_matters(static_op: &Operator) -> bool {
    !static_op.can_use_general_potential_item || static_op.is_sp_char
}

/// Returns true if the player has invested beyond the initial pull state (E0 L1).
fn has_investment(roster: &RosterEntry) -> bool {
    roster.elite > 0 || roster.level > 1
}

/// Log compression on a 0–1 ratio.
fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}
