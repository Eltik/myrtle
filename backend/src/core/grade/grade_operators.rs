use std::{
    collections::{HashMap, HashSet},
    sync::OnceLock,
};

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{
    core::gamedata::types::{
        GameData,
        module::ModuleType,
        operator::{Operator, OperatorModule, OperatorProfession, OperatorRarity},
        trust::Favor,
    },
    database::models::roster::RosterEntry,
};

use super::{
    Dimension,
    calculate::{SECTION_WEIGHT_OPERATOR, SECTION_WEIGHT_TOTAL},
    weighted_average,
};

/// Dimension weights.
///
/// Elite outweighs level by material value: each promotion gates skills,
/// masteries and modules behind a spike of rare materials plus the LMD fee,
/// making it the most expensive *discrete* milestone in an operator's build.
/// Levels rank second (leveling EXP + LMD is the largest *continuous* sanity
/// sink of a full build - see `level_weight`), so the promotion track
/// (elite + level) leads the mastery/module track while potentials stay
/// scored as their own dimension.
const WEIGHT_ELITE: f64 = 35.0;
const WEIGHT_MASTERY: f64 = 30.0;
/// Per advanced module SLOT since 2026-09-22 (see `module_weight`). Every
/// module of a rarity costs the same (4★: 3 blocks / 15 sticks / 5
/// instruments / 75k LMD / 15 T3; 6★: 12 / 60 / 20 / 300k / 9 T5), but with
/// one weight per operator a 6★ with two slots (71 of 127) earned half the
/// dimension per Mod3 that a one-slot 6★ (42) earned for the same materials.
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
pub const TRUST_MILESTONE_PCT: f64 = 100.0;

/// The level at which a skill (M3) or module (Mod3) counts as a milestone.
const MILESTONE: i16 = 3;

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
    #[serde(default)]
    locked: bool,
}

/// Mastery level per skill as stored on the roster entry.
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

/// Levels of the user's unlocked modules restricted to the operator's advanced
/// module slots - the levels that milestone scoring counts.
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

/// The switches that shape the Operators subscore. Each one is a kill switch
/// for a 2026-09-22 change, read from the environment once per process, and
/// flipping it restores the previous numbers exactly:
///
/// - `GRADE_INVESTED_ONLY=1`: average only raised operators (elite > 0 or
///   level > 1). That average let a roster of 75 owned and 2 raised read
///   60.8% on Modules, and the median account had raised only 43.7% of what
///   it owned, so an unraised pull cost nothing.
/// - `GRADE_RARITY_WEIGHTS=legacy`: rarity weights 1/.7/.4/.15/.1/.05 instead
///   of the cost-based 1/.5/.3/.05/.02/.01 (see `rarity_to_weight_in`).
/// - `GRADE_MODULE_PER_SLOT=0`: one `WEIGHT_MODULE` per operator instead of
///   one per advanced module slot (see `module_weight`).
///
/// Priced on 2,630 local accounts (all-owned denominator): the rarity weights
/// alone move the median subscore 0.2164 -> 0.2332 (1,963 up, 400 down by
/// more than 0.5 pt); per-slot modules alone 0.2164 -> 0.2111 (1,185 down);
/// both 0.2164 -> 0.2259 (1,566 up, 465 down). Production needs
/// `regrade-users` after a deploy that flips any of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScoreModel {
    pub invested_only: bool,
    pub legacy_rarity_weights: bool,
    pub module_per_slot: bool,
}

impl ScoreModel {
    /// The model in production since 2026-09-22.
    pub const SHIPPED: Self = Self {
        invested_only: false,
        legacy_rarity_weights: false,
        module_per_slot: true,
    };

    /// `SHIPPED` with each switch overridden from the environment.
    pub fn from_env() -> Self {
        static MODEL: OnceLock<ScoreModel> = OnceLock::new();
        *MODEL.get_or_init(|| {
            let var = |k: &str| std::env::var(k).ok();
            Self {
                invested_only: var("GRADE_INVESTED_ONLY").is_some_and(|v| v == "1"),
                legacy_rarity_weights: var("GRADE_RARITY_WEIGHTS").is_some_and(|v| v == "legacy"),
                module_per_slot: var("GRADE_MODULE_PER_SLOT").is_none_or(|v| v != "0"),
            }
        })
    }
}

/// `GRADE_INVESTED_ONLY` as read by `ScoreModel::from_env`.
pub fn invested_only() -> bool {
    ScoreModel::from_env().invested_only
}

/// Whether a roster entry is inside the Operators average: every owned entry
/// by default, only raised ones under `invested_only`. The improvements builder
/// prices upgrade deltas only for entries this admits, because a delta on an
/// entry outside the average claims a gain the score cannot pay.
pub fn is_graded(entry: &RosterEntry) -> bool {
    !invested_only() || has_investment(entry)
}

/// The operators that count toward `operator_grade`: real, obtainable, owned
/// (static data plus a roster entry), raised or not unless `invested_only`.
/// Every "gradeable operator" set derives from here.
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

pub fn grade_operators(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
) -> f64 {
    grade_operators_in(roster, game_data, support_ids, ScoreModel::from_env())
}

/// `grade_operators` with the model chosen explicitly instead of read from
/// the environment, so every switch can be tested in one process.
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
    total_roster_weight_in(roster, game_data, ScoreModel::from_env())
}

/// `total_roster_weight` under an explicit model.
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

pub fn grade_operator(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
) -> f64 {
    grade_operator_in(
        roster,
        static_op,
        favor,
        is_support,
        &ScoreModel::from_env(),
    )
}

/// `grade_operator` under an explicit model.
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

/// The investment axes an operator is scored on. Which axes apply varies per
/// operator (no advanced modules -> no `Module` dimension, etc.).
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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

/// Display order for breakdowns: active investment first, passive accrual last.
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
        let module_score = module_milestone_score(roster, &advanced_modules);
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

    // Trust - only meaningful once the favor table is loaded.
    if favor.max_trust_pct() > 0.0 {
        let trust_score = trust_milestone_score(roster, favor, is_support);
        dimensions.push((DimensionKind::Trust, (WEIGHT_TRUST, trust_score)));
    }

    dimensions
}

/// One row of the roster-wide operator score breakdown: how much of the
/// Operators subscore a dimension is worth, and how much of that worth the
/// user has earned.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Debug, Clone, Serialize)]
pub struct ScoreDimension {
    pub kind: DimensionKind,
    /// Share of the Operators subscore this dimension carries across the
    /// graded roster (0.0-1.0; all shares sum to 1.0).
    pub weight_share: f64,
    /// Rarity-weighted completion of this dimension (0.0-1.0).
    pub completion: f64,
    /// `weight_share x completion` - contributions sum to `operator_grade`.
    pub contribution: f64,
}

/// Decompose the Operators subscore into per-dimension contributions.
///
/// `grade_operators` averages each operator's dimensions, then averages the
/// operators by rarity weight - both are linear, so the subscore splits
/// exactly: each operator's dimension contributes
/// `(rarity_w / total_rarity_w) * (dim_w / op_dim_w_total) * dim_score`.
/// Summing those per `DimensionKind` yields rows whose `contribution`s add up
/// to the subscore, making the headline number auditable.
pub fn operator_score_breakdown(
    roster: &[RosterEntry],
    game_data: &GameData,
    support_ids: &HashSet<&str>,
) -> Vec<ScoreDimension> {
    operator_score_breakdown_in(roster, game_data, support_ids, ScoreModel::from_env())
}

/// `operator_score_breakdown` with the model chosen explicitly; see
/// `grade_operators_in`.
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
            slot.1 += dim_share * score;
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

/// Level weight per rarity. By average sanity distribution, leveling is the
/// single largest continuous cost of a full build, so for 4★+ it weighs just
/// under the elite gate (`WEIGHT_ELITE`) and ahead of the per-skill
/// dimensions. For 3★ and below - no E2, no masteries, no modules - leveling
/// *is* nearly the whole investment, so it carries a dominant weight.
const fn level_weight(rarity: &OperatorRarity) -> f64 {
    match rarity {
        OperatorRarity::SixStar | OperatorRarity::FiveStar | OperatorRarity::FourStar => 25.0,
        _ => 40.0,
    }
}

/// 0.0-1.0 across every elite phase, log-compressed.
///
/// Example for a 6-star at E2 L60:
///   completed: E0 (50 levels) + E1 (80 levels) + 60 of E2
///   total:     50 + 80 + 90 = 220
///   raw ratio: (50 + 80 + 60) / 220 = 0.864
///   after log:  ~0.90
fn cumulative_level_progress(roster: &RosterEntry, static_op: &Operator) -> f64 {
    cumulative_level_progress_at(static_op, roster.elite, roster.level)
}

/// `cumulative_level_progress` for an overridden (elite, level) pair, so the
/// delta simulator can score a promotion without mutating the `RosterEntry`.
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

/// Mastery dimension, 0.0-1.0 (`milestone_score` on the `mastery_ladder`).
fn mastery_milestone_score(roster: &RosterEntry, num_skills: usize) -> f64 {
    milestone_score(
        &mastery_levels(&roster.masteries),
        num_skills,
        mastery_ladder,
    )
}

/// Module dimension, 0.0-1.0 (`milestone_score` on the `module_ladder`).
fn module_milestone_score(roster: &RosterEntry, advanced_modules: &[&OperatorModule]) -> f64 {
    let levels = advanced_module_levels(&roster.modules, advanced_modules);
    milestone_score(&levels, advanced_modules.len(), module_ladder)
}

/// The milestone curve shared by masteries and modules. Before the first
/// milestone, partial credit is the fraction of the ladder climbed, capped at
/// `PARTIAL_CAP`. From the first milestone on, the dimension's `ladder` sets
/// the base for `reached` of `slots` and the slots still below it add up to
/// `PARTIAL_BONUS` on top.
fn milestone_score(levels: &[i16], slots: usize, ladder: fn(usize, usize) -> f64) -> f64 {
    let reached = levels.iter().filter(|&&l| l >= MILESTONE).count();
    if reached == 0 {
        return sub_milestone_progress(levels, slots) * PARTIAL_CAP;
    }
    let remaining = slots.saturating_sub(reached);
    (ladder(reached, slots) + sub_milestone_progress(levels, remaining) * PARTIAL_BONUS).min(1.0)
}

/// Fraction of the ladder climbed by the entries still below `MILESTONE`,
/// over `slots` slots of `MILESTONE` steps each; 0.0 with no slots.
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

/// One M3 -> 0.50, two -> 0.75, every skill the operator has -> 1.00. Keyed
/// on the operator's own skill count because most 4★/5★ only ever get two
/// skills, and a raw count would cap a fully-mastered one at 0.75 with
/// nothing left to buy.
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

/// The share of advanced modules at Mod3, never below 0.50 for the first.
fn module_ladder(reached: usize, slots: usize) -> f64 {
    (reached as f64 / slots as f64).max(0.50)
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

pub fn advanced_modules(static_op: &Operator) -> Vec<&OperatorModule> {
    static_op
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect()
}

/// Module dimension weight: `WEIGHT_MODULE` per advanced module slot, or per
/// operator under `GRADE_MODULE_PER_SLOT=0`. Per slot, a 6★ Mod3 is worth
/// 25 of a two-slot operator's 145 weight points against 25 of a one-slot
/// operator's 120 (0.172 vs 0.208 of the operator score); per operator it
/// was 12.5 of 120 against 25 of 120, a 2x gap at identical cost.
fn module_weight(slots: usize, model: ScoreModel) -> f64 {
    if model.module_per_slot {
        WEIGHT_MODULE * slots as f64
    } else {
        WEIGHT_MODULE
    }
}

/// How much a fully built operator of each rarity weighs in the roster
/// average, read from `GRADE_RARITY_WEIGHTS`; see `rarity_to_weight_in`.
pub fn rarity_to_weight(rarity: &OperatorRarity) -> f64 {
    rarity_to_weight_in(rarity, ScoreModel::from_env().legacy_rarity_weights)
}

/// Rarity weights follow the cost of a full build relative to a 6★, from a
/// 2026-09-22 census of `character_table` / `uniequip_table` /
/// `gamedata_const` (127 6★, 188 5★, 61 4★, 17 3★). The whole-build ratio
/// under three valuations (tokens priced / tokens at zero / LMD only) is
/// 5★ 0.44 / 0.54 / 0.53, 4★ 0.26 / 0.31 / 0.31, 3★ 0.03 / 0.04 / 0.06; the
/// shipped constants sit between them. This is a TRADE: per dimension the
/// 4★/6★ ratio spreads from 0.25 (a module) to 0.47 (promotion), so 0.30 is
/// 20% generous on a 4★ module and 25% stingy on its levels. The legacy
/// 1/.7/.4/.15 overpriced 5★ and 4★ builds by about 1.5x and a 3★ by 2.5x to
/// 5x. 2★ and 1★ are not censused (one 30-level phase, 3.7% of roster weight
/// under the legacy values, 1.8% now).
pub const fn rarity_to_weight_in(rarity: &OperatorRarity, legacy: bool) -> f64 {
    let table = if legacy {
        &LEGACY_RARITY_WEIGHTS
    } else {
        &RARITY_WEIGHTS
    };
    table[(6 - rarity.to_star_int()) as usize]
}

/// Rarity weights, 6★ first down to 1★.
const RARITY_WEIGHTS: [f64; 6] = [1.0, 0.5, 0.3, 0.05, 0.02, 0.01];
/// The weights before 2026-09-22, same order (`GRADE_RARITY_WEIGHTS=legacy`).
const LEGACY_RARITY_WEIGHTS: [f64; 6] = [1.0, 0.7, 0.4, 0.15, 0.1, 0.05];

pub const fn potential_matters(static_op: &Operator) -> bool {
    !static_op.can_use_general_potential_item || static_op.is_sp_char
}

/// Potential dimension score (0.0-1.0). `potential` is 0-indexed (P1 = 0 …
/// P6 = 5), so a fully-potential'd operator maps to 1.0.
fn potential_score(potential: i16) -> f64 {
    (f64::from(potential) / 5.0).clamp(0.0, 1.0)
}

/// Returns true if the player has invested beyond the initial pull state (E0 L1).
/// Since 2026-09-22 this no longer gates the Operators average (see
/// `invested_only`); it still marks a raised operator elsewhere.
pub const fn has_investment(roster: &RosterEntry) -> bool {
    roster.elite > 0 || roster.level > 1
}

fn log_curve_ratio(t: f64) -> f64 {
    (1.0 + t).ln() / 2.0_f64.ln()
}

/// Score gain from a single upgrade path.
///
/// All deltas are reported as non-negative - if simulating the milestone would
/// somehow not improve the score (shouldn't happen with the current model,
/// but defensive), the delta is clamped to 0.
#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
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
    /// the Operators subscore's share of the overall grade
    /// (`SECTION_WEIGHT_OPERATOR / SECTION_WEIGHT_TOTAL`).
    pub total_score_delta: f64,
}

/// Compute per-tag deltas for the given operator. `missing` is the list of
/// upgrade tags surfaced in `OperatorGap.missing`; each gets a simulated
/// "what if you completed this milestone" score against the current state.
///
/// `rarity_weight` and `total_roster_weight` are the inputs from
/// `grade_operators` used to translate per-op delta -> `operator_grade` delta.
pub fn operator_upgrade_deltas(
    roster: &RosterEntry,
    static_op: &Operator,
    favor: &Favor,
    is_support: bool,
    missing: &[&'static str],
    rarity_weight: f64,
    total_roster_weight: f64,
) -> Vec<UpgradeDelta> {
    let model = ScoreModel::from_env();
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

/// Rebuild dimensions with one milestone reached, then average. Returns `None`
/// if the tag isn't applicable to this operator (defensive - caller already
/// filtered, but cheap to recheck).
///
/// A dimension exists on the operator exactly when the tag applies (no
/// advanced modules -> no `Module` dimension -> MOD3 not applicable), so
/// "replace by kind" doubles as the applicability check.
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
        // Full promotion path: jump to max elite + max level at that phase.
        "ELITE" => {
            set(&mut dims, DimensionKind::Elite, 1.0) && set(&mut dims, DimensionKind::Level, 1.0)
        }
        // Max level at current elite phase only.
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
        // First mastery to M3. We assume the highest currently non-M3 skill is
        // the one promoted, which is the most generous read of the user's
        // current trajectory.
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
        // First advanced module to L3 (same "promote highest non-Mod3" model
        // as M3 above).
        "MOD3" => {
            let advanced_mods = advanced_modules(static_op);
            let user_advanced = advanced_module_levels(&roster.modules, &advanced_mods);
            let simulated = promote_to_milestone(&user_advanced, advanced_mods.len());
            set(
                &mut dims,
                DimensionKind::Module,
                milestone_score(&simulated, advanced_mods.len(), module_ladder),
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

/// Promote the highest entry below `MILESTONE` up to it: "one more M3" /
/// "one more Mod3" on the slot the user is most likely to push next.
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

#[cfg(test)]
mod module_parse_tests {
    use super::parse_modules;

    #[test]
    fn locked_modules_are_ignored() {
        let json = serde_json::json!([
            { "id": "uniequip_002_a", "level": 1, "locked": true },
            { "id": "uniequip_003_a", "level": 2, "locked": false },
            { "id": "uniequip_001_a", "level": 1 }
        ]);
        let ids: Vec<String> = parse_modules(&json).into_iter().map(|m| m.id).collect();
        assert_eq!(ids, ["uniequip_003_a", "uniequip_001_a"]);
    }
}
