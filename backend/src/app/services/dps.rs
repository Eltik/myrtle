use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::GameData;
use crate::core::gamedata::types::module::ModuleType;
use crate::core::gamedata::types::operator::{Operator, OperatorModule};
use crate::dps::data_cache;
use crate::dps::engine::{
    self, DpsMetrics, DpsResult, HpsResult, OperatorFormula, SkillCategory, SpRecovery,
    calculate_dps_detailed, supported_healers, supported_operators,
};
use crate::dps::operator_unit::{
    CycleModel, EnemyStats, OperatorBuffs, OperatorConditionals, OperatorParams, OperatorShred,
    SecondaryTarget,
};
use crate::dps::zones::{BuffZones, MAX_SOURCES, StatReduction};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use ts_rs::TS;
use utoipa::ToSchema;

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInfo {
    pub conditional_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
    /// `true` when `name` describes the flag OFF: the Python reference appends
    /// the label under `if not self.<flag>` (Thorns `rangedAtk`). The switch
    /// then reads "on" when the flag is false. From
    /// `scripts/dps_conditional_polarity.py`; `false` when unresolved.
    pub inverted: bool,
}

#[derive(TS, utoipa::ToSchema)]
#[ts(export)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorListEntry {
    pub id: String,
    pub name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    /// `uniEquipId` for each entry of `available_modules`, same index, resolved
    /// through the simulator's path. Without it the client would have to guess
    /// which module a bare integer names from a list off another endpoint.
    pub available_module_ids: Vec<String>,
    pub default_skill: i32,
    pub default_module: i32,
    pub conditionals: Vec<ConditionalInfo>,
}

type OperatorModuleRef<'a> = &'a OperatorModule;

/// The operator's ADVANCED modules in uniequip order, the order `OperatorData`
/// uses, so a formula module's position indexes into it.
fn advanced_modules_sorted(operator: &Operator) -> Vec<OperatorModuleRef<'_>> {
    let mut mods: Vec<_> = operator
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect();
    mods.sort_by_key(|m| {
        m.module
            .id
            .as_deref()
            .and_then(|id| id.split('_').nth(1))
            .and_then(|n| n.parse::<i32>().ok())
            .unwrap_or(i32::MAX)
    });
    mods
}

/// Physical module for formula module `pos`, or `None` if game data lacks it.
/// Mirrors `OperatorUnit::new` exactly so advertised, named and simulated modules
/// can't drift apart.
fn resolve_module<'a>(
    sorted: &[OperatorModuleRef<'a>],
    pos: usize,
    module_value: i32,
) -> Option<OperatorModuleRef<'a>> {
    sorted.get(pos).copied().or_else(|| {
        sorted
            .iter()
            .find(|m| m.module.char_equip_order == module_value)
            .copied()
    })
}

fn build_list_entries(
    gd: &GameData,
    table: &str,
    formulas: &HashMap<String, OperatorFormula>,
) -> Vec<OperatorListEntry> {
    formulas
        .iter()
        .map(|(id, formula)| {
            // Only advertise modules the current game data can actually resolve.
            let (available_modules, available_module_ids): (Vec<i32>, Vec<String>) =
                match gd.operators.get(id) {
                    Some(operator) => {
                        let sorted = advanced_modules_sorted(operator);
                        formula
                            .available_modules
                            .iter()
                            .copied()
                            .enumerate()
                            .filter_map(|(pos, m)| {
                                resolve_module(&sorted, pos, m)
                                    .map(|module| (m, module.module.uni_equip_id.clone()))
                            })
                            .unzip()
                    }
                    None => (Vec::new(), Vec::new()),
                };
            let default_module = if available_modules.contains(&formula.default_module) {
                formula.default_module
            } else {
                0
            };
            OperatorListEntry {
                id: id.clone(),
                name: formula.name.clone(),
                available_skills: formula.available_skills.clone(),
                available_modules,
                available_module_ids,
                default_skill: formula.default_skill,
                default_module,
                conditionals: formula
                    .conditionals
                    .iter()
                    .map(|c| ConditionalInfo {
                        conditional_type: c.cond_type.clone(),
                        name: c.name.clone(),
                        default: c.default,
                        skills: c.skills.clone(),
                        modules: c.modules.clone(),
                        inverted: engine::conditional_inverted(table, id, &c.cond_type, &c.name),
                    })
                    .collect(),
            }
        })
        .collect()
}

async fn list_json(
    state: &AppState,
    kind: &'static str,
    formulas: &HashMap<String, OperatorFormula>,
) -> Result<CachedJson, ApiError> {
    let (gd, data) = data_identity(state).await?;
    let key = CacheKey::DpsList { kind, data: &data };
    cached_json(state, &key, move || async move {
        let table = if kind == "healers" { "hps" } else { "dps" };
        let entries = build_list_entries(&gd, table, formulas);
        serde_json::to_string(&entries).map_err(|e| ApiError::Internal(e.into()))
    })
    .await
}

pub async fn list_operators_json(state: &AppState) -> Result<CachedJson, ApiError> {
    list_json(state, "operators", supported_operators()).await
}

pub async fn list_healers_json(state: &AppState) -> Result<CachedJson, ApiError> {
    list_json(state, "healers", supported_healers()).await
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct RequestConditionals {
    pub trait_damage: Option<bool>,
    pub talent_damage: Option<bool>,
    pub talent2_damage: Option<bool>,
    pub skill_damage: Option<bool>,
    pub module_damage: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct CalculateRequest {
    pub operator_id: String,
    pub promotion: Option<i32>,
    pub level: Option<i32>,
    pub potential: Option<i32>,
    pub trust: Option<i32>,
    pub skill_index: Option<i32>,
    pub mastery_level: Option<i32>,
    /// Explicit pre-mastery skill level (1-7). Overrides `mastery_level` when set.
    pub skill_level: Option<i32>,
    pub module_index: Option<i32>,
    pub module_level: Option<i32>,
    pub defense: Option<f64>,
    pub res: Option<f64>,
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    pub conditionals: Option<RequestConditionals>,
    pub all_cond: Option<bool>,

    // Phase 1 fields (docs/dps-refactor.md). Each is optional and absent from
    // the serialized body when unset, so a legacy body hashes as before.
    /// Buff zones with stacking rules. Legacy `buffs`, `shred` and `spBoost`
    /// still apply and join their zone (see `dps::zones`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub zones: Option<BuffZones>,
    /// Secondary targets: the response splits into `primary` and `secondary`.
    /// Exclusive with `targets` above 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<RequestSecondary>,
    /// `"v2"`: off-skill DPS from the operator's own formula, SP time by SP
    /// type, and `category` / `startupTime` / `cycleTime` in the response.
    /// Absent or `"legacy"`: the original average.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_model: Option<RequestCycleModel>,
    /// Frame-rounded attack intervals. NOT built (docs/dps-refactor.md,
    /// Results): `true` is rejected, `false` is the only behaviour.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame_timing: Option<bool>,
    /// The operator a Muelsyse clone copies (`char_...`), built at the same
    /// investment. Absent: Ela, the reference's default (`default_clone`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_operator_id: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS, Clone, Copy)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RequestSecondary {
    pub def: f64,
    pub res: f64,
    /// Secondary enemies, 1 to 49.
    pub count: i32,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, TS, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum RequestCycleModel {
    Legacy,
    V2,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct RequestBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
    pub aspd: Option<i32>,
    pub fragile: Option<f32>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct RequestShred {
    pub def: Option<i32>,
    pub def_flat: Option<i32>,
    pub res: Option<i32>,
    pub res_flat: Option<i32>,
}

/// Bounds on a `CalculateRequest`. The endpoint is unauthenticated and feeds the
/// simulator directly. Fields that multiply the tick count (`buffs.aspd` above
/// all) need a cost ceiling, and a non-finite float comes out as a NaN result,
/// serialized as `null` under a 200.
///
/// Ranges are deliberately wider than the game allows: a cost ceiling, not a model
/// of what's reachable in play.
impl CalculateRequest {
    pub fn validate(&self) -> Result<(), ApiError> {
        int_range("promotion", self.promotion, 0, 2)?;
        int_range("level", self.level, 1, 90)?;
        int_range("potential", self.potential, 1, 6)?;
        int_range("trust", self.trust, 0, 200)?;
        int_range("skillIndex", self.skill_index, 0, 3)?;
        int_range("masteryLevel", self.mastery_level, 0, 3)?;
        int_range("skillLevel", self.skill_level, 1, 7)?;
        int_range("moduleIndex", self.module_index, -1, 5)?;
        int_range("moduleLevel", self.module_level, 0, 3)?;
        int_range("targets", self.targets, 1, 50)?;

        float_range("defense", self.defense, 0.0, 100_000.0)?;
        float_range("res", self.res, 0.0, 1_000.0)?;
        float_range("spBoost", self.sp_boost.map(f64::from), -10.0, 100.0)?;

        if let Some(buffs) = &self.buffs {
            // The one that matters: it multiplies the simulated tick count.
            int_range("buffs.aspd", buffs.aspd, -500, 1_000)?;
            int_range("buffs.flatAtk", buffs.flat_atk, -100_000, 100_000)?;
            float_range("buffs.atk", buffs.atk.map(f64::from), -10.0, 100.0)?;
            float_range("buffs.fragile", buffs.fragile.map(f64::from), -10.0, 100.0)?;
        }

        if let Some(shred) = &self.shred {
            int_range("shred.def", shred.def, -100, 100)?;
            int_range("shred.res", shred.res, -100, 100)?;
            int_range("shred.defFlat", shred.def_flat, -100_000, 100_000)?;
            int_range("shred.resFlat", shred.res_flat, -10_000, 10_000)?;
        }

        if let Some(zones) = &self.zones {
            validate_zones(zones)?;
        }

        if let Some(secondary) = &self.secondary {
            float_range("secondary.def", Some(secondary.def), 0.0, 100_000.0)?;
            float_range("secondary.res", Some(secondary.res), 0.0, 1_000.0)?;
            int_range("secondary.count", Some(secondary.count), 1, 49)?;
            if self.targets.is_some_and(|t| t > 1) {
                return Err(ApiError::BadRequest(
                    "targets and secondary are exclusive: secondary.count sets the extra targets"
                        .into(),
                ));
            }
        }

        if self.frame_timing == Some(true) {
            return Err(ApiError::BadRequest(
                "frameTiming is not available: see docs/dps-refactor.md".into(),
            ));
        }

        if let Some(id) = &self.clone_operator_id
            && (id.len() > 64 || !id.starts_with("char_"))
        {
            return Err(ApiError::BadRequest(
                "cloneOperatorId must be an operator id (char_...)".into(),
            ));
        }

        Ok(())
    }
}

/// Zone bounds: cost ceilings like the legacy fields', not a model of play.
fn validate_zones(z: &BuffZones) -> Result<(), ApiError> {
    float_range("zones.atkFlat", z.atk_flat, -100_000.0, 100_000.0)?;
    float_range("zones.atkPercent", z.atk_percent, -10.0, 100.0)?;
    float_range("zones.aspd", z.aspd, -500.0, 1_000.0)?;
    float_range("zones.spRecovery", z.sp_recovery, -10.0, 100.0)?;
    for (field, list) in z.lists() {
        if list.len() > MAX_SOURCES {
            return Err(ApiError::BadRequest(format!(
                "{field} takes at most {MAX_SOURCES} sources"
            )));
        }
        let (min, max) = match field {
            "zones.inspire" => (-100_000.0, 100_000.0),
            "zones.def.steal" => (0.0, 100_000.0),
            "zones.atkMultiplier" => (0.0, 100.0),
            "zones.damageReduction" => (0.0, 1.0),
            _ => (-10.0, 100.0),
        };
        for v in list {
            float_range(field, Some(*v), min, max)?;
            // ATK 0 reaches formulas that divide by their own damage (Thorns
            // module 3 fallout: 0 * inf = NaN), so the multiplier must be > 0.
            if field == "zones.atkMultiplier" && *v <= 0.0 {
                return Err(ApiError::BadRequest(format!(
                    "{field} must be greater than 0 (got {v})"
                )));
            }
        }
    }
    if z.res.steal.len() > MAX_SOURCES {
        return Err(ApiError::BadRequest(format!(
            "zones.res.steal takes at most {MAX_SOURCES} sources"
        )));
    }
    for v in &z.res.steal {
        float_range("zones.res.steal", Some(*v), 0.0, 10_000.0)?;
    }
    validate_reduction("zones.def", &z.def, 100_000.0)?;
    validate_reduction("zones.res", &z.res, 10_000.0)?;
    Ok(())
}

fn validate_reduction(prefix: &str, r: &StatReduction, flat_max: f64) -> Result<(), ApiError> {
    float_range(&format!("{prefix}.flat"), r.flat, -flat_max, flat_max)?;
    float_range(&format!("{prefix}.percent"), r.percent, -1.0, 1.0)?;
    float_range(
        &format!("{prefix}.ignoreFlat"),
        r.ignore_flat,
        0.0,
        flat_max,
    )?;
    float_range(
        &format!("{prefix}.ignorePercent"),
        r.ignore_percent,
        0.0,
        1.0,
    )?;
    Ok(())
}

fn int_range(field: &str, value: Option<i32>, min: i32, max: i32) -> Result<(), ApiError> {
    match value {
        Some(v) if v < min || v > max => Err(ApiError::BadRequest(format!(
            "{field} must be between {min} and {max} (got {v})"
        ))),
        _ => Ok(()),
    }
}

/// Rejects NaN and both infinities as well as out-of-range values: nothing
/// non-finite may reach the arithmetic.
fn float_range(field: &str, value: Option<f64>, min: f64, max: f64) -> Result<(), ApiError> {
    match value {
        Some(v) if !v.is_finite() => Err(ApiError::BadRequest(format!(
            "{field} must be a finite number"
        ))),
        Some(v) if v < min || v > max => Err(ApiError::BadRequest(format!(
            "{field} must be between {min} and {max} (got {v})"
        ))),
        _ => Ok(()),
    }
}

fn build_params(req: CalculateRequest) -> OperatorParams {
    OperatorParams {
        promotion: req.promotion,
        level: req.level,
        potential: req.potential,
        trust: req.trust.or(Some(100)),
        skill_index: req.skill_index,
        mastery_level: req.mastery_level,
        skill_level: req.skill_level,
        module_index: req.module_index,
        module_level: req.module_level,
        buffs: req
            .buffs
            .map(|b| OperatorBuffs {
                atk: b.atk,
                flat_atk: b.flat_atk,
                aspd: b.aspd,
                fragile: b.fragile,
            })
            .unwrap_or_default(),
        sp_boost: req.sp_boost,
        targets: req.targets,
        shred: req.shred.map(|s| OperatorShred {
            def: s.def,
            def_flat: s.def_flat,
            res: s.res,
            res_flat: s.res_flat,
        }),
        conditionals: req.conditionals.map(|c| OperatorConditionals {
            trait_damage: c.trait_damage,
            talent_damage: c.talent_damage,
            talent2_damage: c.talent2_damage,
            skill_damage: c.skill_damage,
            module_damage: c.module_damage,
        }),
        all_cond: req.all_cond,
        zones: req.zones,
        secondary: req.secondary.map(|s| SecondaryTarget {
            defense: s.def,
            res: s.res,
            count: s.count,
        }),
        cycle_model: match req.cycle_model {
            Some(RequestCycleModel::V2) => CycleModel::V2,
            Some(RequestCycleModel::Legacy) | None => CycleModel::Legacy,
        },
        ..Default::default()
    }
}

/// The operator a clone copies when the request names none.
///
/// damage_formulas.py `Muelsyse.__init__` falls back to `Ela(pp)` when no
/// previous operator is given, and every Muelsyse case in the reference
/// fixture is computed that way. Without a clone the transpiled formula
/// divides by a zero clone interval: before 2026-10-24 every Muelsyse request
/// returned `inf` (serialized as `null`), 0 of 192 fixture cases within 15%;
/// with Ela, 192 of 192.
fn default_clone(operator_id: &str) -> Option<&'static str> {
    (operator_id == "char_249_mlyss").then_some("char_4123_ela")
}

/// The default game data and its content identity, the identity computed off
/// the async worker the first time a load is seen (a hash of the operator
/// table; cached per load afterwards).
pub async fn data_identity(state: &AppState) -> Result<(Arc<GameData>, Arc<str>), ApiError> {
    let gd = state.default_game_data();
    if let Some(id) = data_cache::cached_identity(&gd) {
        return Ok((gd, id));
    }
    let hashed = Arc::clone(&gd);
    let id = crate::app::cpu::offload("dps_data_identity", move || {
        data_cache::game_data_identity(&hashed)
    })
    .await?;
    Ok((gd, id))
}

/// A non-finite number would be serialized as `null` under a 200; every
/// response number goes through this instead.
fn finite(what: &str, values: impl IntoIterator<Item = f64>) -> Result<(), ApiError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "{what} produced a non-finite number for this configuration"
        )))
    }
}

fn dps_numbers(r: &DpsResult) -> Vec<f64> {
    let mut v = vec![r.skill_dps, r.total_damage, r.average_dps];
    for m in [r.primary, r.secondary].into_iter().flatten() {
        v.extend([m.skill_dps, m.total_damage, m.average_dps]);
    }
    v.extend(r.startup_time);
    v.extend(r.cycle_time);
    v
}

/// One DPS calculation, plus the skill duration the formula ran with.
pub fn calculate_detailed_in(
    gd: &Arc<GameData>,
    req: CalculateRequest,
) -> Result<(DpsResult, f64), ApiError> {
    let data = data_cache::operator_data(gd, &req.operator_id).ok_or(ApiError::NotFound)?;
    let clone_id = req
        .clone_operator_id
        .as_deref()
        .or_else(|| default_clone(&req.operator_id));
    let clone = match clone_id {
        Some(id) => Some(data_cache::operator_data(gd, id).ok_or_else(|| {
            ApiError::BadRequest(format!("cloneOperatorId {id} is not an operator"))
        })?),
        None => None,
    };

    let enemy = EnemyStats {
        defense: req.defense.unwrap_or(0.0),
        res: req.res.unwrap_or(0.0),
    };
    let mut params = build_params(req);
    params.clone_operator = clone;

    let (result, duration) = calculate_dps_detailed(data, params, &enemy).ok_or(
        ApiError::BadRequest("DPS calculation failed for this operator/config".into()),
    )?;
    finite("DPS calculation", dps_numbers(&result))?;
    Ok((result, duration))
}

/// One DPS calculation against a given game-data load.
pub fn calculate_in(gd: &Arc<GameData>, req: CalculateRequest) -> Result<DpsResult, ApiError> {
    calculate_detailed_in(gd, req).map(|(r, _)| r)
}

pub fn calculate(state: &AppState, req: CalculateRequest) -> Result<DpsResult, ApiError> {
    calculate_in(&state.default_game_data(), req)
}

pub fn calculate_hps_in(gd: &Arc<GameData>, req: CalculateRequest) -> Result<HpsResult, ApiError> {
    let data = data_cache::operator_data(gd, &req.operator_id).ok_or(ApiError::NotFound)?;

    let params = build_params(req);

    let result = engine::calculate_hps_with(data, params).ok_or(ApiError::BadRequest(
        "HPS calculation failed for this operator/config".into(),
    ))?;
    finite(
        "HPS calculation",
        [result.skill_hps, result.base_hps, result.avg_hps],
    )?;
    Ok(result)
}

pub fn calculate_hps(state: &AppState, req: CalculateRequest) -> Result<HpsResult, ApiError> {
    calculate_hps_in(&state.default_game_data(), req)
}

// ---------------------------------------------------------------------------
// Batch
// ---------------------------------------------------------------------------

/// Most requests one batch takes.
pub const MAX_BATCH: usize = 64;
/// Most points one sweep takes.
pub const MAX_SWEEP_STEPS: u32 = 101;

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum SweepAxis {
    /// The primary enemy's DEF (`defense`).
    Def,
    /// The primary enemy's RES (`res`).
    Res,
}

/// `steps` evenly spaced values from `min` to `max` inclusive, each replacing
/// the primary DEF or RES of every request.
#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Sweep {
    pub axis: SweepAxis,
    pub min: f64,
    pub max: f64,
    /// 1 to 101. One step is `min` alone.
    pub steps: u32,
}

impl Sweep {
    /// The swept values: `min + (max - min) * i / (steps - 1)`, the last one
    /// exactly `max`.
    pub fn points(&self) -> Vec<f64> {
        let n = self.steps.max(1);
        (0..n)
            .map(|i| {
                if n == 1 {
                    self.min
                } else if i == n - 1 {
                    self.max
                } else {
                    self.min + (self.max - self.min) * f64::from(i) / f64::from(n - 1)
                }
            })
            .collect()
    }

    fn validate(&self) -> Result<(), ApiError> {
        let max = match self.axis {
            SweepAxis::Def => 100_000.0,
            SweepAxis::Res => 1_000.0,
        };
        float_range("sweep.min", Some(self.min), 0.0, max)?;
        float_range("sweep.max", Some(self.max), 0.0, max)?;
        if self.min > self.max {
            return Err(ApiError::BadRequest(
                "sweep.min must not exceed sweep.max".into(),
            ));
        }
        if self.steps < 1 || self.steps > MAX_SWEEP_STEPS {
            return Err(ApiError::BadRequest(format!(
                "sweep.steps must be between 1 and {MAX_SWEEP_STEPS} (got {})",
                self.steps
            )));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct BatchRequest {
    /// 1 to 64 calculations, each the body `/dps/calculate` takes.
    pub requests: Vec<CalculateRequest>,
    /// DPS only: evaluate every request across a DEF or RES range.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sweep: Option<Sweep>,
}

impl BatchRequest {
    /// Every request through the single-call `validate`, plus the batch bounds.
    pub fn validate(&self) -> Result<(), ApiError> {
        if self.requests.is_empty() || self.requests.len() > MAX_BATCH {
            return Err(ApiError::BadRequest(format!(
                "requests must hold between 1 and {MAX_BATCH} calculations (got {})",
                self.requests.len()
            )));
        }
        for (i, r) in self.requests.iter().enumerate() {
            r.validate().map_err(|e| match e {
                ApiError::BadRequest(m) => ApiError::BadRequest(format!("requests[{i}]: {m}")),
                other => other,
            })?;
        }
        if let Some(sweep) = &self.sweep {
            sweep.validate()?;
        }
        Ok(())
    }
}

/// One point of a sweep: the swept value and the result there.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct SweepPoint {
    pub x: f64,
    #[serde(flatten)]
    pub result: DpsResult,
}

/// One request's outcome. Exactly one of the three fields is present: `result`
/// without a sweep, `points` with one, `error` when this request failed (an
/// unknown operator, a configuration the engine cannot compute). One failed
/// request does not fail the batch.
#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct BatchItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<DpsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub points: Option<Vec<SweepPoint>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct BatchResponse {
    /// Same order as `requests`.
    pub results: Vec<BatchItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct HpsBatchItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<HpsResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HpsBatchResponse {
    pub results: Vec<HpsBatchItem>,
}

fn error_text(req: &CalculateRequest, e: &ApiError) -> String {
    match e {
        ApiError::NotFound => format!("unknown operator {}", req.operator_id),
        other => other.to_string(),
    }
}

/// Every request (and sweep point) of a batch, synchronously: the caller runs
/// this as ONE `cpu::run` job. Each value is computed by `calculate_in`, the
/// single-call path, so a batch entry equals the single call bit for bit.
pub fn batch_in(gd: &Arc<GameData>, body: BatchRequest) -> BatchResponse {
    let xs = body.sweep.as_ref().map(Sweep::points);
    let results = body
        .requests
        .into_iter()
        .map(|req| match (&body.sweep, &xs) {
            (Some(sweep), Some(xs)) => {
                let mut points = Vec::with_capacity(xs.len());
                for &x in xs {
                    let mut r = req.clone();
                    match sweep.axis {
                        SweepAxis::Def => r.defense = Some(x),
                        SweepAxis::Res => r.res = Some(x),
                    }
                    match calculate_in(gd, r) {
                        Ok(result) => points.push(SweepPoint { x, result }),
                        Err(e) => {
                            return BatchItem {
                                error: Some(format!("at {x}: {}", error_text(&req, &e))),
                                ..Default::default()
                            };
                        }
                    }
                }
                BatchItem {
                    points: Some(points),
                    ..Default::default()
                }
            }
            _ => {
                let label = req.clone();
                match calculate_in(gd, req) {
                    Ok(result) => BatchItem {
                        result: Some(result),
                        ..Default::default()
                    },
                    Err(e) => BatchItem {
                        error: Some(error_text(&label, &e)),
                        ..Default::default()
                    },
                }
            }
        })
        .collect();
    BatchResponse { results }
}

/// The HPS batch: no sweep (healing does not read DEF or RES).
pub fn hps_batch_in(gd: &Arc<GameData>, body: BatchRequest) -> HpsBatchResponse {
    let results = body
        .requests
        .into_iter()
        .map(|req| {
            let label = req.clone();
            match calculate_hps_in(gd, req) {
                Ok(result) => HpsBatchItem {
                    result: Some(result),
                    error: None,
                },
                Err(e) => HpsBatchItem {
                    result: None,
                    error: Some(error_text(&label, &e)),
                },
            }
        })
        .collect();
    HpsBatchResponse { results }
}

// ---------------------------------------------------------------------------
// Ranking
// ---------------------------------------------------------------------------

/// The investment every ranked row is computed at.
#[derive(
    Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema, TS, PartialEq, Eq, Hash,
)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RankPreset {
    /// Every investment field left to the engine: max promotion and level,
    /// max skill rank, trust 100, the formula's default potential, module
    /// level 3 for a module row.
    Default,
    /// The ArkDPS / UOEP convention: E2 (or the rarity's highest) at max
    /// level, trust 100, M3 (or max rank), module level 3, potential 1.
    #[default]
    P1Max,
    /// `p1Max` at potential 6.
    P6Max,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize, ToSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum RankMetric {
    #[default]
    SkillDps,
    TotalDamage,
    AverageDps,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, ToSchema, TS, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[ts(export)]
pub enum DamageKind {
    Physical,
    Arts,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase", default)]
#[ts(export, optional_fields)]
pub struct RankFilters {
    /// Game profession codes: `WARRIOR`, `SNIPER`, `CASTER`, `PIONEER`,
    /// `TANK`, `MEDIC`, `SUPPORT`, `SPECIAL`. Empty or absent = every class.
    pub professions: Option<Vec<String>>,
    /// Star counts, 1 to 6.
    pub rarities: Option<Vec<i32>>,
    /// The operator's attack type (`is_physical` in game data).
    pub damage_types: Option<Vec<DamageKind>>,
}

/// The shared context every ranked row is computed under. Every field has the
/// meaning it has on `CalculateRequest`; `operatorId`, the investment fields,
/// `skillIndex` and `moduleIndex` are set per row.
#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export, optional_fields)]
pub struct RankRequest {
    pub defense: Option<f64>,
    pub res: Option<f64>,
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    pub all_cond: Option<bool>,
    pub zones: Option<BuffZones>,
    pub secondary: Option<RequestSecondary>,
    pub cycle_model: Option<RequestCycleModel>,
    /// Absent: `p1Max`.
    pub preset: Option<RankPreset>,
    /// Applied after the ranking is computed (and cached); they never change
    /// a row.
    pub filters: Option<RankFilters>,
    /// Absent: `skillDps`. Descending.
    pub sort_by: Option<RankMetric>,
    /// Rows returned after filtering and sorting, at most 5000. Absent: all.
    pub limit: Option<u32>,
}

/// The row-affecting half of a `RankRequest`: what the cache keys on.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RankContext<'a> {
    defense: Option<f64>,
    res: Option<f64>,
    buffs: &'a Option<RequestBuffs>,
    shred: &'a Option<RequestShred>,
    targets: Option<i32>,
    sp_boost: Option<f32>,
    all_cond: Option<bool>,
    zones: &'a Option<BuffZones>,
    secondary: Option<RequestSecondary>,
    cycle_model: Option<RequestCycleModel>,
    preset: RankPreset,
}

impl RankRequest {
    fn context(&self) -> RankContext<'_> {
        RankContext {
            defense: self.defense,
            res: self.res,
            buffs: &self.buffs,
            shred: &self.shred,
            targets: self.targets,
            sp_boost: self.sp_boost,
            all_cond: self.all_cond,
            zones: &self.zones,
            secondary: self.secondary,
            cycle_model: self.cycle_model,
            preset: self.preset.unwrap_or_default(),
        }
    }

    /// Hash of the row-affecting fields (`DefaultHasher::new()`, fixed keys,
    /// the same scheme as the calculate body hash).
    pub fn context_hash(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        serde_json::to_string(&self.context())
            .unwrap_or_default()
            .hash(&mut hasher);
        hasher.finish()
    }

    /// The calculation for one row.
    pub fn row_request(&self, operator_id: &str, skill: i32, module: i32) -> CalculateRequest {
        let preset = self.preset.unwrap_or_default();
        let maxed = preset != RankPreset::Default;
        CalculateRequest {
            operator_id: operator_id.to_owned(),
            promotion: maxed.then_some(2),
            level: maxed.then_some(90),
            potential: match preset {
                RankPreset::Default => None,
                RankPreset::P1Max => Some(1),
                RankPreset::P6Max => Some(6),
            },
            trust: maxed.then_some(100),
            skill_index: Some(skill),
            mastery_level: maxed.then_some(3),
            skill_level: None,
            module_index: Some(module),
            module_level: (maxed && module > 0).then_some(3),
            defense: self.defense,
            res: self.res,
            buffs: self.buffs.clone(),
            shred: self.shred.clone(),
            targets: self.targets,
            sp_boost: self.sp_boost,
            conditionals: None,
            all_cond: self.all_cond,
            zones: self.zones.clone(),
            secondary: self.secondary,
            cycle_model: self.cycle_model,
            frame_timing: None,
            clone_operator_id: None,
        }
    }

    pub fn validate(&self) -> Result<(), ApiError> {
        self.row_request("char_rank_probe", 0, 0).validate()?;
        if self.limit.is_some_and(|l| l > 5000) {
            return Err(ApiError::BadRequest("limit must be at most 5000".into()));
        }
        if let Some(f) = &self.filters
            && (f.professions.as_ref().is_some_and(|v| v.len() > 16)
                || f.rarities.as_ref().is_some_and(|v| v.len() > 16)
                || f.damage_types.as_ref().is_some_and(|v| v.len() > 4))
        {
            return Err(ApiError::BadRequest("too many filter values".into()));
        }
        Ok(())
    }
}

/// One (operator, skill, module) of the ranking.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RankRow {
    pub operator_id: String,
    pub name: String,
    pub rarity: i32,
    /// Game profession code (`SNIPER`, ...).
    pub profession: String,
    pub sub_profession_id: String,
    pub damage_type: DamageKind,
    /// 0 = basic attack, 1-3 = S1-S3 (the request's `skillIndex`).
    pub skill_index: i32,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_name: Option<String>,
    /// 0 = no module, else the formula's module number (`moduleIndex`).
    pub module_index: i32,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_id: Option<String>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module_name: Option<String>,
    pub skill_dps: f64,
    pub total_damage: f64,
    pub average_dps: f64,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<DpsMetrics>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secondary: Option<DpsMetrics>,
    /// Under `cycleModel: "v2"`.
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<SkillCategory>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sp_recovery: Option<SpRecovery>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub startup_time: Option<f64>,
    /// Seconds the skill lasts as the formula ran it; absent for a skill with
    /// no duration (instant, permanent, basic attack).
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[ts(optional)]
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_time: Option<f64>,
    /// Share of this row's damage that survives the real enemy roster, at the
    /// REFERENCE context (p1Max, no buffs, single target, legacy cycle), never
    /// at this request's: see `GET /dps/retention`. `null` until the table is
    /// computed (`retentionStatus`), or when the row has no damage at DEF 0
    /// RES 0.
    #[serde(default)]
    pub retention: Option<f64>,
}

/// The cached, unfiltered ranking for one context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankComputed {
    pub rows: Vec<RankRow>,
    /// (operator, skill, module) combinations the engine returned no result for.
    pub skipped: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RankResponse {
    pub rows: Vec<RankRow>,
    /// Rows that passed the filters, before `limit`.
    pub matched: usize,
    /// Rows the context computed, before filters.
    pub computed: usize,
    /// Combinations the engine could not compute (missing skill or module data).
    pub skipped: usize,
    pub preset: RankPreset,
    pub sort_by: RankMetric,
    /// The game-data identity the rows were computed on.
    pub data: String,
    /// Whether the rows' `retention` is filled yet.
    pub retention_status: crate::app::services::dps_retention::RetentionStatus,
}

fn profession_code(op: &crate::core::gamedata::types::operator::Operator) -> String {
    serde_json::to_value(&op.profession)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_default()
}

/// Every (operator, skill, module) the DPS formula table supports, computed
/// under one context. Synchronous: run it inside `cpu::run`. Each row comes
/// from `calculate_detailed_in`, the single-call path.
pub fn rank_rows_in(gd: &Arc<GameData>, req: &RankRequest) -> RankComputed {
    let mut ids: Vec<&String> = supported_operators().keys().collect();
    ids.sort();
    let mut rows = Vec::new();
    let mut skipped = 0;
    for id in ids {
        let formula = &supported_operators()[id];
        let Some(operator) = gd.operators.get(id) else {
            continue;
        };
        let Some(data) = data_cache::operator_data(gd, id) else {
            continue;
        };
        let sorted = advanced_modules_sorted(operator);
        let mut modules: Vec<(i32, Option<OperatorModuleRef<'_>>)> = vec![(0, None)];
        for (pos, &m) in formula.available_modules.iter().enumerate() {
            if let Some(module) = resolve_module(&sorted, pos, m) {
                modules.push((m, Some(module)));
            }
        }
        let skills: Vec<i32> = if formula.available_skills.is_empty() {
            vec![0]
        } else {
            formula.available_skills.clone()
        };
        for &skill in &skills {
            let skill_static = usize::try_from(skill - 1)
                .ok()
                .and_then(|i| operator.skills.get(i));
            for &(module, module_ref) in &modules {
                let Ok((r, duration)) =
                    calculate_detailed_in(gd, req.row_request(id, skill, module))
                else {
                    skipped += 1;
                    continue;
                };
                rows.push(RankRow {
                    operator_id: id.clone(),
                    name: operator.name.clone(),
                    rarity: data.rarity,
                    profession: profession_code(operator),
                    sub_profession_id: operator.sub_profession_id.clone(),
                    damage_type: if data.is_physical {
                        DamageKind::Physical
                    } else {
                        DamageKind::Arts
                    },
                    skill_index: skill,
                    skill_id: skill_static.map(|s| s.skill_id.clone()),
                    skill_name: skill_static
                        .and_then(|s| s.static_data.as_ref())
                        .and_then(|s| s.levels.last())
                        .map(|l| l.name.clone()),
                    module_index: module,
                    module_id: module_ref.map(|m| m.module.uni_equip_id.clone()),
                    module_name: module_ref.map(|m| m.module.uni_equip_name.clone()),
                    skill_dps: r.skill_dps,
                    total_damage: r.total_damage,
                    average_dps: r.average_dps,
                    primary: r.primary,
                    secondary: r.secondary,
                    category: r.category,
                    sp_recovery: r.sp_recovery,
                    startup_time: r.startup_time,
                    duration: (skill > 0 && duration > 0.0).then_some(duration),
                    cycle_time: r.cycle_time,
                    retention: None,
                });
            }
        }
    }
    RankComputed { rows, skipped }
}

/// An absent or empty filter passes everything.
fn passes<T>(filter: &Option<Vec<T>>, matches: impl Fn(&T) -> bool) -> bool {
    match filter {
        Some(values) if !values.is_empty() => values.iter().any(matches),
        _ => true,
    }
}

/// Filters, sorts (descending, ties by operator, skill, module) and limits.
pub fn finish_rank(computed: &RankComputed, req: &RankRequest, data: &str) -> RankResponse {
    let sort_by = req.sort_by.unwrap_or_default();
    let f = req.filters.clone().unwrap_or_default();
    let mut rows: Vec<RankRow> = computed
        .rows
        .iter()
        .filter(|r| {
            passes(&f.professions, |p| p.eq_ignore_ascii_case(&r.profession))
                && passes(&f.rarities, |v| *v == r.rarity)
                && passes(&f.damage_types, |v| *v == r.damage_type)
        })
        .cloned()
        .collect();
    let metric = |r: &RankRow| match sort_by {
        RankMetric::SkillDps => r.skill_dps,
        RankMetric::TotalDamage => r.total_damage,
        RankMetric::AverageDps => r.average_dps,
    };
    rows.sort_by(|a, b| {
        metric(b)
            .total_cmp(&metric(a))
            .then_with(|| a.operator_id.cmp(&b.operator_id))
            .then_with(|| a.skill_index.cmp(&b.skill_index))
            .then_with(|| a.module_index.cmp(&b.module_index))
    });
    let matched = rows.len();
    if let Some(limit) = req.limit {
        rows.truncate(limit as usize);
    }
    RankResponse {
        rows,
        matched,
        computed: computed.rows.len(),
        skipped: computed.skipped,
        preset: req.preset.unwrap_or_default(),
        sort_by,
        data: data.to_owned(),
        retention_status: crate::app::services::dps_retention::RetentionStatus::Pending,
    }
}

// ---------------------------------------------------------------------------
// Supporters
// ---------------------------------------------------------------------------

pub use crate::dps::supporters::{SupporterInvestment, SupportersResponse};

/// Every supporter table entry resolved at one investment. Pure table walk
/// plus cached `OperatorData`, milliseconds.
pub fn supporters_in(gd: &Arc<GameData>, inv: SupporterInvestment) -> SupportersResponse {
    use crate::dps::supporters::{SUPPORTERS, UnresolvedSupporter, resolve};
    let mut entries = Vec::new();
    let mut unresolved = Vec::new();
    for def in SUPPORTERS {
        let outcome = data_cache::operator_data(gd, def.char_id)
            .ok_or_else(|| format!("{} is not in the loaded game data", def.char_id))
            .and_then(|data| resolve(def, &data, inv));
        match outcome {
            Ok(entry) => entries.push(entry),
            Err(reason) => unresolved.push(UnresolvedSupporter {
                id: def.id.to_owned(),
                reason,
            }),
        }
    }
    SupportersResponse {
        entries,
        unresolved,
    }
}
