use std::collections::{HashMap, HashSet};

use myrtle_gamedata_raw::tables::{
    module::ModuleType,
    operator::{Operator, OperatorModule, OperatorProfession, OperatorRarity},
    trust::Favor,
};
use myrtle_types::roster::RosterEntry;
use serde::{Deserialize, Serialize};

use crate::{
    Dimension, GradeData as GameData,
    total::{SECTION_WEIGHT_OPERATOR, SECTION_WEIGHT_TOTAL},
    weighted_average,
};

const WEIGHT_ELITE: f64 = 35.0;
const WEIGHT_MASTERY: f64 = 30.0;

const WEIGHT_MODULE: f64 = 25.0;
const WEIGHT_POTENTIAL: f64 = 10.0;
const WEIGHT_SKILL_LEVEL: f64 = 20.0;

const WEIGHT_TRUST: f64 = 5.0;

pub const TRUST_MILESTONE_PCT: f64 = 100.0;

const MILESTONE: i16 = 3;

const PARTIAL_CAP: f64 = 0.30;

const PARTIAL_BONUS: f64 = 0.10;

const RARITY_WEIGHTS: [f64; 6] = [1.0, 0.5, 0.3, 0.05, 0.02, 0.01];
const LEGACY_RARITY_WEIGHTS: [f64; 6] = [1.0, 0.7, 0.4, 0.15, 0.1, 0.05];

#[derive(Deserialize)]
struct MasteryEntry {
    mastery: i16,
}

#[derive(Deserialize)]
struct ModuleEntry {
    id: String,
    level: i16,
    #[serde(default)]
    locked: bool,
}

fn mastery_levels(masteries_json: &serde_json::Value) -> Vec<i16> {
    Vec::<MasteryEntry>::deserialize(masteries_json)
        .unwrap_or_default()
        .iter()
        .map(|m| m.mastery)
        .collect()
}

fn parse_modules(modules_json: &serde_json::Value) -> Vec<ModuleEntry> {
    let mut modules = Vec::<ModuleEntry>::deserialize(modules_json).unwrap_or_default();
    modules.retain(|m| !m.locked);
    modules
}

#[must_use]
pub fn advanced_module_levels(
    modules_json: &serde_json::Value,
    advanced: &[&OperatorModule],
) -> Vec<i16> {
    let advanced_ids: HashSet<&str> = advanced
        .iter()
        .map(|m| m.module.uni_equip_id.as_str())
        .collect();
    parse_modules(modules_json)
        .iter()
        .filter(|m| advanced_ids.contains(m.id.as_str()))
        .map(|m| m.level)
        .collect()
}

fn build_roster_map(roster: &[RosterEntry]) -> HashMap<&str, &RosterEntry> {
    roster.iter().map(|r| (r.operator_id.as_str(), r)).collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreModel {
    pub invested_only: bool,
    pub legacy_rarity_weights: bool,
    pub module_per_slot: bool,
}

impl ScoreModel {
    pub const SHIPPED: Self = Self {
        invested_only: false,
        legacy_rarity_weights: false,
        module_per_slot: true,
    };

    pub fn from_vars(var: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            invested_only: var("GRADE_INVESTED_ONLY").is_some_and(|v| v == "1"),
            legacy_rarity_weights: var("GRADE_RARITY_WEIGHTS").is_some_and(|v| v == "legacy"),
            module_per_slot: var("GRADE_MODULE_PER_SLOT").is_none_or(|v| v != "0"),
        }
    }
}

#[must_use]
pub const fn is_graded(entry: &RosterEntry, model: ScoreModel) -> bool {
    !model.invested_only || has_investment(entry)
}

fn graded_operators_in<'a>(
    roster_map: &'a HashMap<&'a str, &'a RosterEntry>,
    game_data: &'a GameData,
    invested_only: bool,
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
            (!invested_only || has_investment(entry)).then_some((op_id.as_str(), static_op, entry))
        })
}

#[must_use]
pub fn grade_operators_in(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
    model: ScoreModel,
) -> f64 {
    let roster_map = build_roster_map(roster);
    let mut weighted_sum = 0.0;
    let mut weight_total = 0.0;

    for (op_id, static_op, roster_entry) in
        graded_operators_in(&roster_map, game_data, model.invested_only)
    {
        let rarity_weight = rarity_to_weight_in(&static_op.rarity, model.legacy_rarity_weights);
        let is_support = support_ids.contains(op_id);
        let op_score = grade_operator_in(
            roster_entry,
            static_op,
            &game_data.favor,
            is_support,
            &model,
        );

        weighted_sum = op_score.mul_add(rarity_weight, weighted_sum);
        weight_total += rarity_weight;
    }

    if weight_total > 0.0 {
        weighted_sum / weight_total
    } else {
        0.0
    }
}

#[must_use]
pub fn total_roster_weight_in(
    roster: &[RosterEntry],
    game_data: &GameData,
    model: ScoreModel,
) -> f64 {
    let roster_map = build_roster_map(roster);
    graded_operators_in(&roster_map, game_data, model.invested_only)
        .map(|(_, static_op, _)| {
            rarity_to_weight_in(&static_op.rarity, model.legacy_rarity_weights)
        })
        .sum()
}

#[must_use]
pub fn grade_operator_in(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    model: &ScoreModel,
) -> f64 {
    average_dimensions(&build_dimensions(
        roster, static_op, favor, is_support, *model,
    ))
}

fn average_dimensions(dims: &[(DimensionKind, Dimension)]) -> f64 {
    let unlabeled: Vec<Dimension> = dims.iter().map(|(_, dim)| *dim).collect();
    weighted_average(&unlabeled)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DimensionKind {
    Elite,
    Level,
    SkillLevel,
    Mastery,
    Module,
    Potential,
    Trust,
}

const DIMENSION_ORDER: [DimensionKind; 7] = [
    DimensionKind::Elite,
    DimensionKind::Level,
    DimensionKind::Mastery,
    DimensionKind::SkillLevel,
    DimensionKind::Module,
    DimensionKind::Potential,
    DimensionKind::Trust,
];

fn build_dimensions(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    model: ScoreModel,
) -> Vec<(DimensionKind, Dimension)> {
    let max_elite = (static_op.phases.len() - 1) as f64;
    let num_skills = static_op.skills.len();
    let can_master = num_skills > 0 && static_op.phases.len() >= 3;
    let advanced_modules = advanced_modules(static_op);

    let mut dimensions: Vec<(DimensionKind, Dimension)> = vec![];

    if max_elite > 0.0 {
        let elite_score = f64::from(roster.elite) / max_elite;
        dimensions.push((DimensionKind::Elite, (WEIGHT_ELITE, elite_score)));
    }

    let level_score = cumulative_level_progress(roster, static_op);
    dimensions.push((
        DimensionKind::Level,
        (level_weight(&static_op.rarity), level_score),
    ));

    if !can_master && num_skills > 0 {
        let sl_score = f64::from(roster.skill_level - 1) / 6.0; // SL1=0, SL7=1.0
        dimensions.push((DimensionKind::SkillLevel, (WEIGHT_SKILL_LEVEL, sl_score)));
    }

    if can_master {
        let mastery_score = mastery_milestone_score(roster, num_skills);
        dimensions.push((DimensionKind::Mastery, (WEIGHT_MASTERY, mastery_score)));
    }

    if !advanced_modules.is_empty() {
        let module_score = module_milestone_score(roster, &advanced_modules, model);
        dimensions.push((
            DimensionKind::Module,
            (module_weight(advanced_modules.len(), model), module_score),
        ));
    }

    if potential_matters(static_op) {
        dimensions.push((
            DimensionKind::Potential,
            (WEIGHT_POTENTIAL, potential_score(roster.potential)),
        ));
    }

    // Trust needs the favor table loaded.
    if favor.max_trust_pct() > 0.0 {
        let trust_score = trust_milestone_score(roster, favor, is_support);
        dimensions.push((DimensionKind::Trust, (WEIGHT_TRUST, trust_score)));
    }

    dimensions
}

#[derive(Debug, Clone, Serialize)]
pub struct ScoreDimension {
    pub kind: DimensionKind,
    pub weight_share: f64,
    pub completion: f64,
    pub contribution: f64,
}

#[must_use]
pub fn operator_score_breakdown_in(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
    model: ScoreModel,
) -> Vec<ScoreDimension> {
    let roster_map = build_roster_map(roster);
    let rarity_weight =
        |static_op: &Operator| rarity_to_weight_in(&static_op.rarity, model.legacy_rarity_weights);
    let total_rarity_weight = total_roster_weight_in(roster, game_data, model);
    if total_rarity_weight <= 0.0 {
        return Vec::new();
    }

    let mut shares: HashMap<DimensionKind, (f64, f64)> = HashMap::new();
    for (op_id, static_op, entry) in
        graded_operators_in(&roster_map, game_data, model.invested_only)
    {
        let is_support = support_ids.contains(op_id);
        let dims = build_dimensions(entry, static_op, &game_data.favor, is_support, model);
        let op_weight_total: f64 = dims.iter().map(|(_, (w, _))| w).sum();
        if op_weight_total <= 0.0 {
            continue;
        }
        let op_share = rarity_weight(static_op) / total_rarity_weight;
        for (kind, (weight, score)) in dims {
            let dim_share = op_share * weight / op_weight_total;
            let slot = shares.entry(kind).or_insert((0.0, 0.0));
            slot.0 += dim_share;
            slot.1 = dim_share.mul_add(score, slot.1);
        }
    }

    DIMENSION_ORDER
        .iter()
        .filter_map(|kind| {
            let &(weight_share, contribution) = shares.get(kind)?;
            (weight_share > 0.0).then_some(ScoreDimension {
                kind: *kind,
                weight_share,
                completion: contribution / weight_share,
                contribution,
            })
        })
        .collect()
}

const fn level_weight(rarity: &OperatorRarity) -> f64 {
    match rarity {
        OperatorRarity::SixStar | OperatorRarity::FiveStar | OperatorRarity::FourStar => 25.0,
        _ => 40.0,
    }
}

fn cumulative_level_progress(roster: &RosterEntry, static_op: &Operator) -> f64 {
    cumulative_level_progress_at(static_op, roster.elite, roster.level)
}

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

fn mastery_milestone_score(roster: &RosterEntry, num_skills: usize) -> f64 {
    milestone_score(
        &mastery_levels(&roster.masteries),
        num_skills,
        mastery_ladder,
    )
}

fn module_milestone_score(
    roster: &RosterEntry,
    advanced_modules: &[&OperatorModule],
    model: ScoreModel,
) -> f64 {
    let levels = advanced_module_levels(&roster.modules, advanced_modules);
    milestone_score(&levels, advanced_modules.len(), module_ladder(model))
}

fn milestone_score(levels: &[i16], slots: usize, ladder: fn(usize, usize) -> f64) -> f64 {
    let reached = levels.iter().filter(|&&l| l >= MILESTONE).count();
    if reached == 0 {
        return sub_milestone_progress(levels, slots) * PARTIAL_CAP;
    }
    let remaining = slots.saturating_sub(reached);
    sub_milestone_progress(levels, remaining).mul_add(PARTIAL_BONUS, ladder(reached, slots)).min(1.0)
}

fn sub_milestone_progress(levels: &[i16], slots: usize) -> f64 {
    let max = slots as f64 * f64::from(MILESTONE);
    if max <= 0.0 {
        return 0.0;
    }
    let climbed: f64 = levels
        .iter()
        .filter(|&&l| l < MILESTONE)
        .map(|&l| f64::from(l))
        .sum();
    climbed / max
}

const fn mastery_ladder(reached: usize, slots: usize) -> f64 {
    if reached >= slots {
        return 1.0;
    }
    match reached {
        1 => 0.50,
        2 => 0.75,
        _ => 1.00,
    }
}

fn module_ladder(model: ScoreModel) -> fn(usize, usize) -> f64 {
    if model.module_per_slot {
        module_share
    } else {
        module_share_floored
    }
}

fn module_share(reached: usize, slots: usize) -> f64 {
    reached as f64 / slots as f64
}

fn module_share_floored(reached: usize, slots: usize) -> f64 {
    module_share(reached, slots).max(0.50)
}

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

#[must_use]
pub fn advanced_modules(static_op: &Operator) -> Vec<&OperatorModule> {
    static_op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect()
}

fn module_weight(slots: usize, model: ScoreModel) -> f64 {
    if model.module_per_slot {
        WEIGHT_MODULE * slots as f64
    } else {
        WEIGHT_MODULE
    }
}

#[must_use]
pub const fn rarity_to_weight_in(rarity: &OperatorRarity, legacy: bool) -> f64 {
    let table = if legacy {
        &LEGACY_RARITY_WEIGHTS
    } else {
        &RARITY_WEIGHTS
    };
    table[(6 - rarity.to_star_int()) as usize]
}

#[must_use]
pub const fn potential_matters(static_op: &Operator) -> bool {
    !static_op.can_use_general_potential_item || static_op.is_sp_char
}

fn potential_score(potential: i16) -> f64 {
    (f64::from(potential) / 5.0).clamp(0.0, 1.0)
}

#[must_use]
pub const fn has_investment(roster: &RosterEntry) -> bool {
    roster.elite > 0 || roster.level > 1
}

fn log_curve_ratio(t: f64) -> f64 {
    t.ln_1p() / 2.0_f64.ln()
}

#[derive(Debug, Clone, Serialize)]
pub struct UpgradeDelta {
    /// Tag from `OperatorGap.missing`, e.g. "ELITE", "M3", "MOD3", "TRUST"
    pub tag: &'static str,
    /// Δ in this operator's score (0.0-1.0)
    pub operator_score_delta: f64,
    /// Δ contribution to the user's `operator_grade` subscore (0.0-1.0)
    pub operator_grade_delta: f64,
    /// Δ contribution to the user's `total_score` (0.0-1.0)
    pub total_score_delta: f64,
}

#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn operator_upgrade_deltas(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    missing: &[&'static str],
    rarity_weight: f64,
    total_roster_weight: f64,
    model: ScoreModel,
) -> Vec<UpgradeDelta> {
    let current_dims = build_dimensions(roster, static_op, favor, is_support, model);
    if current_dims.iter().map(|(_, (w, _))| w).sum::<f64>() <= 0.0 {
        return Vec::new();
    }
    let current_score = average_dimensions(&current_dims);

    let mut out = Vec::with_capacity(missing.len());
    for &tag in missing {
        let new_score = simulate_score_for_tag(roster, static_op, favor, is_support, tag, model);
        let Some(new_score) = new_score else { continue };
        let op_delta = (new_score - current_score).max(0.0);
        let grade_delta = if total_roster_weight > 0.0 {
            op_delta * rarity_weight / total_roster_weight
        } else {
            0.0
        };
        let total_delta = grade_delta * SECTION_WEIGHT_OPERATOR / SECTION_WEIGHT_TOTAL;
        out.push(UpgradeDelta {
            tag,
            operator_score_delta: op_delta,
            operator_grade_delta: grade_delta,
            total_score_delta: total_delta,
        });
    }
    out
}

fn simulate_score_for_tag(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    tag: &str,
    model: ScoreModel,
) -> Option<f64> {
    fn set(dims: &mut [(DimensionKind, Dimension)], kind: DimensionKind, score: f64) -> bool {
        match dims.iter_mut().find(|(k, _)| *k == kind) {
            Some((_, dim)) => {
                dim.1 = score;
                true
            }
            None => false,
        }
    }

    let mut dims = build_dimensions(roster, static_op, favor, is_support, model);
    let applied = match tag {
        "ELITE" => {
            set(&mut dims, DimensionKind::Elite, 1.0) && set(&mut dims, DimensionKind::Level, 1.0)
        }
        "MAX_LEVEL" => {
            let max_lvl_here = static_op
                .phases
                .get(roster.elite as usize)
                .map_or(0, |p| p.max_level as i16);
            set(
                &mut dims,
                DimensionKind::Level,
                cumulative_level_progress_at(static_op, roster.elite, max_lvl_here),
            )
        }
        "M3" => {
            let num_skills = static_op.skills.len();
            let simulated = promote_to_milestone(&mastery_levels(&roster.masteries), num_skills);
            set(
                &mut dims,
                DimensionKind::Mastery,
                milestone_score(&simulated, num_skills, mastery_ladder),
            )
        }
        "SL7" => set(&mut dims, DimensionKind::SkillLevel, 1.0),
        "MOD3" => {
            let advanced_mods = advanced_modules(static_op);
            let user_advanced = advanced_module_levels(&roster.modules, &advanced_mods);
            let simulated = promote_to_milestone(&user_advanced, advanced_mods.len());
            set(
                &mut dims,
                DimensionKind::Module,
                milestone_score(&simulated, advanced_mods.len(), module_ladder(model)),
            )
        }
        "POT6" => set(&mut dims, DimensionKind::Potential, 1.0),
        "TRUST" => set(&mut dims, DimensionKind::Trust, 1.0),
        _ => false,
    };
    if !applied {
        return None;
    }
    Some(average_dimensions(&dims))
}

fn promote_to_milestone(levels: &[i16], slots: usize) -> Vec<i16> {
    let mut padded: Vec<i16> = levels.to_vec();
    padded.resize(padded.len().max(slots), 0);
    let pick = padded
        .iter()
        .enumerate()
        .filter(|&(_, &v)| v < MILESTONE)
        .max_by_key(|&(_, &v)| v)
        .map(|(i, _)| i);
    if let Some(idx) = pick {
        padded[idx] = MILESTONE;
    }
    padded
}
