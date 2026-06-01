use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::{
    core::gamedata::types::{
        GameData,
        module::ModuleType,
        operator::{Operator, OperatorModule, OperatorProfession, OperatorRarity},
        trust::Favor,
    },
    database::models::roster::RosterEntry,
};

use super::{Dimension, weighted_average};

/// Dimension weights
const WEIGHT_ELITE: f64 = 25.0;
const WEIGHT_MASTERY: f64 = 30.0;
const WEIGHT_MODULE: f64 = 25.0;
const WEIGHT_POTENTIAL: f64 = 10.0;
const WEIGHT_SKILL_LEVEL: f64 = 20.0;
/// Trust is mostly passive accrual through usage, so it weighs lighter than
/// the active investment dimensions but still rewards engagement.
const WEIGHT_TRUST: f64 = 5.0;

/// Trust percent at which an ordinary operator is considered fully invested.
/// Reaching this maps to a perfect trust dimension; trust beyond doesn't help
/// non-support ops. Support-unit operators are held to `max_favor` instead -
/// publishing an op for others to borrow implies completionist intent.
const TRUST_MILESTONE_PCT: f64 = 100.0;

/// Maximum partial credit when no milestone (M3 / Mod3) has been reached.
const PARTIAL_CAP: f64 = 0.30;

/// Bonus partial credit from non-maxed entries when at least one milestone exists.
const PARTIAL_BONUS: f64 = 0.10;

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

/// Index a roster by operator id for O(1) lookups.
fn build_roster_map(roster: &[RosterEntry]) -> HashMap<&str, &RosterEntry> {
    roster.iter().map(|r| (r.operator_id.as_str(), r)).collect()
}

/// The operators that count toward `operator_grade`: real, obtainable operators the
/// roster has invested in, paired with their static data and roster entry. This is
/// the single source of truth for "gradeable operator" - share it everywhere.
fn invested_operators<'a>(
    roster_map: &'a HashMap<&'a str, &'a RosterEntry>,
    game_data: &'a GameData,
) -> impl Iterator<Item = (&'a str, &'a Operator, &'a RosterEntry)> {
    game_data
        .operators
        .iter()
        .filter(|(_, op)| {
            !matches!(
                op.profession,
                OperatorProfession::Token | OperatorProfession::Trap
            ) && !op.is_not_obtainable
        })
        .filter_map(move |(op_id, static_op)| {
            let entry = *roster_map.get(op_id.as_str())?;
            has_investment(entry).then_some((op_id.as_str(), static_op, entry))
        })
}

pub fn grade_operators(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
) -> f64 {
    let roster_map = build_roster_map(roster);
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (op_id, static_op, roster_entry) in invested_operators(&roster_map, game_data) {
        let rarity_weight = rarity_to_weight(&static_op.rarity);
        let is_support = support_ids.contains(op_id);
        let op_score = grade_operator(roster_entry, static_op, &game_data.favor, is_support);

        weighted_sum += op_score * rarity_weight;
        weight_total += rarity_weight;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

/// Returns the sum of rarity weights across all roster entries that count
/// toward `operator_grade` - i.e. the same set that `grade_operators` iterates.
/// Used by the improvements builder to translate per-operator score deltas
/// into a contribution against the user's overall Operators subscore.
pub fn total_roster_weight(roster: &[RosterEntry], game_data: &GameData) -> f64 {
    let roster_map = build_roster_map(roster);
    invested_operators(&roster_map, game_data)
        .map(|(_, static_op, _)| rarity_to_weight(&static_op.rarity))
        .sum()
}

pub fn grade_operator(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
) -> f64 {
    let dims = build_dimensions(roster, static_op, favor, is_support);
    weighted_average(&dims)
}

fn build_dimensions(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
) -> Vec<Dimension> {
    let max_elite = (static_op.phases.len() - 1) as f64;
    let num_skills = static_op.skills.len();
    let can_master = num_skills > 0 && static_op.phases.len() >= 3;
    let advanced_modules = advanced_modules(static_op);

    let mut dimensions: Vec<Dimension> = vec![];

    // Elite promotion progress
    if max_elite > 0.0 {
        let elite_score = f64::from(roster.elite) / max_elite;
        dimensions.push((WEIGHT_ELITE, elite_score));
    }

    // Level progress
    let level_score = cumulative_level_progress(roster, static_op);
    dimensions.push((level_weight(&static_op.rarity), level_score));

    // Skill level
    if !can_master && num_skills > 0 {
        let sl_score = f64::from(roster.skill_level - 1) / 6.0; // SL1=0, SL7=1.0
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
        let pot_score = f64::from(roster.potential - 1) / 5.0;
        dimensions.push((WEIGHT_POTENTIAL, pot_score));
    }

    // Trust - only meaningful once the favor table is loaded.
    if favor.max_trust_pct() > 0.0 {
        let trust_score = trust_milestone_score(roster, favor, is_support);
        dimensions.push((WEIGHT_TRUST, trust_score));
    }

    dimensions
}

const fn level_weight(rarity: &OperatorRarity) -> f64 {
    match rarity {
        OperatorRarity::SixStar => 15.0,
        OperatorRarity::FiveStar => 18.0,
        OperatorRarity::FourStar => 20.0,
        _ => 40.0,
    }
}

/// Calculates cumulative level progress across all elite phases.
/// Returns 0.0-1.0 with logarithmic compression.
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
        let max_lvl = f64::from(phase.max_level);
        total += max_lvl;

        if (i as i16) < roster.elite {
            progress += max_lvl;
        } else if i as i16 == roster.elite {
            progress += f64::from(roster.level);
        }
    }

    if total == 0.0 {
        return 1.0;
    }

    let raw = progress / total;
    log_curve_ratio(raw)
}

/// Same shape as `cumulative_level_progress` but for an overridden (elite, level)
/// pair - used by the delta simulator without mutating the `RosterEntry`.
fn cumulative_level_progress_at(static_op: &Operator, elite: i16, level: i16) -> f64 {
    let mut progress = 0.0;
    let mut total = 0.0;
    for (i, phase) in static_op.phases.iter().enumerate() {
        let max_lvl = f64::from(phase.max_level);
        total += max_lvl;
        if (i as i16) < elite {
            progress += max_lvl;
        } else if i as i16 == elite {
            progress += f64::from(level);
        }
    }
    if total == 0.0 {
        return 1.0;
    }
    log_curve_ratio(progress / total)
}

/// Returns 0.0-1.0 based on mastery milestones.
///
/// Without any M3, partial credit is capped at `PARTIAL_CAP` (0.30).
/// With M3 skills: 1 → 0.50, 2 → 0.75, all → 1.00, plus partial bonus.
fn mastery_milestone_score(roster: &RosterEntry, num_skills: usize) -> f64 {
    let masteries = parse_masteries(roster);
    mastery_milestone_from_levels(
        &masteries.iter().map(|m| m.mastery).collect::<Vec<_>>(),
        num_skills,
    )
}

fn mastery_milestone_from_levels(levels: &[i16], num_skills: usize) -> f64 {
    let m3_count = levels.iter().filter(|&&m| m >= 3).count();

    if m3_count == 0 {
        let total: f64 = levels.iter().map(|&m| f64::from(m)).sum();
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
            let non_m3_mastery: f64 = levels
                .iter()
                .filter(|&&m| m < 3)
                .map(|&m| f64::from(m))
                .sum();
            let remaining_max = remaining_skills as f64 * 3.0;
            let partial = (non_m3_mastery / remaining_max) * PARTIAL_BONUS;
            (base + partial).min(1.0)
        } else {
            base
        }
    }
}

/// Returns 0.0-1.0 based on module milestones.
///
/// Without any Mod3, partial credit is capped at `PARTIAL_CAP` (0.30).
/// With Mod3: first → 0.50, second → 0.80, all → 1.00, plus partial bonus.
fn module_milestone_score(roster: &RosterEntry, advanced_modules: &[&OperatorModule]) -> f64 {
    let user_modules = parse_modules(roster);
    let advanced_ids: HashSet<&str> = advanced_modules
        .iter()
        .map(|m| m.module.uni_equip_id.as_str())
        .collect();

    let user_advanced: Vec<i16> = user_modules
        .iter()
        .filter(|m| advanced_ids.contains(m.id.as_str()))
        .map(|m| m.level)
        .collect();
    module_milestone_from_levels(&user_advanced, advanced_modules.len())
}

fn module_milestone_from_levels(user_advanced: &[i16], num_available: usize) -> f64 {
    if num_available == 0 {
        return 0.0;
    }

    let mod3_count = user_advanced.iter().filter(|&&lvl| lvl >= 3).count();

    if mod3_count == 0 {
        let total_levels: f64 = user_advanced.iter().map(|&l| f64::from(l)).sum();
        let max_total = num_available as f64 * 3.0;
        (total_levels / max_total) * PARTIAL_CAP
    } else {
        let base = mod3_count as f64 / num_available as f64;
        let milestone = base.max(0.50);

        let non_max_levels: f64 = user_advanced
            .iter()
            .filter(|&&lvl| lvl < 3)
            .map(|&l| f64::from(l))
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

/// Returns 0.0-1.0 based on trust progress.
///
/// The target trust depends on whether the operator is currently published as
/// a support unit:
///   - Ordinary roster ops: full score at `TRUST_MILESTONE_PCT` (100% trust).
///     Trust beyond doesn't help - 100 is "complete".
///   - Support-unit ops: full score only at the favor table's max (typically
///     200% trust). Falls linearly below that, so a published op at 100 trust
///     scores ~0.5 and drags the dimension down.
///
/// All thresholds derive from the favor table so the curve adjusts
/// automatically if the game ships a different max trust.
fn trust_milestone_score(roster: &RosterEntry, favor: &Favor, is_support: bool) -> f64 {
    let trust_pct = favor.trust_pct(roster.favor_point);
    let max_pct = favor.max_trust_pct();
    let target = if is_support {
        max_pct
    } else {
        TRUST_MILESTONE_PCT.min(max_pct)
    };
    if target <= 0.0 {
        return 0.0;
    }
    (trust_pct / target).clamp(0.0, 1.0)
}

fn advanced_modules(static_op: &Operator) -> Vec<&OperatorModule> {
    static_op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect()
}

pub const fn rarity_to_weight(rarity: &OperatorRarity) -> f64 {
    match rarity {
        OperatorRarity::SixStar => 1.0,
        OperatorRarity::FiveStar => 0.7,
        OperatorRarity::FourStar => 0.4,
        OperatorRarity::ThreeStar => 0.15,
        OperatorRarity::TwoStar => 0.1,
        OperatorRarity::OneStar => 0.05,
    }
}

const fn potential_matters(static_op: &Operator) -> bool {
    !static_op.can_use_general_potential_item || static_op.is_sp_char
}

/// Returns true if the player has invested beyond the initial pull state (E0 L1).
pub const fn has_investment(roster: &RosterEntry) -> bool {
    roster.elite > 0 || roster.level > 1
}

/// Log compression on a 0-1 ratio.
fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}

/// Score gain from a single upgrade path.
///
/// All deltas are reported as non-negative - if simulating the milestone would
/// somehow not improve the score (shouldn't happen with the current model,
/// but defensive), the delta is clamped to 0.
#[derive(Debug, Clone, Serialize)]
pub struct UpgradeDelta {
    /// Tag from `OperatorGap.missing`, e.g. "ELITE", "M3", "MOD3", "TRUST".
    pub tag: &'static str,
    /// Δ in this operator's score (0.0-1.0). The change in `grade_operator(...)`
    /// before any rarity weighting.
    pub operator_score_delta: f64,
    /// Δ contribution to the user's `operator_grade` subscore (0.0-1.0).
    /// Already accounts for this operator's rarity weight and the total
    /// roster weight.
    pub operator_grade_delta: f64,
    /// Δ contribution to the user's `total_score` (0.0-1.0). Accounts for
    /// the Operators subscore's share of the overall grade (1.0 / 2.6).
    pub total_score_delta: f64,
}

/// Total of all category weights in `calculate_user_grade` (operator 1.0 +
/// base 0.5 + roguelike 0.3 + medal 0.2 + stage 0.4 + sandbox 0.2).
/// Kept in sync manually with `core::grade::calculate`.
const TOTAL_CATEGORY_WEIGHT: f64 = 2.6;

/// Compute per-tag deltas for the given operator. `missing` is the list of
/// upgrade tags surfaced in `OperatorGap.missing`; each gets a simulated
/// "what if you completed this milestone" score against the current state.
///
/// `rarity_weight` and `total_roster_weight` are the inputs from
/// `grade_operators` used to translate per-op delta → `operator_grade` delta.
pub fn operator_upgrade_deltas(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    missing: &[&'static str],
    rarity_weight: f64,
    total_roster_weight: f64,
) -> Vec<UpgradeDelta> {
    let current_dims = build_dimensions(roster, static_op, favor, is_support);
    let total_weight: f64 = current_dims.iter().map(|(w, _)| w).sum();
    if total_weight <= 0.0 {
        return Vec::new();
    }
    let current_score = current_dims.iter().map(|(w, s)| w * s).sum::<f64>() / total_weight;

    let mut out = Vec::with_capacity(missing.len());
    for &tag in missing {
        let new_score = simulate_score_for_tag(roster, static_op, favor, is_support, tag);
        let Some(new_score) = new_score else { continue };
        let op_delta = (new_score - current_score).max(0.0);
        let grade_delta = if total_roster_weight > 0.0 {
            op_delta * rarity_weight / total_roster_weight
        } else {
            0.0
        };
        let total_delta = grade_delta / TOTAL_CATEGORY_WEIGHT;
        out.push(UpgradeDelta {
            tag,
            operator_score_delta: op_delta,
            operator_grade_delta: grade_delta,
            total_score_delta: total_delta,
        });
    }
    out
}

/// Rebuild dimensions with one milestone reached, then average. Returns `None`
/// if the tag isn't applicable to this operator (defensive - caller already
/// filtered, but cheap to recheck).
fn simulate_score_for_tag(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    tag: &str,
) -> Option<f64> {
    let max_elite = (static_op.phases.len() - 1) as i16;
    let num_skills = static_op.skills.len();
    let can_master = num_skills > 0 && static_op.phases.len() >= 3;
    let advanced_mods = advanced_modules(static_op);
    let level_w = level_weight(&static_op.rarity);
    let trust_active = favor.max_trust_pct() > 0.0;

    // Build the current set of dimensions, but capture each component so we
    // can selectively replace one (or two, for ELITE which also lifts the
    // level dimension).
    let mut elite_dim: Option<Dimension> = if max_elite > 0 {
        Some((WEIGHT_ELITE, f64::from(roster.elite) / f64::from(max_elite)))
    } else {
        None
    };
    let mut level_dim: Dimension = (level_w, cumulative_level_progress(roster, static_op));
    let mut sl_dim: Option<Dimension> = if !can_master && num_skills > 0 {
        Some((WEIGHT_SKILL_LEVEL, f64::from(roster.skill_level - 1) / 6.0))
    } else {
        None
    };
    let mut mastery_dim: Option<Dimension> = if can_master {
        Some((WEIGHT_MASTERY, mastery_milestone_score(roster, num_skills)))
    } else {
        None
    };
    let mut module_dim: Option<Dimension> = if advanced_mods.is_empty() {
        None
    } else {
        Some((
            WEIGHT_MODULE,
            module_milestone_score(roster, &advanced_mods),
        ))
    };
    let mut pot_dim: Option<Dimension> = if potential_matters(static_op) {
        Some((WEIGHT_POTENTIAL, f64::from(roster.potential - 1) / 5.0))
    } else {
        None
    };
    let mut trust_dim: Option<Dimension> = if trust_active {
        Some((
            WEIGHT_TRUST,
            trust_milestone_score(roster, favor, is_support),
        ))
    } else {
        None
    };

    match tag {
        // Full promotion path: jump to max elite + max level at that phase.
        // Both elite and the level dimensions go to 1.0.
        "ELITE" => {
            if max_elite <= 0 {
                return None;
            }
            elite_dim = Some((WEIGHT_ELITE, 1.0));
            level_dim = (level_w, 1.0);
        }
        // Max level at current elite phase only.
        "MAX_LEVEL" => {
            let max_lvl_here = static_op
                .phases
                .get(roster.elite as usize)
                .map_or(0, |p| p.max_level as i16);
            level_dim = (
                level_w,
                cumulative_level_progress_at(static_op, roster.elite, max_lvl_here),
            );
        }
        // First mastery to M3. We assume the highest currently non-M3 skill is
        // the one promoted, which is the most generous read of the user's
        // current trajectory.
        "M3" => {
            if !can_master {
                return None;
            }
            let levels: Vec<i16> = parse_masteries(roster).iter().map(|m| m.mastery).collect();
            let simulated = promote_to_milestone(&levels, num_skills, 3);
            mastery_dim = Some((
                WEIGHT_MASTERY,
                mastery_milestone_from_levels(&simulated, num_skills),
            ));
        }
        // Skill level → 7 (the dimension's max).
        "SL7" => {
            if can_master || num_skills == 0 {
                return None;
            }
            sl_dim = Some((WEIGHT_SKILL_LEVEL, 1.0));
        }
        // First advanced module to L3 (same "promote highest non-Mod3" model
        // as M3 above).
        "MOD3" => {
            if advanced_mods.is_empty() {
                return None;
            }
            let advanced_ids: HashSet<&str> = advanced_mods
                .iter()
                .map(|m| m.module.uni_equip_id.as_str())
                .collect();
            let user_advanced: Vec<i16> = parse_modules(roster)
                .iter()
                .filter(|m| advanced_ids.contains(m.id.as_str()))
                .map(|m| m.level)
                .collect();
            // Pad with zeros so the slice length matches the number of slots -
            // module_milestone_from_levels treats absent slots as level 0.
            let mut padded = user_advanced;
            while padded.len() < advanced_mods.len() {
                padded.push(0);
            }
            let simulated = promote_to_milestone(&padded, advanced_mods.len(), 3);
            module_dim = Some((
                WEIGHT_MODULE,
                module_milestone_from_levels(&simulated, advanced_mods.len()),
            ));
        }
        "POT6" => {
            if !potential_matters(static_op) {
                return None;
            }
            pot_dim = Some((WEIGHT_POTENTIAL, 1.0));
        }
        "TRUST" => {
            if !trust_active {
                return None;
            }
            trust_dim = Some((WEIGHT_TRUST, 1.0));
        }
        _ => return None,
    }

    let mut dims: Vec<Dimension> = vec![level_dim];
    if let Some(d) = elite_dim {
        dims.push(d);
    }
    if let Some(d) = sl_dim {
        dims.push(d);
    }
    if let Some(d) = mastery_dim {
        dims.push(d);
    }
    if let Some(d) = module_dim {
        dims.push(d);
    }
    if let Some(d) = pot_dim {
        dims.push(d);
    }
    if let Some(d) = trust_dim {
        dims.push(d);
    }
    Some(weighted_average(&dims))
}

/// Promote the highest entry below `milestone` up to `milestone`. Used to
/// model "one more M3" / "one more Mod3" - most generous interpretation of
/// which slot the user would push.
fn promote_to_milestone(levels: &[i16], slots: usize, milestone: i16) -> Vec<i16> {
    let mut padded: Vec<i16> = levels.to_vec();
    while padded.len() < slots {
        padded.push(0);
    }
    // Find index of the highest entry strictly below milestone.
    let pick = padded
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v < milestone)
        .max_by_key(|&(_, &v)| v)
        .map(|(i, _)| i);
    if let Some(idx) = pick {
        padded[idx] = milestone;
    }
    padded
}
