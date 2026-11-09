use std::sync::{Arc, LazyLock};

use myrtle_dps_formulas::{
    HpsResult, dispatch, dispatch_hps,
    operator_data::OperatorData,
    operator_unit::{EnemyStats, OperatorBaseBuffs, OperatorParams, OperatorUnit},
    skill_duration_override, skill_duration_override_hps,
    tables::{SkillFormula, get_formula, get_heal_formula},
};
use myrtle_gamedata_raw::tables::operator::Operator;
use serde::{Deserialize, Serialize};

use crate::{
    request::{CycleModel, DpsRequest},
    shreds::apply_shreds,
    zones::{
        BuffZones, DamageType, ResolvedReduction, max_source, post_multiplier, product,
        resolve_enemy,
    },
};

/// The skill entry for basic attack (`skill_index` 0)
static BASIC_ATTACK: LazyLock<SkillFormula> = LazyLock::new(SkillFormula::default);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DpsResult {
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
    /// Primary enemy's share, present when request set `secondary`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<DpsMetrics>,
    /// Secondary enemies' share, `f(1 + count) - f(1)` against
    /// secondary enemy. Present when the request set `secondary`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<DpsMetrics>,
    /// Skill shape. Present under `cycleModel: "v2"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<SkillCategory>,
    /// Seconds from deployment to the first activation: `(cost - initial SP)`
    /// at the skill's SP rate. Under `cycleModel: "v2"`
    #[serde(
        rename = "startupTime",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub startup_time: Option<f64>,
    /// Seconds per skill cycle (duration + charge). Under `cycleModel: "v2"`
    #[serde(rename = "cycleTime", default, skip_serializing_if = "Option::is_none")]
    pub cycle_time: Option<f64>,
    /// The skill's SP type as the cycle saw it. Under `cycleModel: "v2"`
    #[serde(
        rename = "spRecovery",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub sp_recovery: Option<SpRecovery>,
    /// Where `average_dps` took the off-skill DPS from. Under `cycleModel: "v2"`
    /// for a skill that cycles.
    #[serde(
        rename = "offSkillSource",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub off_skill_source: Option<OffSkillSource>,
    /// `true` when a physical or arts fragility zone was requested and the
    /// formula's damage responds to both DEF and RES (or to neither).
    /// Present only when a typed fragility zone was requested.
    #[serde(
        rename = "approximateTypeSplit",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub approximate_type_split: Option<bool>,
    /// `true` when `f(1 + count) - f(1)` came out negative on a metric and the
    /// secondary share was clamped to 0 there. Present only when it fired.
    #[serde(
        rename = "secondaryClamped",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub secondary_clamped: Option<bool>,
}

/// The three damage numbers for one share of the targets
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
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

    const fn clamped(self) -> Self {
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

/// `sustained`: skill's DPS is the DPS (no skill, passive, or permanent)
/// `burst`: a timed or ammo window that recharges
/// `instant`: zero-duration activation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SkillCategory {
    Sustained,
    Burst,
    Instant,
}

/// Skill's SP type.
/// TODO: `defensive` (SP on hit) and
/// `none` (passive) are NOT modelled: their `average_dps` is the legacy one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpRecovery {
    Auto,
    Offensive,
    Defensive,
    None,
}

/// `formula`: the operator's own function at skill 0. `normalAttack`: the
/// generic `atk / interval` attack, used when the function has no skill-0 path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum OffSkillSource {
    Formula,
    NormalAttack,
}

#[must_use]
pub fn calculate_skill_dps(
    unit: &OperatorUnit,
    _formula: &SkillFormula,
    enemy: &EnemyStats,
) -> f64 {
    dispatch(unit, enemy).unwrap_or(0.0)
}

#[must_use]
pub fn calculate_dps(
    operator: &Operator,
    request: DpsRequest,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    calculate_dps_with(
        Arc::new(OperatorData::new(operator.clone())),
        request,
        enemy,
    )
}

/// The defaults the formula table gives an operator, or the basic-attack-only
/// defaults for an operator with no table
fn formula_defaults(op_id: &str) -> (i32, i32, i32, Vec<i32>, Vec<i32>) {
    get_formula(op_id).map_or((0, 1, 0, Vec::new(), Vec::new()), |f| {
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
/// clone copies
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

/// Fills the `clone_*` fields from the copied operator.
/// A module the copied operator cannot resolve falls back to the
/// copied operator's DEFAULT module
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

/// How the request's DEF / RES reach the formula
enum EnemyResolution {
    /// `apply_shreds` with the unit's legacy `shreds`
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
/// returns DEF / RES resolution. Order: A, B, C, D (D scales A and C).
fn apply_zones(unit: &mut OperatorUnit, zones: &BuffZones) -> EnemyResolution {
    if let Some(a) = zones.atk_flat {
        unit.atk += a;
    }
    if let Some(b) = zones.atk_percent {
        unit.buff_atk += b;
    }
    if !zones.inspire.is_empty() {
        let mut sources = zones.inspire.clone();
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
    unit.shreds = vec![def.ratio, def.flat, res.ratio, res.flat];
    EnemyResolution::Zones(def, res)
}

/// Which typed fragility group the formula's damage belongs to
fn damage_type(unit: &OperatorUnit, formula: &SkillFormula) -> (DamageType, bool) {
    let at = |unit: &OperatorUnit, defense, res| {
        calculate_skill_dps(unit, formula, &EnemyStats { defense, res })
    };
    let base = at(unit, 0.0, 0.0);
    #[allow(clippy::float_cmp)]
    let (by_def, by_res) = (at(unit, 2000.0, 0.0) != base, at(unit, 0.0, 90.0) != base);
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

/// The operator's own formula at skill 0 when it has a skill-0 path
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

/// SP behaviour of the unit's active skill
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

/// Cycle facts reported under `cycleModel: "v2"`
#[derive(Clone, Copy)]
struct CycleInfo {
    category: SkillCategory,
    startup_time: Option<f64>,
    cycle_time: Option<f64>,
    sp_recovery: SpRecovery,
    off_skill_source: Option<OffSkillSource>,
}

/// Everything one evaluation needs besides the unit and the enemy
struct EvalContext<'a> {
    formula: &'a SkillFormula,
    cycle: CycleModel,
    /// Multiplier on the on-skill DPS (zone F and G, or legacy `1 + fragile`)
    on_mult: f64,
    /// Multiplier on the off-skill DPS. Legacy: `1 + unit.buff_fragile`, which
    /// is `1 + 0` (engine never applied external fragile off-skill)
    /// V2: the same as `on_mult`
    off_mult: f64,
}

/// One evaluation against one already resolved enemy at the unit's current
/// `targets`
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

    let sp_rate = match sp.recovery {
        SpRecovery::Auto => Some(1.0 + f64::from(unit.sp_boost)),
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
            // TOOD: Defensive or unrecognised SP: not modelled
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

#[must_use]
pub fn calculate_dps_with(
    data: Arc<OperatorData>,
    request: DpsRequest,
    enemy: &EnemyStats,
) -> Option<DpsResult> {
    calculate_dps_detailed(data, request, enemy).map(|(result, _)| result)
}

#[must_use]
pub fn calculate_dps_detailed(
    data: Arc<OperatorData>,
    request: DpsRequest,
    enemy: &EnemyStats,
) -> Option<(DpsResult, f64)> {
    let formula = get_formula(data.data.id.as_deref()?)?;

    let DpsRequest {
        params,
        zones,
        secondary,
        cycle_model: cycle,
        clone_operator: clone,
    } = request;

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
    if let Some(duration) = skill_duration_override(&unit) {
        unit.skill_duration = duration;
    }

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
                let (kind, approximate) = damage_type(&unit, skill_formula);
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

/// Returns `None` for operators without a transpiled HPS implementation
#[must_use]
pub fn calculate_hps(operator: &Operator, params: OperatorParams) -> Option<HpsResult> {
    calculate_hps_with(Arc::new(OperatorData::new(operator.clone())), params)
}

#[must_use]
pub fn calculate_hps_with(data: Arc<OperatorData>, params: OperatorParams) -> Option<HpsResult> {
    let op_id = data.data.id.clone()?;
    let formula = get_heal_formula(&op_id)?;

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
    if let Some(duration) = skill_duration_override_hps(&unit) {
        unit.skill_duration = duration;
    }

    dispatch_hps(&unit)
}
