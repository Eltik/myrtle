use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock};

use crate::core::gamedata::types::operator::Operator;
use crate::dps::operator_data::OperatorData;
use crate::dps::operator_unit::{CycleModel, OperatorBaseBuffs, OperatorParams};
use crate::dps::zones::{
    BuffZones, DamageType, ResolvedReduction, max_source, post_multiplier, product, resolve_enemy,
};

use super::custom::apply_init;
use super::custom::dispatch;
use super::custom::dispatch_hps;
use super::formulas::apply_shreds;
use super::operator_unit::{EnemyStats, OperatorUnit};
use ts_rs::TS;
use utoipa::ToSchema;

const FORMULAS_JSON: &str = include_str!("config/operator_formulas.json");
const HEAL_FORMULAS_JSON: &str = include_str!("config/heal_formulas.json");
/// Derived by `scripts/dps_conditional_polarity.py` from the Python reference,
/// kept apart from the generated tables (see the script).
const POLARITY_JSON: &str = include_str!("config/conditional_polarity.json");

/// `dps` / `hps` -> operator id -> `"<type>|<name>"` -> inverted (`null` =
/// unresolved). Two sections because an operator can be in both tables.
type PolarityTable = HashMap<String, HashMap<String, HashMap<String, Option<bool>>>>;
static POLARITY: LazyLock<PolarityTable> = LazyLock::new(|| {
    serde_json::from_str(POLARITY_JSON).expect("Invalid conditional_polarity.json")
});

/// Whether a conditional's label describes its flag OFF (the Python reference
/// appends it under `if not self.<flag>`). `table` is `dps` or `hps`. `false`
/// when the polarity is ON or unresolved (one conditional on 2026-10-24,
/// Diamante `avgNecrosis`).
pub fn conditional_inverted(table: &str, op_id: &str, cond_type: &str, name: &str) -> bool {
    POLARITY
        .get(table)
        .and_then(|t| t.get(op_id))
        .and_then(|m| m.get(&format!("{cond_type}|{name}")))
        .copied()
        .flatten()
        .unwrap_or(false)
}

/// The polarity table as derived: (on, inverted, unresolved) counts.
pub fn polarity_counts() -> (usize, usize, usize) {
    let mut counts = (0, 0, 0);
    for v in POLARITY
        .values()
        .flat_map(HashMap::values)
        .flat_map(HashMap::values)
    {
        match v {
            Some(false) => counts.0 += 1,
            Some(true) => counts.1 += 1,
            None => counts.2 += 1,
        }
    }
    counts
}

pub fn load_formulas() -> HashMap<String, OperatorFormula> {
    let mut formulas: HashMap<String, OperatorFormula> =
        serde_json::from_str(FORMULAS_JSON).expect("Invalid operator_formulas.json");
    for formula in formulas.values_mut() {
        normalize_default_skill(formula);
    }
    formulas
}

/// Makes `default_skill` one the table can actually compute.
///
/// The client opens an operator on `default_skill`, so a default with no formula
/// shows no DPS until the user picks another skill. Two shapes reach that today:
/// a default outside `available_skills` (`char_1044_hsgma2` defaults to S2,
/// but only S1 and S3 are transpiled), which moves to the highest available
/// skill; and a table with no skills at all (1 and 2 stars, and operators whose
/// skills were not transpiled), which moves to 0, basic attack, the index the
/// engine already forces for rarity 2 and below. DPS tables only: `calculate_hps`
/// never reads the skill map, so the healer defaults are left as shipped.
fn normalize_default_skill(formula: &mut OperatorFormula) {
    match formula.available_skills.iter().max() {
        None => formula.default_skill = 0,
        Some(&highest) if !formula.available_skills.contains(&formula.default_skill) => {
            formula.default_skill = highest;
        }
        Some(_) => {}
    }
}

/// The skill entry for basic attack (`skill_index` 0), which no table carries.
/// Every entry is the `custom` kind and `calculate_skill_dps` dispatches on the
/// operator id, so a default entry computes exactly what the operator's function
/// does with no skill active.
static BASIC_ATTACK: LazyLock<SkillFormula> = LazyLock::new(SkillFormula::default);

pub fn load_heal_formulas() -> HashMap<String, OperatorFormula> {
    serde_json::from_str(HEAL_FORMULAS_JSON).expect("Invalid heal_formulas.json")
}

/// One operator configuration's damage.
///
/// The three snake_case fields are the original response and keep their names
/// and meaning. Every field after them is optional, camelCase, and present
/// only when the request asked for the behaviour that produces it.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct DpsResult {
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
    /// The primary enemy's share. Present when the request set `secondary`.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<DpsMetrics>,
    /// The secondary enemies' share, `f(1 + count) - f(1)` against the
    /// secondary enemy. Present when the request set `secondary`; the three
    /// top-level fields are then `primary + secondary`.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<DpsMetrics>,
    /// Skill shape. Present under `cycleModel: "v2"`.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<SkillCategory>,
    /// Seconds from deployment to the first activation: `(cost - initial SP)`
    /// at the skill's SP rate. Under `cycleModel: "v2"`, for a skill whose SP
    /// type the engine models.
    #[ts(optional)]
    #[serde(
        rename = "startupTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub startup_time: Option<f64>,
    /// Seconds per skill cycle (duration + charge). Under `cycleModel: "v2"`,
    /// for a modelled SP type and a skill that cycles.
    #[ts(optional)]
    #[serde(rename = "cycleTime", default, skip_serializing_if = "Option::is_none")]
    pub cycle_time: Option<f64>,
    /// The skill's SP type as the cycle saw it. Under `cycleModel: "v2"`.
    #[ts(optional)]
    #[serde(
        rename = "spRecovery",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sp_recovery: Option<SpRecovery>,
    /// Where `average_dps` took the off-skill DPS from. Under `cycleModel: "v2"`
    /// for a skill that cycles.
    #[ts(optional)]
    #[serde(
        rename = "offSkillSource",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub off_skill_source: Option<OffSkillSource>,
    /// `true` when a physical or arts fragility zone was requested and the
    /// formula's damage responds to both DEF and RES (or to neither), so only
    /// the generic group and the operator's `is_physical` group were applied.
    /// Present only when a typed fragility zone was requested.
    #[ts(optional)]
    #[serde(
        rename = "approximateTypeSplit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub approximate_type_split: Option<bool>,
    /// `true` when `f(1 + count) - f(1)` came out negative on a metric and the
    /// secondary share was clamped to 0 there. Present only when it fired.
    #[ts(optional)]
    #[serde(
        rename = "secondaryClamped",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_clamped: Option<bool>,
}

/// The three damage numbers for one share of the targets. Same snake_case names
/// as the top level of [`DpsResult`], so one client type reads both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct DpsMetrics {
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
}

impl DpsMetrics {
    fn plus(self, other: Self) -> Self {
        Self {
            skill_dps: self.skill_dps + other.skill_dps,
            total_damage: self.total_damage + other.total_damage,
            average_dps: self.average_dps + other.average_dps,
        }
    }

    /// Each metric floored at 0.
    fn clamped(self) -> Self {
        Self {
            skill_dps: self.skill_dps.max(0.0),
            total_damage: self.total_damage.max(0.0),
            average_dps: self.average_dps.max(0.0),
        }
    }

    fn minus(self, other: Self) -> Self {
        Self {
            skill_dps: self.skill_dps - other.skill_dps,
            total_damage: self.total_damage - other.total_damage,
            average_dps: self.average_dps - other.average_dps,
        }
    }
}

/// `sustained`: the skill's DPS is the DPS (no skill, passive, or permanent).
/// `burst`: a timed or ammo window that recharges. `instant`: a zero-duration
/// activation; the formulas for these already average the hit over its charge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SkillCategory {
    Sustained,
    Burst,
    Instant,
}

/// The skill's SP type from game data. `defensive` (SP on taking a hit) and
/// `none` (passive) are NOT modelled: their `average_dps` is the legacy one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum SpRecovery {
    Auto,
    Offensive,
    Defensive,
    None,
}

/// `formula`: the operator's own function at skill 0. `normalAttack`: the
/// generic `atk / interval` attack, used when the function has no skill-0 path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum OffSkillSource {
    Formula,
    NormalAttack,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct HpsResult {
    /// Heal-per-second while the active skill is up.
    pub skill_hps: f64,
    /// Heal-per-second during the SP-charge phase (0 for burst healers).
    pub base_hps: f64,
    /// Cycle-averaged HPS including charge time.
    pub avg_hps: f64,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SkillFormula {
    #[serde(rename = "type")]
    pub formula_type: String,
    // Indices into skill_parameters/talent_parameters for param resolution
    pub atk_scale_idx: Option<usize>, // skill_params[idx] for ATK scale
    pub hits_idx: Option<usize>,      // skill_params[idx] for hit count
    pub hits: Option<f64>,            // literal hit count
    pub targets: Option<f64>,         // literal target count
    pub aspd_talent_idx: Option<usize>, // talent1_params[idx] for ASPD bonus
    pub def_ignore_idx: Option<usize>, // talent1_params[idx] for DEF ignore
    pub res_ignore_idx: Option<usize>, // talent1_params[idx] for RES ignore
    // Extended fields for generic_dps
    pub scale_on_skill_only: Option<bool>, // only apply atk_scale when skill is active
    pub cycle_average: Option<bool>,       // average skill + basic attack over SP cost
    pub talent_atk_idx: Option<usize>,     // talent1_params[idx] for ATK buff
    pub module_atk_scale: Option<f64>,     // module multiplier on final ATK (e.g. 1.1)
    pub override_interval: Option<f64>,    // skill overrides attack interval
    pub damage_type: Option<String>,       // "arts" or "physical" to override is_physical
}

impl Default for SkillFormula {
    fn default() -> Self {
        Self {
            formula_type: "custom".to_string(),
            atk_scale_idx: None,
            hits_idx: None,
            hits: None,
            targets: None,
            aspd_talent_idx: None,
            def_ignore_idx: None,
            res_ignore_idx: None,
            scale_on_skill_only: None,
            cycle_average: None,
            talent_atk_idx: None,
            module_atk_scale: None,
            override_interval: None,
            damage_type: None,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OperatorFormula {
    pub name: String,
    pub class_name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    pub default_skill: i32,
    pub default_potential: i32,
    pub default_module: i32,
    pub skills: HashMap<String, SkillFormula>,
    pub conditionals: Vec<ConditionalConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ConditionalConfig {
    #[serde(rename = "type")]
    pub cond_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
}

pub fn calculate_skill_dps(
    unit: &OperatorUnit,
    _formula: &SkillFormula,
    enemy: &EnemyStats,
) -> f64 {
    dispatch(unit, enemy).unwrap_or(0.0)
}

static FORMULAS: LazyLock<HashMap<String, OperatorFormula>> = LazyLock::new(load_formulas);
static HEAL_FORMULAS: LazyLock<HashMap<String, OperatorFormula>> =
    LazyLock::new(load_heal_formulas);

pub fn get_formula(op_id: &str) -> Option<&OperatorFormula> {
    FORMULAS.get(op_id)
}

pub fn supported_operators() -> &'static HashMap<String, OperatorFormula> {
    &FORMULAS
}

pub fn get_heal_formula(op_id: &str) -> Option<&OperatorFormula> {
    HEAL_FORMULAS.get(op_id)
}

pub fn supported_healers() -> &'static HashMap<String, OperatorFormula> {
    &HEAL_FORMULAS
}

pub fn calculate_dps(
    operator: &Operator,
    params: OperatorParams,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    calculate_dps_with(Arc::new(OperatorData::new(operator.clone())), params, enemy)
}

/// The defaults the formula table gives an operator, or the basic-attack-only
/// defaults for an operator with no table (a clone source may have none).
fn formula_defaults(op_id: &str) -> (i32, i32, i32, Vec<i32>, Vec<i32>) {
    FORMULAS
        .get(op_id)
        .map_or((0, 1, 0, Vec::new(), Vec::new()), |f| {
            (
                f.default_skill,
                f.default_potential,
                f.default_module,
                f.available_skills.clone(),
                f.available_modules.clone(),
            )
        })
}

/// The investment half of `params`, for building the operator a Muelsyse
/// clone copies. damage_formulas.py `Muelsyse.__init__` builds it as
/// `Ela(pp)`: the same parameter object, so the same promotion, level,
/// potential, trust, skill and module choices.
fn investment_params(p: &OperatorParams, module_index: Option<i32>) -> OperatorParams {
    OperatorParams {
        potential: p.potential,
        promotion: p.promotion,
        level: p.level,
        trust: p.trust,
        skill_index: p.skill_index,
        mastery_level: p.mastery_level,
        skill_level: p.skill_level,
        module_index,
        module_level: p.module_level,
        base_buffs: OperatorBaseBuffs {
            atk: p.base_buffs.atk,
            flat_atk: p.base_buffs.flat_atk,
        },
        ..Default::default()
    }
}

/// Fills the `clone_*` fields from the copied operator, built at the same
/// investment. A module the copied operator cannot resolve falls back to the
/// copied operator's DEFAULT module, as damage_formulas.py `Operator.__init__`
/// does (`module = default_mod`, kept when `params.module` is not available).
/// Until the phase 1 review this fell back to no module, so every Muelsyse
/// request with module 2 built an Ela without her module (docs/dps-refactor.md,
/// Phase 2, "Phase 1 review fixes").
fn apply_clone(unit: &mut OperatorUnit, clone: Arc<OperatorData>) {
    let clone_id = clone.data.id.clone().unwrap_or_default();
    let (ds, dp, dm, skills, modules) = formula_defaults(&clone_id);
    let build = |module_index| {
        OperatorUnit::new(
            Arc::clone(&clone),
            investment_params(&unit.params, module_index),
            ds,
            dp,
            dm,
            skills.clone(),
            modules.clone(),
        )
    };
    let mut copied = build(unit.params.module_index);
    if copied.module_unavailable {
        copied = build(Some(dm));
    }
    unit.clone_atk = copied.atk;
    unit.clone_atk_interval = f64::from(copied.attack_interval);
    unit.clone_is_ranged = copied.is_ranged;
    unit.clone_is_physical = copied.is_physical;
}

/// How the request's DEF / RES reach the formula.
enum EnemyResolution {
    /// `apply_shreds` with the unit's legacy `shreds`: the engine before zones.
    Legacy(Vec<f64>),
    Zones(ResolvedReduction, ResolvedReduction),
}

impl EnemyResolution {
    fn resolve(&self, enemy: &EnemyStats) -> EnemyStats {
        match self {
            Self::Legacy(shreds) => apply_shreds(enemy, shreds),
            Self::Zones(def, res) => resolve_enemy(enemy, def, res),
        }
    }
}

/// Folds the ATK, ASPD and SP zones into the unit fields the formulas read,
/// and returns the DEF / RES resolution. Order: A, B, C, D (D scales A and C).
fn apply_zones(unit: &mut OperatorUnit, zones: &BuffZones) -> EnemyResolution {
    if let Some(a) = zones.atk_flat {
        unit.atk += a;
    }
    if let Some(b) = zones.atk_percent {
        unit.buff_atk += b;
    }
    if !zones.inspire.is_empty() {
        let mut sources = zones.inspire.clone();
        // A legacy `buffs.flatAtk` is one more Inspire source.
        if unit.params.buffs.flat_atk.is_some() {
            sources.push(unit.buff_atk_flat);
        }
        unit.buff_atk_flat = max_source(&sources).unwrap_or(0.0);
    }
    if !zones.atk_multiplier.is_empty() {
        let d = product(&zones.atk_multiplier);
        unit.atk *= d;
        unit.buff_atk_flat *= d;
        unit.drone_atk *= d;
        unit.clone_atk *= d;
    }
    if let Some(aspd) = zones.aspd {
        unit.attack_speed += aspd;
    }
    let mut sp = f64::from(unit.sp_boost);
    if let Some(r) = zones.sp_recovery {
        sp += r;
    }
    if let Some(aura) = max_source(&zones.sp_aura) {
        sp += aura;
    }
    if zones.sp_recovery.is_some() || !zones.sp_aura.is_empty() {
        unit.sp_boost = sp as f32;
    }

    let def = ResolvedReduction::new(&zones.def, unit.shreds[0], unit.shreds[1]);
    let res = ResolvedReduction::new(&zones.res, unit.shreds[2], unit.shreds[3]);
    // The 24 functions that read `shreds` (to undo and re-apply a percentage
    // around their own) see the merged flat and percentage. They do not see
    // the ignore steps: a TRADE, the shreds slots have no place for them.
    unit.shreds = vec![def.ratio, def.flat, res.ratio, res.flat];
    EnemyResolution::Zones(def, res)
}

/// Which typed fragility group the formula's damage belongs to, from how it
/// responds to DEF and to RES, plus whether that was an approximation.
fn damage_type(unit: &mut OperatorUnit, formula: &SkillFormula) -> (DamageType, bool) {
    let at = |unit: &OperatorUnit, defense, res| {
        calculate_skill_dps(unit, formula, &EnemyStats { defense, res })
    };
    let base = at(unit, 0.0, 0.0);
    let by_def = at(unit, 2000.0, 0.0) != base;
    let by_res = at(unit, 0.0, 90.0) != base;
    let fallback = if unit.is_physical {
        DamageType::Physical
    } else {
        DamageType::Arts
    };
    match (by_def, by_res) {
        (true, false) => (DamageType::Physical, false),
        (false, true) => (DamageType::Arts, false),
        _ => (fallback, true),
    }
}

/// The operator's own formula at skill 0, when it has a skill-0 path.
///
/// The reference's `avg_dps` calls `skill_dps` with `self.skill = 0` and every
/// other field (the active skill's parameters included) unchanged, and so does
/// this. A function whose skill-0 branch is real does not read the active
/// skill's parameters, duration or cost, so the probe evaluates it twice, the
/// second time with all three perturbed, and trusts it only when the two
/// agree bit for bit. Otherwise (a skill-0 call that lands in another skill's
/// branch, or a non-finite result) the caller falls back to `normal_attack`.
pub fn skill_zero_dps(
    unit: &mut OperatorUnit,
    formula: &SkillFormula,
    enemy: &EnemyStats,
) -> Option<f64> {
    let skill = unit.skill_index;
    unit.skill_index = 0;
    let real = calculate_skill_dps(unit, formula, enemy);

    let params = unit.skill_parameters.clone();
    let duration = unit.skill_duration;
    let cost = unit.skill_cost;
    unit.skill_parameters = params.iter().map(|p| p * 2.0 + 1.0).collect();
    unit.skill_duration = duration * 2.0 + 1.0;
    unit.skill_cost = cost * 2 + 1;
    let perturbed = calculate_skill_dps(unit, formula, enemy);

    unit.skill_parameters = params;
    unit.skill_duration = duration;
    unit.skill_cost = cost;
    unit.skill_index = skill;

    (real.is_finite() && real.to_bits() == perturbed.to_bits()).then_some(real)
}

/// SP behaviour of the unit's active skill at its level, from game data.
struct SkillSp {
    recovery: SpRecovery,
    init_sp: i32,
    ammo: bool,
}

fn skill_sp(unit: &OperatorUnit) -> SkillSp {
    let level = usize::try_from(unit.skill_index - 1)
        .ok()
        .and_then(|i| unit.data.data.skills.get(i))
        .and_then(|s| s.static_data.as_ref())
        .and_then(|s| {
            usize::try_from(unit.skill_level - 1)
                .ok()
                .and_then(|l| s.levels.get(l))
        });
    let Some(level) = level else {
        return SkillSp {
            recovery: SpRecovery::None,
            init_sp: 0,
            ammo: false,
        };
    };
    let recovery = match level.sp_data.sp_type.as_str() {
        "INCREASE_WITH_TIME" => SpRecovery::Auto,
        "INCREASE_WHEN_ATTACK" => SpRecovery::Offensive,
        "INCREASE_WHEN_TAKEN_DAMAGE" => SpRecovery::Defensive,
        _ => SpRecovery::None,
    };
    SkillSp {
        recovery,
        init_sp: level.sp_data.init_sp,
        ammo: level.duration_type.as_deref() == Some("AMMO"),
    }
}

/// Cycle facts reported under `cycleModel: "v2"`.
#[derive(Clone, Copy)]
struct CycleInfo {
    category: SkillCategory,
    startup_time: Option<f64>,
    cycle_time: Option<f64>,
    sp_recovery: SpRecovery,
    off_skill_source: Option<OffSkillSource>,
}

/// Everything one evaluation needs besides the unit and the enemy.
struct EvalContext<'a> {
    formula: &'a SkillFormula,
    cycle: CycleModel,
    /// Multiplier on the on-skill DPS (zone F and G, or legacy `1 + fragile`).
    on_mult: f64,
    /// Multiplier on the off-skill DPS. Legacy: `1 + unit.buff_fragile`, which
    /// is `1 + 0`: the engine never applied external fragile off-skill.
    /// V2: the same as `on_mult`.
    off_mult: f64,
}

/// One evaluation against one (already resolved) enemy at the unit's current
/// `targets`.
fn evaluate(
    unit: &mut OperatorUnit,
    ctx: &EvalContext<'_>,
    enemy: &EnemyStats,
) -> (DpsMetrics, Option<CycleInfo>) {
    let skill_dps = calculate_skill_dps(unit, ctx.formula, enemy) * ctx.on_mult;

    let total_damage = if unit.skill_duration > 0.0 {
        skill_dps * unit.skill_duration
    } else {
        skill_dps
    };

    let legacy_average = |unit: &OperatorUnit| {
        if unit.skill_duration > 0.0 && unit.skill_cost > 0 {
            let off_skill_dps =
                unit.normal_attack(enemy, None, None, None) * (1.0 + unit.buff_fragile);
            let sp_time = f64::from(unit.skill_cost) / (1.0 + f64::from(unit.sp_boost));
            let cycle_dmg = skill_dps * unit.skill_duration + off_skill_dps * sp_time;
            cycle_dmg / (unit.skill_duration + sp_time)
        } else {
            skill_dps
        }
    };

    if ctx.cycle == CycleModel::Legacy {
        let average_dps = legacy_average(unit);
        return (
            DpsMetrics {
                skill_dps,
                total_damage,
                average_dps,
            },
            None,
        );
    }

    let sp = skill_sp(unit);
    let duration = unit.skill_duration;
    let cost = f64::from(unit.skill_cost);

    // SP per second off-skill, for the SP types the engine models.
    let sp_rate = match sp.recovery {
        SpRecovery::Auto => Some(1.0 + f64::from(unit.sp_boost)),
        // One SP per attack: attacks per second at the off-skill attack speed.
        // SP auras recover SP over time and do not apply.
        SpRecovery::Offensive => Some(unit.attack_speed / 100.0 / f64::from(unit.attack_interval)),
        SpRecovery::Defensive | SpRecovery::None => None,
    }
    .filter(|r| *r > 0.0 && r.is_finite());

    let category = if unit.skill_index == 0 || unit.skill_cost <= 0 {
        SkillCategory::Sustained
    } else if sp.ammo || duration > 0.0 {
        SkillCategory::Burst
    } else if duration < 0.0 {
        SkillCategory::Sustained
    } else {
        SkillCategory::Instant
    };

    let startup_time = if unit.skill_index == 0 {
        Some(0.0)
    } else {
        sp_rate
            .filter(|_| unit.skill_cost > 0)
            .map(|r| (cost - f64::from(sp.init_sp)).max(0.0) / r)
    };

    let sp_time = sp_rate.filter(|_| unit.skill_cost > 0).map(|r| cost / r);
    let cycle_time = match category {
        SkillCategory::Burst if duration > 0.0 => sp_time.map(|t| duration + t),
        SkillCategory::Instant => sp_time,
        _ => None,
    };

    let mut off_skill_source = None;
    // Ammo skills cycle too: damage_formulas.py `avg_dps` cycles any skill
    // with `skill_duration >= 1` whatever its duration type, and so does the
    // legacy path. Until the phase 1 review v2 skipped them (`!sp.ammo`) while
    // still reporting a Burst cycle time for them.
    let average_dps = if unit.skill_index > 0 && duration > 0.0 && unit.skill_cost > 0 {
        match sp_time {
            Some(sp_time) => {
                let (off, source) = match skill_zero_dps(unit, ctx.formula, enemy) {
                    Some(v) => (v, OffSkillSource::Formula),
                    None => (
                        unit.normal_attack(enemy, None, None, None),
                        OffSkillSource::NormalAttack,
                    ),
                };
                off_skill_source = Some(source);
                let off = off * ctx.off_mult;
                (skill_dps * duration + off * sp_time) / (duration + sp_time)
            }
            // Defensive or unrecognised SP: not modelled, the legacy average.
            None => legacy_average(unit),
        }
    } else {
        skill_dps
    };

    (
        DpsMetrics {
            skill_dps,
            total_damage,
            average_dps,
        },
        Some(CycleInfo {
            category,
            startup_time,
            cycle_time,
            sp_recovery: sp.recovery,
            off_skill_source,
        }),
    )
}

/// `calculate_dps` on a prepared (usually cached) `OperatorData`.
///
/// With only the legacy fields set (no `zones`, no `secondary`, `cycle_model`
/// Legacy, no `clone_operator`) this is the engine as it was before phase 1,
/// bit for bit: the kill-switch rule in docs/dps-refactor.md.
pub fn calculate_dps_with(
    data: Arc<OperatorData>,
    params: OperatorParams,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    calculate_dps_detailed(data, params, enemy).map(|(result, _)| result)
}

/// [`calculate_dps_with`] plus the skill duration the formula ran with
/// (`unit.skill_duration`, seconds; 0 or negative for a skill with none), which
/// the ranking reports. Same computation, so the result is the same bits.
pub fn calculate_dps_detailed(
    data: Arc<OperatorData>,
    mut params: OperatorParams,
    enemy: &EnemyStats,
) -> Option<(DpsResult, f64)> {
    let formula = FORMULAS.get(data.data.id.as_deref()?)?;

    let zones = params.zones.take();
    let secondary = params.secondary.take();
    let cycle = params.cycle_model;
    let clone = params.clone_operator.take();

    let mut unit = OperatorUnit::new(
        data,
        params,
        formula.default_skill,
        formula.default_potential,
        formula.default_module,
        formula.available_skills.clone(),
        formula.available_modules.clone(),
    );

    if unit.module_unavailable {
        return None;
    }

    if let Some(clone) = clone {
        apply_clone(&mut unit, clone);
    }

    apply_init(&mut unit);

    let skill_key = unit.skill_index.to_string();
    let skill_formula = match formula.skills.get(&skill_key) {
        Some(skill_formula) => skill_formula,
        None if unit.skill_index == 0 => &BASIC_ATTACK,
        None => return None,
    };

    let resolution = match &zones {
        None => EnemyResolution::Legacy(unit.shreds.clone()),
        Some(z) => apply_zones(&mut unit, z),
    };

    // buff_fragile is 0 during skill_dps in the Python reference: the operator never
    // sees external fragile, it is multiplied on after the call.
    let external_fragile = unit.buff_fragile;
    unit.buff_fragile = 0.0;

    let mut approximate_type_split = None;
    let on_mult = match &zones {
        None => 1.0 + external_fragile,
        Some(z) => {
            let legacy_fragile = unit.params.buffs.fragile.map(|_| external_fragile);
            let typed = if z.physical_fragile.is_empty() && z.arts_fragile.is_empty() {
                None
            } else {
                let (kind, approximate) = damage_type(&mut unit, skill_formula);
                approximate_type_split = Some(approximate);
                Some(kind)
            };
            post_multiplier(z, legacy_fragile, typed)
        }
    };
    let ctx = EvalContext {
        formula: skill_formula,
        cycle,
        on_mult,
        off_mult: match cycle {
            CycleModel::Legacy => 1.0,
            CycleModel::V2 => on_mult,
        },
    };

    let primary_enemy = resolution.resolve(enemy);

    let mut secondary_clamped = None;
    let (metrics, cycle_info, primary, secondary) = match secondary {
        None => {
            let (m, info) = evaluate(&mut unit, &ctx, &primary_enemy);
            (m, info, None, None)
        }
        Some(sec) => {
            let secondary_enemy = resolution.resolve(&EnemyStats {
                defense: sec.defense,
                res: sec.res,
            });
            unit.targets = 1;
            let (primary, info) = evaluate(&mut unit, &ctx, &primary_enemy);
            let (one, _) = evaluate(&mut unit, &ctx, &secondary_enemy);
            unit.targets = 1 + sec.count.max(1);
            let (all, _) = evaluate(&mut unit, &ctx, &secondary_enemy);
            // A formula that is not monotone in `targets` can make the
            // difference negative; it is clamped at 0 and flagged.
            let raw = all.minus(one);
            let secondary = raw.clamped();
            if secondary != raw {
                secondary_clamped = Some(true);
            }
            (
                primary.plus(secondary),
                info,
                Some(primary),
                Some(secondary),
            )
        }
    };

    let duration = unit.skill_duration;
    Some((
        DpsResult {
            skill_dps: metrics.skill_dps,
            total_damage: metrics.total_damage,
            average_dps: metrics.average_dps,
            primary,
            secondary,
            category: cycle_info.map(|c| c.category),
            startup_time: cycle_info.and_then(|c| c.startup_time),
            cycle_time: cycle_info.and_then(|c| c.cycle_time),
            sp_recovery: cycle_info.map(|c| c.sp_recovery),
            off_skill_source: cycle_info.and_then(|c| c.off_skill_source),
            approximate_type_split,
            secondary_clamped,
        },
        duration,
    ))
}

/// Returns `None` for operators without a transpiled HPS implementation.
pub fn calculate_hps(operator: &Operator, params: OperatorParams) -> Option<HpsResult> {
    calculate_hps_with(Arc::new(OperatorData::new(operator.clone())), params)
}

/// `calculate_hps` on a prepared (usually cached) `OperatorData`.
pub fn calculate_hps_with(data: Arc<OperatorData>, params: OperatorParams) -> Option<HpsResult> {
    let op_id = data.data.id.clone()?;
    let formula = HEAL_FORMULAS.get(&op_id)?;

    let mut unit = OperatorUnit::new(
        data,
        params,
        formula.default_skill,
        formula.default_potential,
        formula.default_module,
        formula.available_skills.clone(),
        formula.available_modules.clone(),
    );

    if unit.module_unavailable {
        return None;
    }

    apply_init_fixups(&mut unit, &op_id);

    dispatch_hps(&unit)
}

/// Per-operator init side effects that the transpiler can't derive from the
/// skill formula alone (e.g. values set in the operator's Python `__init__`).
fn apply_init_fixups(unit: &mut OperatorUnit, op_id: &str) {
    // Warfarin S1 heals for a fraction of the target's max HP; default it the
    // way the reference does when no explicit target HP is supplied.
    if op_id == "char_171_bldsk" && unit.skill_index == 1 {
        unit.target_hp = (1000.0 * (f64::from(unit.elite) + 1.0)).max(100.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Operators whose `skills` map has no entry for their own `default_skill`.
    fn default_skill_gaps(formulas: &HashMap<String, OperatorFormula>) -> Vec<String> {
        let mut gaps: Vec<String> = formulas
            .iter()
            .filter(|(_, f)| !f.skills.contains_key(&f.default_skill.to_string()))
            .map(|(id, _)| id.clone())
            .collect();
        gaps.sort();
        gaps
    }

    #[test]
    fn bundled_formula_tables_parse() {
        assert_eq!(load_formulas().len(), 260);
        assert_eq!(load_heal_formulas().len(), 61);
        assert_eq!(supported_operators().len(), 260);
        assert_eq!(supported_healers().len(), 61);
    }

    #[test]
    fn every_available_skill_has_a_formula() {
        for (id, f) in supported_operators().iter().chain(supported_healers()) {
            for skill in &f.available_skills {
                assert!(
                    f.skills.contains_key(&skill.to_string()),
                    "{id}: S{skill} listed but has no formula"
                );
            }
        }
    }

    #[test]
    fn every_formula_skill_is_the_custom_kind() {
        for (id, f) in supported_operators().iter().chain(supported_healers()) {
            for (key, skill) in &f.skills {
                assert_eq!(skill.formula_type, "custom", "{id} S{key}");
            }
        }
    }

    #[test]
    fn lookups_hit_and_miss() {
        assert!(get_formula("char_1044_hsgma2").is_some());
        assert!(get_formula("char_does_not_exist").is_none());
        assert!(get_heal_formula("char_473_mberry").is_some());
        assert!(get_heal_formula("char_does_not_exist").is_none());
    }

    #[test]
    fn every_dps_default_skill_is_computable() {
        // The client opens an operator on its default skill. Every default is
        // either a skill with a formula or 0, basic attack, which `calculate_dps`
        // serves without one. Before `normalize_default_skill` these eight had
        // neither and showed no DPS when first opened.
        for (id, f) in supported_operators() {
            assert!(
                f.default_skill == 0 || f.skills.contains_key(&f.default_skill.to_string()),
                "{id}: default S{} has no formula",
                f.default_skill
            );
        }
        let basic_attack_defaults: Vec<String> = default_skill_gaps(supported_operators());
        assert_eq!(
            basic_attack_defaults,
            vec![
                "char_009_12fce",
                "char_286_cast3",
                "char_347_jaksel",
                "char_4000_jnight",
                "char_4077_palico",
                "char_501_durin",
                "char_503_rang",
            ]
        );
        assert_eq!(
            get_formula("char_1044_hsgma2").map(|f| f.default_skill),
            Some(3)
        );
        assert_eq!(
            get_formula("char_347_jaksel").map(|f| f.default_skill),
            Some(0)
        );
    }

    #[test]
    fn normalize_default_skill_shapes() {
        let mut f: OperatorFormula = serde_json::from_value(serde_json::json!({
            "name": "x", "class_name": "x", "available_skills": [1, 3],
            "available_modules": [], "default_skill": 2, "default_potential": 1,
            "default_module": 0, "skills": {}, "conditionals": []
        }))
        .unwrap();
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 3, "outside the list moves to the highest");
        f.default_skill = 1;
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 1, "a listed default is kept");
        f.available_skills.clear();
        f.default_skill = 3;
        normalize_default_skill(&mut f);
        assert_eq!(f.default_skill, 0, "no skills means basic attack");
    }

    #[test]
    fn healers_with_no_default_skill_formula() {
        // `calculate_hps` never consults `skills`, so for healers this gap is
        // harmless; pinned so a regenerated table that changes it is noticed.
        assert_eq!(
            default_skill_gaps(supported_healers()),
            vec![
                "char_181_flower",
                "char_275_breeze",
                "char_285_medic2",
                "char_4091_ulika",
                "char_4163_rosesa",
                "char_449_glider",
                "char_473_mberry",
            ]
        );
    }

    #[test]
    fn skill_formula_default_is_custom_with_no_overrides() {
        let f = SkillFormula::default();
        assert_eq!(f.formula_type, "custom");
        assert!(f.atk_scale_idx.is_none() && f.hits.is_none() && f.damage_type.is_none());
    }
}
