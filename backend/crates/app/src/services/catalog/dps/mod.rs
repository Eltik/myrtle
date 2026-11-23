mod rank;
mod request;
mod retention;
pub mod watch;

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::Duration;

use dashmap::DashMap;
use myrtle_dps_formulas::operator_data::{OperatorData, uniequip_number};
use myrtle_dps_formulas::operator_unit::EnemyStats;
use myrtle_dps_formulas::tables::{
    OperatorFormula, conditional_inverted, supported_healers, supported_operators,
};
use myrtle_engine_dps::supporters::{
    SUPPORTERS, SupporterInvestment, SupportersResponse, UnresolvedSupporter, resolve,
};
use myrtle_engine_dps::{
    DpsResult, ENGINE_VERSION, HpsResult, calculate_dps_detailed, calculate_hps_with,
};
use myrtle_gamedata::model::GameModel;
use myrtle_gamedata_raw::tables::module::ModuleType;
use myrtle_gamedata_raw::tables::operator::{Operator, OperatorModule};
use serde::Serialize;
use tokio::sync::OnceCell;

pub use rank::{
    DamageKind, RankComputed, RankFilters, RankMetric, RankPreset, RankRequest, RankResponse,
    RankRow, finish_rank, rank_rows_in,
};
pub use request::{
    BatchItem, BatchRequest, BatchResponse, CalculateRequest, HpsBatchItem, HpsBatchResponse,
    MAX_BATCH, MAX_SWEEP_STEPS, RequestBuffs, RequestConditionals, RequestCycleModel,
    RequestSecondary, RequestShred, SupportersQuery, Sweep, SweepAxis, SweepPoint,
};
pub use retention::{
    REFERENCE, RetentionExclusion, RetentionResponse, RetentionRoster, RetentionRow,
    RetentionStatus, RetentionTable, RosterPair, attach, compute_retention, enemy_roster,
    reference_context,
};

use crate::cache::{CachedJson, key_hash};
use crate::snapshots::Snapshot;
use crate::{AppContext, AppError};

const CALC_TTL: Duration = Duration::from_hours(1);
const LIST_TTL: Duration = Duration::from_hours(1);
const RANK_GRID_TTL: Duration = Duration::from_hours(24);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionalInfo {
    pub conditional_type: String,
    pub name: String,
    pub default: bool,
    pub skills: Vec<i32>,
    pub modules: Vec<i32>,
    pub inverted: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OperatorListEntry {
    pub id: String,
    pub name: String,
    pub available_skills: Vec<i32>,
    pub available_modules: Vec<i32>,
    pub available_module_ids: Vec<String>,
    pub default_skill: i32,
    pub default_module: i32,
    pub conditionals: Vec<ConditionalInfo>,
}

struct Memo {
    owner: Weak<Snapshot>,
    identity: Arc<str>,
    operators: DashMap<String, Arc<OperatorData>>,
    retention: OnceCell<Arc<RetentionTable>>,
}

static MEMOS: Mutex<Vec<Arc<Memo>>> = Mutex::new(Vec::new());

fn memo_of(snapshot: &Arc<Snapshot>) -> Option<Arc<Memo>> {
    MEMOS
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .iter()
        .find(|m| std::ptr::eq(m.owner.as_ptr(), Arc::as_ptr(snapshot)))
        .cloned()
}

#[derive(Clone)]
pub struct Data {
    pub snapshot: Arc<Snapshot>,
    memo: Arc<Memo>,
}

impl Data {
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.memo.identity
    }

    #[must_use]
    pub fn operator(&self, id: &str) -> Option<Arc<OperatorData>> {
        if let Some(hit) = self.memo.operators.get(id) {
            return Some(Arc::clone(&hit));
        }
        let operator = self.snapshot.model.operators.get(id)?;
        let built = Arc::new(OperatorData::new(operator.clone()));
        Some(Arc::clone(
            &self.memo.operators.entry(id.to_owned()).or_insert(built),
        ))
    }

    #[must_use]
    pub fn retention_table(&self) -> Option<Arc<RetentionTable>> {
        self.memo.retention.get().cloned()
    }
}

fn identity_of(model: &GameModel) -> Arc<str> {
    let value = serde_json::to_value(&model.operators).unwrap_or_default();
    let table = serde_json::to_vec(&value).unwrap_or_default();
    Arc::from(key_hash(&table))
}

pub async fn data(ctx: &AppContext) -> Result<Data, AppError> {
    let snapshot = ctx.snapshots.or_default(ctx.snapshots.default_server())?;
    if let Some(memo) = memo_of(&snapshot) {
        return Ok(Data { snapshot, memo });
    }
    let hashed = Arc::clone(&snapshot);
    let identity = ctx
        .cpu
        .offload("dps_data_identity", move || identity_of(&hashed.model))
        .await?;
    let mut memos = MEMOS.lock().unwrap_or_else(PoisonError::into_inner);
    memos.retain(|m| m.owner.strong_count() > 0);
    let memo = if let Some(m) = memos
        .iter()
        .find(|m| std::ptr::eq(m.owner.as_ptr(), Arc::as_ptr(&snapshot)))
    {
        Arc::clone(m)
    } else {
        let memo = Arc::new(Memo {
            owner: Arc::downgrade(&snapshot),
            identity,
            operators: DashMap::new(),
            retention: OnceCell::new(),
        });
        memos.push(Arc::clone(&memo));
        memo
    };
    drop(memos);
    Ok(Data { snapshot, memo })
}

fn calc_key(kind: &str, data: &Data, body: &impl Serialize) -> String {
    let body = serde_json::to_string(body).unwrap_or_default();
    format!(
        "dps:calc:{kind}:{ENGINE_VERSION}:{}:{}",
        data.identity(),
        key_hash(body.as_bytes())
    )
}

fn to_json(value: &impl Serialize) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(AppError::internal)
}

fn advanced_modules(operator: &Operator) -> Vec<&OperatorModule> {
    let mut mods: Vec<_> = operator
        .modules
        .iter()
        .filter(|m| m.module.module_type == ModuleType::Advanced)
        .collect();
    mods.sort_by_key(|m| uniequip_number(m));
    mods
}

fn resolve_module<'a>(
    sorted: &[&'a OperatorModule],
    pos: usize,
    module_value: i32,
) -> Option<&'a OperatorModule> {
    sorted.get(pos).copied().or_else(|| {
        sorted
            .iter()
            .find(|m| m.module.char_equip_order == module_value)
            .copied()
    })
}

fn list_entries(
    model: &GameModel,
    table: &str,
    formulas: &HashMap<String, OperatorFormula>,
) -> Vec<OperatorListEntry> {
    formulas
        .iter()
        .map(|(id, formula)| {
            let (available_modules, available_module_ids): (Vec<i32>, Vec<String>) =
                model.operators.get(id).map_or_else(Default::default, |op| {
                    let sorted = advanced_modules(op);
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
                });
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
                        inverted: conditional_inverted(table, id, &c.cond_type, &c.name),
                    })
                    .collect(),
            }
        })
        .collect()
}

async fn list(ctx: &AppContext, kind: &'static str) -> Result<CachedJson, AppError> {
    let data = data(ctx).await?;
    let key = format!("dps:list:{kind}:{ENGINE_VERSION}:{}", data.identity());
    ctx.cache
        .json(&key, LIST_TTL, || async move {
            let (table, formulas) = if kind == "healers" {
                ("hps", supported_healers())
            } else {
                ("dps", supported_operators())
            };
            to_json(&list_entries(&data.snapshot.model, table, formulas))
        })
        .await
}

pub async fn list_operators(ctx: &AppContext) -> Result<CachedJson, AppError> {
    list(ctx, "operators").await
}

pub async fn list_healers(ctx: &AppContext) -> Result<CachedJson, AppError> {
    list(ctx, "healers").await
}

fn default_clone(operator_id: &str) -> Option<&'static str> {
    (operator_id == "char_249_mlyss").then_some("char_4123_ela")
}

fn finite(what: &str, values: impl IntoIterator<Item = f64>) -> Result<(), AppError> {
    if values.into_iter().all(f64::is_finite) {
        Ok(())
    } else {
        Err(AppError::BadRequest(format!(
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

pub fn calculate_detailed_in(
    data: &Data,
    req: CalculateRequest,
) -> Result<(DpsResult, f64), AppError> {
    let operator = data
        .operator(&req.operator_id)
        .ok_or(AppError::NotFound(None))?;
    let clone = match req
        .clone_operator_id
        .as_deref()
        .or_else(|| default_clone(&req.operator_id))
    {
        Some(id) => Some(data.operator(id).ok_or_else(|| {
            AppError::BadRequest(format!("cloneOperatorId {id} is not an operator"))
        })?),
        None => None,
    };
    let enemy = EnemyStats {
        defense: req.defense.unwrap_or(0.0),
        res: req.res.unwrap_or(0.0),
    };
    let mut request = req.params();
    request.clone_operator = clone;
    let (result, duration) = calculate_dps_detailed(operator, request, &enemy)
        .ok_or_else(|| AppError::bad_request("DPS calculation failed for this operator/config"))?;
    finite("DPS calculation", dps_numbers(&result))?;
    Ok((result, duration))
}

pub fn calculate_in(data: &Data, req: CalculateRequest) -> Result<DpsResult, AppError> {
    calculate_detailed_in(data, req).map(|(r, _)| r)
}

pub fn calculate_hps_in(data: &Data, req: CalculateRequest) -> Result<HpsResult, AppError> {
    let operator = data
        .operator(&req.operator_id)
        .ok_or(AppError::NotFound(None))?;
    let result = calculate_hps_with(operator, req.params().params)
        .ok_or_else(|| AppError::bad_request("HPS calculation failed for this operator/config"))?;
    finite(
        "HPS calculation",
        [result.skill_hps, result.base_hps, result.avg_hps],
    )?;
    Ok(result)
}

fn error_text(operator_id: &str, e: &AppError) -> String {
    match e {
        AppError::NotFound(_) => format!("unknown operator {operator_id}"),
        other => other.to_string(),
    }
}

pub fn batch_in(data: &Data, body: BatchRequest) -> BatchResponse {
    let xs = body.sweep.as_ref().map(Sweep::points);
    let results = body
        .requests
        .into_iter()
        .map(|req| {
            let operator_id = req.operator_id.clone();
            let (Some(sweep), Some(xs)) = (&body.sweep, &xs) else {
                return match calculate_in(data, req) {
                    Ok(result) => BatchItem {
                        result: Some(result),
                        ..BatchItem::default()
                    },
                    Err(e) => BatchItem {
                        error: Some(error_text(&operator_id, &e)),
                        ..BatchItem::default()
                    },
                };
            };
            let mut points = Vec::with_capacity(xs.len());
            for &x in xs {
                let mut r = req.clone();
                match sweep.axis {
                    SweepAxis::Def => r.defense = Some(x),
                    SweepAxis::Res => r.res = Some(x),
                }
                match calculate_in(data, r) {
                    Ok(result) => points.push(SweepPoint { x, result }),
                    Err(e) => {
                        return BatchItem {
                            error: Some(format!("at {x}: {}", error_text(&operator_id, &e))),
                            ..BatchItem::default()
                        };
                    }
                }
            }
            BatchItem {
                points: Some(points),
                ..BatchItem::default()
            }
        })
        .collect();
    BatchResponse { results }
}

#[must_use]
pub fn hps_batch_in(data: &Data, body: BatchRequest) -> HpsBatchResponse {
    let results = body
        .requests
        .into_iter()
        .map(|req| {
            let operator_id = req.operator_id.clone();
            match calculate_hps_in(data, req) {
                Ok(result) => HpsBatchItem {
                    result: Some(result),
                    error: None,
                },
                Err(e) => HpsBatchItem {
                    result: None,
                    error: Some(error_text(&operator_id, &e)),
                },
            }
        })
        .collect();
    HpsBatchResponse { results }
}

#[must_use]
pub fn supporters_in(data: &Data, inv: SupporterInvestment) -> SupportersResponse {
    let mut entries = Vec::new();
    let mut unresolved = Vec::new();
    for def in SUPPORTERS {
        let outcome = data
            .operator(def.char_id)
            .ok_or_else(|| format!("{} is not in the loaded game data", def.char_id))
            .and_then(|op| resolve(def, &op, inv));
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

pub async fn calculate(ctx: &AppContext, req: CalculateRequest) -> Result<CachedJson, AppError> {
    req.validate()?;
    let data = data(ctx).await?;
    let key = calc_key("dps3", &data, &req);
    ctx.cache
        .json(&key, CALC_TTL, || async move {
            let result = ctx
                .cpu
                .run("dps_calculate", move || calculate_in(&data, req))
                .await??;
            to_json(&result)
        })
        .await
}

pub async fn calculate_hps(
    ctx: &AppContext,
    req: CalculateRequest,
) -> Result<CachedJson, AppError> {
    req.validate()?;
    let data = data(ctx).await?;
    let key = calc_key("hps3", &data, &req);
    ctx.cache
        .json(&key, CALC_TTL, || async move {
            let result = ctx
                .cpu
                .run("hps_calculate", move || calculate_hps_in(&data, req))
                .await??;
            to_json(&result)
        })
        .await
}

pub async fn batch(ctx: &AppContext, body: BatchRequest) -> Result<CachedJson, AppError> {
    body.validate()?;
    let data = data(ctx).await?;
    let key = calc_key("dpsbatch1", &data, &body);
    ctx.cache
        .json(&key, CALC_TTL, || async move {
            let result = ctx
                .cpu
                .run("dps_batch", move || batch_in(&data, body))
                .await?;
            to_json(&result)
        })
        .await
}

pub async fn batch_hps(ctx: &AppContext, body: BatchRequest) -> Result<CachedJson, AppError> {
    body.validate()?;
    if body.sweep.is_some() {
        return Err(AppError::bad_request(
            "sweep is DPS only: healing does not read DEF or RES",
        ));
    }
    let data = data(ctx).await?;
    let key = calc_key("hpsbatch1", &data, &body);
    ctx.cache
        .json(&key, CALC_TTL, || async move {
            let result = ctx
                .cpu
                .run("hps_batch", move || hps_batch_in(&data, body))
                .await?;
            to_json(&result)
        })
        .await
}

pub async fn rank(ctx: &AppContext, req: RankRequest) -> Result<RankResponse, AppError> {
    req.validate()?;
    let data = data(ctx).await?;
    let key = format!(
        "dps:rank:{ENGINE_VERSION}:{}:{}",
        data.identity(),
        key_hash(req.context_json().as_bytes())
    );
    let computed = if let Some(hit) = ctx.cache.get_json::<RankComputed>(&key).await {
        hit
    } else {
        let (rows_data, context) = (data.clone(), req.clone());
        let computed = ctx
            .cpu
            .run("dps_rank", move || rank_rows_in(&rows_data, &context))
            .await?;
        let ttl = if req.on_page_grid() {
            RANK_GRID_TTL
        } else {
            Duration::from_secs(ctx.config.dps.rank_off_grid_ttl_secs)
        };
        ctx.cache.set_json(&key, &computed, ttl).await;
        computed
    };
    let mut response = finish_rank(&computed, &req, data.identity());
    attach(&data, &mut response);
    Ok(response)
}

pub async fn retention(ctx: &AppContext) -> Result<RetentionResponse, AppError> {
    let data = data(ctx).await?;
    let computed = if let Some(table) = data.retention_table() {
        Ok(table)
    } else {
        let (cpu, table_data) = (ctx.cpu.clone(), data.clone());
        tokio::spawn(async move {
            let memo = Arc::clone(&table_data.memo);
            memo.retention
                .get_or_try_init(|| async move {
                    cpu.run("dps_retention", move || {
                        Arc::new(compute_retention(&table_data))
                    })
                    .await
                })
                .await
                .cloned()
        })
        .await
        .unwrap_or(Err(AppError::Unavailable))
    };
    match computed {
        Ok(table) => Ok(RetentionResponse::new(
            &data,
            RetentionStatus::of(Some(&table)),
            Some(&table),
        )),
        Err(AppError::Unavailable) => Err(AppError::Unavailable),
        Err(e) => {
            tracing::error!(error = %e, "dps retention failed");
            Ok(RetentionResponse::new(&data, RetentionStatus::Failed, None))
        }
    }
}

pub async fn supporters(ctx: &AppContext, query: SupportersQuery) -> Result<CachedJson, AppError> {
    let d = SupporterInvestment::default();
    let inv = SupporterInvestment {
        promotion: query.promotion.unwrap_or(d.promotion),
        skill_level: query.skill_level.unwrap_or(d.skill_level),
        potential: query.potential.unwrap_or(d.potential),
        module_level: query.module_level.unwrap_or(d.module_level),
    };
    for (field, v, min, max) in [
        ("promotion", inv.promotion, 0, 2),
        ("skillLevel", inv.skill_level, 1, 10),
        ("potential", inv.potential, 1, 6),
        ("moduleLevel", inv.module_level, 0, 3),
    ] {
        if v < min || v > max {
            return Err(AppError::BadRequest(format!(
                "{field} must be between {min} and {max} (got {v})"
            )));
        }
    }
    let data = data(ctx).await?;
    let key = calc_key("supporters1", &data, &inv);
    ctx.cache
        .json(&key, CALC_TTL, || async move {
            let result = ctx
                .cpu
                .offload("dps_supporters", move || supporters_in(&data, inv))
                .await?;
            to_json(&result)
        })
        .await
}
