use crate::app::cache::keys::CacheKey;
use crate::app::cache::{CachedJson, cached_json};
use crate::app::error::ApiError;
use crate::app::state::AppState;
use crate::core::gamedata::types::module::ModuleType;
use crate::core::gamedata::types::operator::{Operator, OperatorModule};
use crate::dps::engine::{
    self, DpsResult, HpsResult, OperatorFormula, calculate_dps, supported_healers,
    supported_operators,
};
use crate::dps::operator_unit::{
    EnemyStats, OperatorBuffs, OperatorConditionals, OperatorParams, OperatorShred,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

#[derive(TS)]
#[ts(export)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInfo {
    pub conditional_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorListEntry {
    pub id: String,
    pub name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    /// `uniEquipId` for each entry of `available_modules`, same index, resolved
    /// through the same path the simulator uses. Without this the client has
    /// only a bare integer and has to guess which module it names by counting
    /// down a list it fetched from a different endpoint.
    pub available_module_ids: Vec<String>,
    pub default_skill: i32,
    pub default_module: i32,
    pub conditionals: Vec<ConditionalInfo>,
}

type OperatorModuleRef<'a> = &'a OperatorModule;

/// The operator's ADVANCED modules, sorted by uniequip number - the same order
/// `OperatorData` uses, so a formula module's position indexes into it.
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

/// The physical module a formula module at `pos` resolves to, or `None` when the
/// current game data has no such module. Mirrors the resolution in
/// `OperatorUnit::new` exactly, so what the API advertises, what it names, and
/// what the engine simulates cannot drift apart.
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
    state: &AppState,
    formulas: &HashMap<String, OperatorFormula>,
) -> Vec<OperatorListEntry> {
    let gd = state.default_game_data();
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
                    // Operator absent from game data entirely - advertise nothing.
                    None => (Vec::new(), Vec::new()),
                };
            // Keep default_module consistent: if it's been filtered out, drop it.
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
    let key = CacheKey::DpsList { kind };
    cached_json(state, &key, move || async move {
        let entries = build_list_entries(state, formulas);
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestConditionals {
    pub trait_damage: Option<bool>,
    pub talent_damage: Option<bool>,
    pub talent2_damage: Option<bool>,
    pub skill_damage: Option<bool>,
    pub module_damage: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CalculateRequest {
    pub operator_id: String,
    // Operator config
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
    // Enemy
    pub defense: Option<f64>,
    pub res: Option<f64>,
    // Buffs
    pub buffs: Option<RequestBuffs>,
    pub shred: Option<RequestShred>,
    // Targets
    pub targets: Option<i32>,
    pub sp_boost: Option<f32>,
    // Conditionals
    pub conditionals: Option<RequestConditionals>,
    pub all_cond: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestBuffs {
    pub atk: Option<f32>,
    pub flat_atk: Option<i32>,
    pub aspd: Option<i32>,
    pub fragile: Option<f32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestShred {
    pub def: Option<i32>,
    pub def_flat: Option<i32>,
    pub res: Option<i32>,
    pub res_flat: Option<i32>,
}

/// Bounds on a `CalculateRequest`.
///
/// The endpoint is unauthenticated and every field feeds the simulator
/// directly, so two classes of input have to be refused here. Fields that
/// multiply the simulated tick count - `buffs.aspd` above all - decide how much
/// work one request costs, and need a ceiling. Non-finite floats propagate
/// through the arithmetic into a NaN result, which serialises as JSON `null`
/// under a 200: a wrong answer presented as a correct one.
///
/// The ranges are deliberately wider than the game allows. This is a ceiling on
/// cost, not a model of what is reachable in play, and a speculative query
/// should not be refused.
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

        Ok(())
    }
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
        ..Default::default()
    }
}

pub fn calculate(state: &AppState, req: CalculateRequest) -> Result<DpsResult, ApiError> {
    let gd = state.default_game_data();
    let operator = gd
        .operators
        .get(&req.operator_id)
        .ok_or(ApiError::NotFound)?;

    let enemy = EnemyStats {
        defense: req.defense.unwrap_or(0.0),
        res: req.res.unwrap_or(0.0),
    };
    let params = build_params(req);

    calculate_dps(operator, params, &enemy).ok_or(ApiError::BadRequest(
        "DPS calculation failed for this operator/config".into(),
    ))
}

pub fn calculate_hps(state: &AppState, req: CalculateRequest) -> Result<HpsResult, ApiError> {
    let gd = state.default_game_data();
    let operator = gd
        .operators
        .get(&req.operator_id)
        .ok_or(ApiError::NotFound)?;

    let params = build_params(req);

    engine::calculate_hps(operator, params).ok_or(ApiError::BadRequest(
        "HPS calculation failed for this operator/config".into(),
    ))
}
